use injectra_core::{Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

const LIMIT: u64 = 1024 * 1024;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HtmlMode {
    #[default]
    InnerHtml,
    Dom,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    version: u32,
    id: String,
    css: Option<String>,
    html: Option<String>,
    js: Option<String>,
    #[serde(default)]
    html_mode: HtmlMode,
}

pub struct Bundle {
    pub id: String,
    pub css: String,
    pub html: String,
    pub js: String,
    pub html_mode: HtmlMode,
}

pub fn validate_id(id: &str) -> Result<()> {
    if id.is_empty()
        || id.len() > 80
        || !id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
    {
        return Err(
            "bundle id must be 1..80 ASCII letters, digits, dots, underscores or hyphens".into(),
        );
    }
    Ok(())
}

fn read_small(path: &Path) -> Result<String> {
    use std::io::Read;
    let mut text = String::new();
    fs::File::open(path)?
        .take(LIMIT + 1)
        .read_to_string(&mut text)?;
    if text.len() as u64 > LIMIT {
        return Err("bundle file exceeds 1 MiB".into());
    }
    if let Some(stripped) = text.strip_prefix('\u{feff}') {
        return Ok(stripped.to_owned());
    }
    Ok(text)
}

impl Bundle {
    pub fn load(path: &Path) -> Result<Self> {
        let path = path.canonicalize()?;
        let manifest: Manifest = serde_json::from_str(&read_small(&path)?)
            .map_err(|e| Error::prepare(ErrorKind::InvalidInput, e.to_string()).caused_by(e))?;
        if manifest.version != 1 {
            return Err("unsupported bundle version (expected 1)".into());
        }
        validate_id(&manifest.id)?;
        let root = path.parent().ok_or("bundle has no parent directory")?;
        let read = |name: Option<String>| -> Result<String> {
            match name {
                None => Ok(String::new()),
                Some(name) => {
                    if Path::new(&name).is_absolute() {
                        return Err("bundle assets must be relative paths".into());
                    }
                    let asset = root.join(name).canonicalize()?;
                    if !asset.starts_with(root) {
                        return Err("bundle asset escapes its directory".into());
                    }
                    read_small(&asset)
                }
            }
        };
        let bundle = Self {
            id: manifest.id,
            css: read(manifest.css)?,
            html: read(manifest.html)?,
            js: read(manifest.js)?,
            html_mode: manifest.html_mode,
        };
        if bundle.css.is_empty() && bundle.html.is_empty() && bundle.js.is_empty() {
            return Err("bundle must contain CSS, HTML or JavaScript".into());
        }
        Ok(bundle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod fixture {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/support/bundle.rs"
        ));
    }
    use fixture::BundleFixture;

    #[test]
    fn validates_ids_before_source_generation() {
        for id in ["", "a/b", "x');throw 1;//", "空"] {
            assert!(validate_id(id).is_err());
        }
        assert!(validate_id("theme.dark-v1").is_ok());
    }

    #[test]
    fn loads_ephemeral_contract_input() {
        let input = BundleFixture::new();
        assert_eq!(
            Bundle::load(&input.manifest).unwrap().id,
            "injectra.test.contract"
        );
    }
    #[test]
    fn rejects_escaped_assets_and_unknown_versions() {
        let input = BundleFixture::new();
        std::fs::write(
            &input.manifest,
            r#"{"version":1,"id":"escape","css":"../outside.css"}"#,
        )
        .unwrap();
        let escape = Bundle::load(&input.manifest).err().unwrap();
        assert!(escape.to_string().contains("escapes"));
        std::fs::write(
            &input.manifest,
            r#"{"version":2,"id":"test","js":"script.js"}"#,
        )
        .unwrap();
        let version = Bundle::load(&input.manifest).err().unwrap();
        assert!(version.to_string().contains("unsupported bundle version"));
    }

    #[test]
    fn loads_utf8_bom_manifest_and_assets() {
        let input = BundleFixture::new();
        std::fs::write(
            &input.manifest,
            "\u{feff}{\"version\":1,\"id\":\"bom.bundle\",\"html_mode\":\"dom\",\"html\":\"panel.html\"}",
        )
        .unwrap();
        std::fs::write(
            input.manifest.parent().unwrap().join("panel.html"),
            "\u{feff}<div id=\"bom-ok\">ok</div>",
        )
        .unwrap();
        let loaded = Bundle::load(&input.manifest).unwrap();
        assert_eq!(loaded.id, "bom.bundle");
        assert_eq!(loaded.html_mode, HtmlMode::Dom);
        assert_eq!(loaded.html, "<div id=\"bom-ok\">ok</div>");
    }
}
