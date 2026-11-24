use crate::{LaunchPlan, inspect::chromium_cdp_plan};
use injectra_core::Result;
use injectra_platform_macos::{ApplicationInfo, inspect_app};
use std::path::Path;

pub fn macos_launch_plan(app: &Path, port: u16) -> Result<LaunchPlan> {
    if !cfg!(target_os = "macos") {
        return Err("launch-plan currently supports macOS only".into());
    }
    if port == 0 {
        return Err("debugging port must be nonzero".into());
    }
    plan(inspect_app(app)?, port)
}
fn plan(app: ApplicationInfo, port: u16) -> Result<LaunchPlan> {
    if !app
        .frameworks
        .iter()
        .any(|name| name == "Electron Framework.framework")
    {
        return Err(
            "standard Electron Framework not found; custom bundles are not yet supported".into(),
        );
    }
    Ok(chromium_cdp_plan(app.executable, port))
}
#[cfg(test)]
mod tests {
    use super::*;
    use injectra_platform_macos::CodeSignSummary;
    #[test]
    fn host_recipe_checks_electron_and_preserves_launch_shape() {
        let app = ApplicationInfo {
            bundle_path: "fixture".into(),
            executable: "fixture/Contents/MacOS/electron".into(),
            bundle_id: None,
            version: None,
            frameworks: vec!["Electron Framework.framework".into()],
            framework_details: Vec::new(),
            resources_dir: "fixture/Contents/Resources".into(),
            linked_libraries: Vec::new(),
            codesign: CodeSignSummary::default(),
        };
        let output = plan(app, 9222).unwrap();
        assert_eq!(
            output.arguments,
            [
                "--remote-debugging-port=9222",
                "--remote-debugging-address=127.0.0.1"
            ]
        );
        assert!(output.environment.is_empty());
        let custom = ApplicationInfo {
            bundle_path: "fixture".into(),
            executable: "fixture/custom".into(),
            bundle_id: None,
            version: None,
            frameworks: vec![],
            framework_details: Vec::new(),
            resources_dir: "fixture/Contents/Resources".into(),
            linked_libraries: Vec::new(),
            codesign: CodeSignSummary::default(),
        };
        assert!(
            plan(custom, 9222)
                .unwrap_err()
                .to_string()
                .contains("standard Electron Framework not found")
        );
    }
}
