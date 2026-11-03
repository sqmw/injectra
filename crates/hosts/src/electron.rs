use crate::LaunchPlan;
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
    Ok(LaunchPlan {
        executable: app.executable,
        arguments: vec![
            format!("--remote-debugging-port={port}"),
            "--remote-debugging-address=127.0.0.1".into(),
        ],
        note: "Plan only; does not launch, restart or modify the app. Verify loopback binding and target identity before injection.",
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn host_recipe_checks_electron_and_preserves_launch_shape() {
        let app = ApplicationInfo {
            executable: "fixture/Contents/MacOS/electron".into(),
            frameworks: vec!["Electron Framework.framework".into()],
        };
        let output = plan(app, 9222).unwrap();
        assert_eq!(
            output.arguments,
            [
                "--remote-debugging-port=9222",
                "--remote-debugging-address=127.0.0.1"
            ]
        );
        let custom = ApplicationInfo {
            executable: "fixture/custom".into(),
            frameworks: vec![],
        };
        assert!(
            plan(custom, 9222)
                .unwrap_err()
                .to_string()
                .contains("standard Electron Framework not found")
        );
    }
}
