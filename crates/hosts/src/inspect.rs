//! Cross-host read-only architecture detection, route assessment and launch planning.
use crate::{
    LaunchPlan,
    asar_probe::{PackageSummary, probe_electron_package},
};
use injectra_core::{Error, ErrorKind, Result};
use injectra_platform_macos::{ApplicationInfo, CodeSignSummary};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

const ELECTRON_FUSE_V1: &[u8] = b"dL7pKGdnNz796PbbjQWNKmHXBZaB9tsX";
const ELECTRON_FUSE_V2: &[u8] = b"AGbevlPCksUGKNL8TSn7wGmJEuJsXb2A";
const HANDLE_SCOPE_SYMBOL: &[u8] = b"_ZN2v811HandleScopeC1EPNS_7IsolateE";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostFamily {
    Electron,
    Nwjs,
    Cef,
    SystemWebview,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RouteAssessment {
    pub route: &'static str,
    pub supported: bool,
    pub prerequisites: Vec<String>,
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InspectionReport {
    pub target_path: PathBuf,
    pub platform_format: &'static str,
    pub executable: PathBuf,
    pub host_family: HostFamily,
    pub engine: &'static str,
    pub detected_components: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<PackageSummary>,
    pub routes: Vec<RouteAssessment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_plan: Option<LaunchPlan>,
}

pub(crate) fn chromium_cdp_plan(executable: PathBuf, port: u16) -> LaunchPlan {
    LaunchPlan {
        executable,
        arguments: vec![
            format!("--remote-debugging-port={port}"),
            "--remote-debugging-address=127.0.0.1".into(),
        ],
        environment: BTreeMap::new(),
        note: "Plan only; does not launch, restart or modify the app. Verify loopback binding and target identity before injection.",
    }
}

fn webview2_cdp_plan(executable: PathBuf, port: u16) -> LaunchPlan {
    let mut environment = BTreeMap::new();
    environment.insert(
        "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS".into(),
        format!("--remote-debugging-port={port} --remote-debugging-address=127.0.0.1"),
    );
    LaunchPlan {
        executable,
        arguments: Vec::new(),
        environment,
        note: "Plan only; does not launch, restart or modify the app. Verify loopback binding and target identity before injection.",
    }
}

fn find_subslice_bmh(haystack: &[u8], needle: &[u8]) -> bool {
    let m = needle.len();
    if m == 0 {
        return true;
    }
    if haystack.len() < m {
        return false;
    }
    let last = m - 1;
    let last_byte = needle[last];
    let mut shift = [m; 256];
    for (idx, &b) in needle[..last].iter().enumerate() {
        shift[usize::from(b)] = last - idx;
    }
    let mut i = 0usize;
    let limit = haystack.len() - m;
    while i <= limit {
        let end_byte = haystack[i + last];
        if end_byte == last_byte && &haystack[i..i + m] == needle {
            return true;
        }
        i += shift[usize::from(end_byte)];
    }
    false
}

fn scan_binary_markers(path: &Path, markers: &[&[u8]]) -> Vec<bool> {
    let mut found = vec![false; markers.len()];
    let Ok(mut file) = File::open(path) else {
        return found;
    };
    let overlap = markers.iter().map(|m| m.len()).max().unwrap_or(0);
    let chunk_size = 1024 * 1024;
    let mut buf = vec![0u8; chunk_size + overlap];
    let mut carry = 0usize;
    while let Ok(n) = file.read(&mut buf[carry..]) {
        if n == 0 {
            break;
        }
        let valid = carry + n;
        let window = &buf[..valid];
        for (idx, marker) in markers.iter().enumerate() {
            if !found[idx] && find_subslice_bmh(window, marker) {
                found[idx] = true;
            }
        }
        if found.iter().all(|&f| f) {
            break;
        }
        let keep = valid.min(overlap.saturating_sub(1));
        if keep > 0 {
            buf.copy_within(valid - keep..valid, 0);
        }
        carry = keep;
    }
    found
}

fn dyld_gadget_blocker(codesign: &CodeSignSummary) -> Option<String> {
    if codesign.signed
        && !codesign.adhoc
        && codesign.hardened_runtime
        && !codesign.get_task_allow
        && !(codesign.allow_dyld_env && codesign.disable_library_validation)
    {
        return Some(
            "Hardened Runtime without allow-dyld-environment-variables + disable-library-validation or get-task-allow blocks unsigned/third-party DYLD_INSERT_LIBRARIES".into(),
        );
    }
    None
}

fn electron_prepare_route(
    package: &PackageSummary,
    sandboxed: bool,
    macos_bundle: bool,
) -> RouteAssessment {
    let mut prerequisites = vec!["Optional preparation tools (make setup-preparation)".into()];
    if sandboxed {
        prerequisites.push(
            "App is sandboxed; derived copies require explicitly approved --data-mode shared"
                .into(),
        );
    }
    let mut blockers = Vec::new();
    if macos_bundle {
        if !(cfg!(target_os = "macos") && cfg!(target_arch = "aarch64")) {
            blockers.push("macOS .app preparation requires macOS Apple Silicon host".into());
        }
    } else if !(cfg!(target_os = "windows")
        && (cfg!(target_arch = "x86_64") || cfg!(target_arch = "x86")))
    {
        blockers.push("Windows Electron preparation requires a Windows x64/x86 host".into());
    }
    if !matches!(package.layout, "asar" | "directory" | "nested-asar") {
        blockers.push(format!("Unsupported package layout: {}", package.layout));
    }
    if !matches!(package.entry_kind, "commonjs" | "commonjs-delegate" | "esm") {
        blockers.push(format!(
            "Unsupported main entry kind: {}",
            package.entry_kind
        ));
    }
    RouteAssessment {
        route: "prepare-dev",
        supported: blockers.is_empty(),
        prerequisites,
        blockers,
    }
}

fn windows_v8_route(pe_arch: Option<injectra_platform_windows::pe::PeArch>) -> RouteAssessment {
    let mut blockers = Vec::new();
    if !cfg!(target_os = "windows") {
        blockers.push("Windows V8 attach requires a Windows host".into());
    }
    if let Some(injectra_platform_windows::pe::PeArch::Arm64) = pe_arch {
        blockers.push("Windows V8 MSVC ABI attach currently supports x64 and x86 only".into());
    }
    RouteAssessment {
        route: "windows-v8",
        supported: blockers.is_empty(),
        prerequisites: vec![
            "Optional Python/Frida environment (INJECTRA_FRIDA_PYTHON)".into(),
            "Running explicit target PID with matching executable path and exported MSVC V8 symbols (x64/x86)".into(),
        ],
        blockers,
    }
}

pub(crate) fn classify_macos_bundle(
    info: &ApplicationInfo,
    port: Option<u16>,
    has_handle_scope: Option<bool>,
    has_electron_fuse: Option<bool>,
) -> Result<InspectionReport> {
    let mut detected = Vec::new();
    if let Some(fw) = info.framework_details.iter().find(|f| {
        f.bundle_id.as_deref() == Some("com.github.Electron.framework")
            || f.dir_name == "Electron Framework.framework"
    }) {
        detected.push(fw.dir_name.clone());
        if let Some(v) = &fw.version {
            detected.push(format!("electron-framework:{v}"));
        }
        if has_electron_fuse == Some(true) {
            detected.push("electron-fuse".into());
        }
        let package = probe_electron_package(&info.resources_dir);
        let mut gadget_blockers = Vec::new();
        if let Some(blocker) = dyld_gadget_blocker(&info.codesign) {
            gadget_blockers.push(blocker);
        }
        if has_handle_scope == Some(false) {
            gadget_blockers.push(
                "Electron framework lacks V8 HandleScope export (_ZN2v811HandleScopeC1EPNS_7IsolateE)".into(),
            );
        }
        let routes = vec![
            RouteAssessment {
                route: "cdp",
                supported: true,
                prerequisites: vec![
                    "Launch target with --remote-debugging-port=<PORT> --remote-debugging-address=127.0.0.1 and verify an accessible page endpoint".into(),
                ],
                blockers: Vec::new(),
            },
            RouteAssessment {
                route: "electron-gadget",
                supported: gadget_blockers.is_empty(),
                prerequisites: vec![
                    "Optional Python/Frida environment (make setup-frida)".into(),
                    "Compatible FridaGadget.dylib (make setup-gadget)".into(),
                ],
                blockers: gadget_blockers,
            },
            electron_prepare_route(&package, info.codesign.sandboxed, true),
        ];
        return Ok(InspectionReport {
            target_path: info.bundle_path.clone(),
            platform_format: "macos-app",
            executable: info.executable.clone(),
            host_family: HostFamily::Electron,
            engine: "chromium",
            detected_components: detected,
            package: Some(package),
            routes,
            launch_plan: port.map(|p| chromium_cdp_plan(info.executable.clone(), p)),
        });
    }

    if let Some(fw) = info.framework_details.iter().find(|f| {
        f.bundle_id.as_deref() == Some("io.nwjs.nwjs.framework")
            || f.dir_name == "nwjs Framework.framework"
    }) {
        detected.push(fw.dir_name.clone());
        return Ok(InspectionReport {
            target_path: info.bundle_path.clone(),
            platform_format: "macos-app",
            executable: info.executable.clone(),
            host_family: HostFamily::Nwjs,
            engine: "chromium",
            detected_components: detected,
            package: None,
            routes: vec![RouteAssessment {
                route: "cdp",
                supported: true,
                prerequisites: vec![
                    "Launch target with --remote-debugging-port=<PORT> --remote-debugging-address=127.0.0.1".into(),
                ],
                blockers: Vec::new(),
            }],
            launch_plan: port.map(|p| chromium_cdp_plan(info.executable.clone(), p)),
        });
    }

    if let Some(fw) = info.framework_details.iter().find(|f| {
        f.bundle_id.as_deref() == Some("org.cef.framework")
            || f.dir_name == "Chromium Embedded Framework.framework"
    }) {
        detected.push(fw.dir_name.clone());
        return Ok(InspectionReport {
            target_path: info.bundle_path.clone(),
            platform_format: "macos-app",
            executable: info.executable.clone(),
            host_family: HostFamily::Cef,
            engine: "chromium",
            detected_components: detected,
            package: None,
            routes: vec![RouteAssessment {
                route: "cdp",
                supported: true,
                prerequisites: vec![
                    "Launch target with --remote-debugging-port=<PORT> --remote-debugging-address=127.0.0.1; native-only startup views without an initialized CEF page will not appear in targets".into(),
                ],
                blockers: Vec::new(),
            }],
            launch_plan: port.map(|p| chromium_cdp_plan(info.executable.clone(), p)),
        });
    }

    if info
        .linked_libraries
        .iter()
        .any(|lib| lib.contains("WebKit.framework"))
    {
        detected.push("WebKit.framework".into());
        let blockers = dyld_gadget_blocker(&info.codesign)
            .into_iter()
            .collect::<Vec<_>>();
        return Ok(InspectionReport {
            target_path: info.bundle_path.clone(),
            platform_format: "macos-app",
            executable: info.executable.clone(),
            host_family: HostFamily::SystemWebview,
            engine: "wkwebview",
            detected_components: detected,
            package: None,
            routes: vec![RouteAssessment {
                route: "webkit",
                supported: blockers.is_empty(),
                prerequisites: vec![
                    "Optional Python/Frida environment (make setup-frida)".into(),
                    "Compatible FridaGadget.dylib (make setup-gadget)".into(),
                ],
                blockers,
            }],
            launch_plan: None,
        });
    }

    Err(Error::prepare(
        ErrorKind::Unsupported,
        "Unsupported: no recognized Web UI host architecture in .app bundle (native-only or heavily packed target)",
    ))
}

fn classify_directory_or_executable(path: &Path, port: Option<u16>) -> Result<InspectionReport> {
    let layout = injectra_platform_windows::inspect_layout(path)?;
    let executable = layout.executable;

    let mut detected = Vec::new();
    if let Some(arch) = layout.pe_arch {
        detected.push(format!("pe-{}", arch.as_str()));
    }
    if !layout.asar_integrity.is_empty() {
        detected.push("pe-asar-integrity".into());
    }

    let mut has_nw = false;
    let mut has_cef = false;
    let mut has_webview2 = false;
    let electron_resources = layout.resources_dirs.first().cloned();

    for file in &layout.component_files {
        let name = file
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if name.eq_ignore_ascii_case("nw.dll") || name == "libnw.so" {
            has_nw = true;
            if !detected.iter().any(|d| d == name) {
                detected.push(name.to_owned());
            }
        } else if name.eq_ignore_ascii_case("libcef.dll") || name == "libcef.so" {
            has_cef = true;
            if !detected.iter().any(|d| d == name) {
                detected.push(name.to_owned());
            }
        } else if name.eq_ignore_ascii_case("WebView2Loader.dll") {
            has_webview2 = true;
            if !detected.iter().any(|d| d == name) {
                detected.push(name.to_owned());
            }
        } else if name == "app.asar" && !detected.iter().any(|d| d == "app.asar") {
            detected.push("app.asar".into());
        } else if name == "package.json"
            && file
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|s| s.to_str())
                == Some("app")
            && !detected.iter().any(|d| d == "app/package.json")
        {
            detected.push("app/package.json".into());
        }
    }

    let exe_markers = scan_binary_markers(&executable, &[ELECTRON_FUSE_V1, ELECTRON_FUSE_V2]);
    let has_electron_fuse = exe_markers.iter().any(|&found| found);
    if has_electron_fuse {
        detected.push("electron-fuse".into());
    }

    let platform_format = if path.is_file() {
        "executable"
    } else {
        "directory"
    };

    if electron_resources.is_some() || has_electron_fuse {
        let package = electron_resources
            .as_deref()
            .map(probe_electron_package)
            .unwrap_or(PackageSummary {
                layout: "missing",
                main_entry: None,
                entry_kind: "missing",
            });
        let routes = vec![
            RouteAssessment {
                route: "cdp",
                supported: true,
                prerequisites: vec![
                    "Launch target with --remote-debugging-port=<PORT> --remote-debugging-address=127.0.0.1".into(),
                ],
                blockers: Vec::new(),
            },
            electron_prepare_route(&package, false, false),
            windows_v8_route(layout.pe_arch),
        ];
        return Ok(InspectionReport {
            target_path: path.to_path_buf(),
            platform_format,
            executable: executable.clone(),
            host_family: HostFamily::Electron,
            engine: "chromium",
            detected_components: detected,
            package: Some(package),
            routes,
            launch_plan: port.map(|p| chromium_cdp_plan(executable, p)),
        });
    }

    if has_nw {
        return Ok(InspectionReport {
            target_path: path.to_path_buf(),
            platform_format,
            executable: executable.clone(),
            host_family: HostFamily::Nwjs,
            engine: "chromium",
            detected_components: detected,
            package: None,
            routes: vec![RouteAssessment {
                route: "cdp",
                supported: true,
                prerequisites: vec![
                    "Launch target with --remote-debugging-port=<PORT> --remote-debugging-address=127.0.0.1".into(),
                ],
                blockers: Vec::new(),
            }],
            launch_plan: port.map(|p| chromium_cdp_plan(executable, p)),
        });
    }

    if has_cef {
        return Ok(InspectionReport {
            target_path: path.to_path_buf(),
            platform_format,
            executable: executable.clone(),
            host_family: HostFamily::Cef,
            engine: "chromium",
            detected_components: detected,
            package: None,
            routes: vec![RouteAssessment {
                route: "cdp",
                supported: true,
                prerequisites: vec![
                    "Launch target with --remote-debugging-port=<PORT> --remote-debugging-address=127.0.0.1; native-only startup windows without initialized CEF pages will not appear in targets".into(),
                ],
                blockers: Vec::new(),
            }],
            launch_plan: port.map(|p| chromium_cdp_plan(executable, p)),
        });
    }

    if has_webview2 {
        return Ok(InspectionReport {
            target_path: path.to_path_buf(),
            platform_format,
            executable: executable.clone(),
            host_family: HostFamily::SystemWebview,
            engine: "webview2",
            detected_components: detected,
            package: None,
            routes: vec![RouteAssessment {
                route: "cdp",
                supported: true,
                prerequisites: vec![
                    "Launch target with WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=<PORT> --remote-debugging-address=127.0.0.1".into(),
                ],
                blockers: Vec::new(),
            }],
            launch_plan: port.map(|p| webview2_cdp_plan(executable, p)),
        });
    }

    Err(Error::prepare(
        ErrorKind::Unsupported,
        "Unsupported: no recognized Web UI host architecture in target path (native-only or heavily packed target)",
    ))
}

pub fn inspect_app(path: &Path, port: Option<u16>) -> Result<InspectionReport> {
    if port == Some(0) {
        return Err(Error::prepare(
            ErrorKind::InvalidInput,
            "debugging port must be nonzero",
        ));
    }
    let canonical = injectra_platform_windows::normalize_canonical_path(path)?;
    if cfg!(target_os = "macos") && canonical.extension().and_then(|s| s.to_str()) == Some("app") {
        let info = injectra_platform_macos::inspect_app(&canonical)?;
        let electron_fw_exe = info
            .framework_details
            .iter()
            .find(|f| {
                f.bundle_id.as_deref() == Some("com.github.Electron.framework")
                    || f.dir_name == "Electron Framework.framework"
            })
            .and_then(|f| f.executable.as_deref());
        let (has_handle_scope, has_fuse) = if let Some(fw_exe) = electron_fw_exe {
            let markers = scan_binary_markers(
                fw_exe,
                &[HANDLE_SCOPE_SYMBOL, ELECTRON_FUSE_V1, ELECTRON_FUSE_V2],
            );
            (Some(markers[0]), Some(markers[1] || markers[2]))
        } else {
            (None, None)
        };
        return classify_macos_bundle(&info, port, has_handle_scope, has_fuse);
    }
    classify_directory_or_executable(&canonical, port)
}

#[cfg(test)]
mod tests {
    use super::*;
    use injectra_platform_macos::FrameworkInfo;
    use std::fs;

    fn mac_app(
        frameworks: Vec<FrameworkInfo>,
        linked_libraries: Vec<&str>,
        codesign: CodeSignSummary,
        resources_dir: PathBuf,
    ) -> ApplicationInfo {
        ApplicationInfo {
            bundle_path: PathBuf::from("/tmp/Test.app"),
            executable: PathBuf::from("/tmp/Test.app/Contents/MacOS/Test"),
            bundle_id: Some("io.test.app".into()),
            version: Some("1.0.0".into()),
            frameworks: frameworks
                .iter()
                .map(|f| f.dir_name.clone().into())
                .collect(),
            framework_details: frameworks,
            resources_dir,
            linked_libraries: linked_libraries.into_iter().map(str::to_owned).collect(),
            codesign,
        }
    }

    fn temp_dir(label: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("injectra-inspect-test-{label}-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn classifies_electron_36_vs_45_and_renamed_framework() {
        let dir = temp_dir("electron");
        let app_dir = dir.join("app");
        fs::create_dir_all(&app_dir).unwrap();
        fs::write(
            app_dir.join("package.json"),
            br#"{"name":"t","main":"main.js"}"#,
        )
        .unwrap();
        fs::write(app_dir.join("main.js"), b"console.log('ok');").unwrap();

        let info = mac_app(
            vec![FrameworkInfo {
                dir_name: "RenamedHost.framework".into(),
                bundle_id: Some("com.github.Electron.framework".into()),
                version: Some("36.4.0".into()),
                executable: None,
            }],
            vec![],
            CodeSignSummary::default(),
            dir.clone(),
        );
        let e36 = classify_macos_bundle(&info, Some(9222), Some(true), Some(true)).unwrap();
        assert_eq!(e36.host_family, HostFamily::Electron);
        assert!(
            e36.routes
                .iter()
                .find(|r| r.route == "electron-gadget")
                .unwrap()
                .supported
        );
        assert_eq!(
            e36.launch_plan.as_ref().unwrap().arguments,
            [
                "--remote-debugging-port=9222",
                "--remote-debugging-address=127.0.0.1"
            ]
        );

        let e45 = classify_macos_bundle(&info, Some(9222), Some(false), Some(true)).unwrap();
        let gadget = e45
            .routes
            .iter()
            .find(|r| r.route == "electron-gadget")
            .unwrap();
        assert!(!gadget.supported);
        assert!(gadget.blockers.iter().any(|b| b.contains("HandleScope")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn classifies_cef_nwjs_wkwebview_and_rejects_native_only() {
        let empty = temp_dir("empty");
        let cef = mac_app(
            vec![FrameworkInfo {
                dir_name: "Chromium Embedded Framework.framework".into(),
                bundle_id: Some("org.cef.framework".into()),
                version: Some("156.0.26".into()),
                executable: None,
            }],
            vec![],
            CodeSignSummary::default(),
            empty.clone(),
        );
        let cef_report = classify_macos_bundle(&cef, Some(9222), None, None).unwrap();
        assert_eq!(cef_report.host_family, HostFamily::Cef);
        assert!(cef_report.launch_plan.is_some());

        let nwjs = mac_app(
            vec![FrameworkInfo {
                dir_name: "nwjs Framework.framework".into(),
                bundle_id: Some("io.nwjs.nwjs.framework".into()),
                version: None,
                executable: None,
            }],
            vec![],
            CodeSignSummary::default(),
            empty.clone(),
        );
        assert_eq!(
            classify_macos_bundle(&nwjs, None, None, None)
                .unwrap()
                .host_family,
            HostFamily::Nwjs
        );

        let wk_blocked = mac_app(
            vec![],
            vec!["/System/Library/Frameworks/WebKit.framework/Versions/A/WebKit"],
            CodeSignSummary {
                signed: true,
                adhoc: false,
                hardened_runtime: true,
                ..CodeSignSummary::default()
            },
            empty.clone(),
        );
        let wk_report = classify_macos_bundle(&wk_blocked, Some(9222), None, None).unwrap();
        assert_eq!(wk_report.host_family, HostFamily::SystemWebview);
        assert_eq!(wk_report.engine, "wkwebview");
        assert!(wk_report.launch_plan.is_none());
        assert!(!wk_report.routes[0].supported);

        let native = mac_app(
            vec![],
            vec!["/usr/lib/libSystem.B.dylib"],
            CodeSignSummary::default(),
            empty.clone(),
        );
        let err = classify_macos_bundle(&native, None, None, None).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Unsupported);
        let _ = fs::remove_dir_all(&empty);
    }

    #[test]
    fn classifies_webview2_directory_and_rejects_unknown_directory() {
        let wv2 = temp_dir("wv2");
        fs::write(wv2.join("Sample.exe"), b"MZ").unwrap();
        fs::write(wv2.join("WebView2Loader.dll"), b"MZ").unwrap();
        let report = inspect_app(&wv2, Some(9333)).unwrap();
        assert_eq!(report.host_family, HostFamily::SystemWebview);
        assert_eq!(report.engine, "webview2");
        let plan = report.launch_plan.unwrap();
        assert!(plan.arguments.is_empty());
        assert_eq!(
            plan.environment
                .get("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS")
                .map(String::as_str),
            Some("--remote-debugging-port=9333 --remote-debugging-address=127.0.0.1")
        );
        let _ = fs::remove_dir_all(&wv2);

        let native = temp_dir("native");
        fs::write(native.join("Plain.exe"), b"MZ").unwrap();
        assert_eq!(
            inspect_app(&native, None).unwrap_err().kind,
            ErrorKind::Unsupported
        );
        let _ = fs::remove_dir_all(&native);

        let win_esm = temp_dir("win-esm");
        let app_dir = win_esm.join("app-1.0.0/resources/app");
        fs::create_dir_all(&app_dir).unwrap();
        fs::write(win_esm.join("Update.exe"), b"MZ").unwrap();
        fs::write(win_esm.join("app-1.0.0/Sample.exe"), b"MZ").unwrap();
        fs::write(
            app_dir.join("package.json"),
            br#"{"name":"esm-app","type":"module","main":"main.mjs"}"#,
        )
        .unwrap();
        fs::write(app_dir.join("main.mjs"), b"export default {};").unwrap();
        let esm_report = inspect_app(&win_esm, Some(9444)).unwrap();
        assert_eq!(esm_report.host_family, HostFamily::Electron);
        assert_eq!(esm_report.executable.file_name().unwrap(), "Sample.exe");
        assert_eq!(esm_report.package.as_ref().unwrap().entry_kind, "esm");
        let prep = esm_report
            .routes
            .iter()
            .find(|r| r.route == "prepare-dev")
            .unwrap();
        assert!(
            !prep
                .blockers
                .iter()
                .any(|b| b.contains("Unsupported main entry kind"))
        );
        assert!(esm_report.routes.iter().any(|r| r.route == "windows-v8"));
        let _ = fs::remove_dir_all(&win_esm);
    }
}
