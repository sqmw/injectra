use crate::{
    MAX_FILE_BYTES, MAX_WORKSPACE_BYTES, Snapshot, SourceFile, SourceOrigin, SourceReference,
    content_hash,
    filesystem::{Root, user_path},
    invalid, json,
    model::Manifest,
    unique_id,
};
use injectra_core::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};
const CONFIG: &str = "injectra.workspace.json";
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    version: u32,
    id: String,
    imports: BTreeMap<String, SourceReference>,
}

type Captured = (Manifest, Vec<SourceFile>, BTreeMap<String, Vec<u8>>);

pub struct Workspace {
    root: PathBuf,
    config: Config,
    lock: Mutex<()>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SaveOutcome {
    Saved { file: SourceFile },
    Conflict { current_hash: Option<String> },
}
impl Workspace {
    fn empty(root: &Path) -> Result<PathBuf> {
        if root.exists() {
            if fs::symlink_metadata(root)?.file_type().is_symlink()
                || !root.is_dir()
                || fs::read_dir(root)?.next().is_some()
            {
                return Err(invalid(
                    "workspace destination must be a new or empty directory",
                ));
            }
        } else {
            fs::create_dir_all(root)?;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(root, fs::Permissions::from_mode(0o700))?;
        }
        Ok(root.canonicalize()?)
    }
    pub fn create(root: &Path, bundle_id: &str) -> Result<Self> {
        let manifest = Manifest {
            version: 1,
            id: bundle_id.to_owned(),
            js: Some("page.js".into()),
            css: Some("page.css".into()),
            html: Some("page.html".into()),
        };
        Manifest::parse(&json(&manifest)?)?;
        let root = Self::empty(root)?;
        let dir = Root::open(&root)?;
        dir.write(
            "page.js",
            b"// User page increment. Run explicitly in the selected page.\n",
            true,
        )?;
        dir.write("page.css", b"", true)?;
        dir.write("page.html", b"", true)?;
        Self::finish(root, manifest, BTreeMap::new())
    }
    pub fn import_manifest(root: &Path, manifest_path: &Path) -> Result<Self> {
        let name = manifest_path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| invalid("invalid manifest path"))?;
        let parent = manifest_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .canonicalize()?;
        let source = Root::open(&parent)?;
        let manifest_bytes = source.read(name)?;
        let mut manifest = Manifest::parse_import(&manifest_bytes)?;
        let mut assets = Vec::new();
        let mut total = 0;
        for (asset, destination) in [
            (&mut manifest.js, "page.js"),
            (&mut manifest.css, "page.css"),
            (&mut manifest.html, "page.html"),
        ] {
            if let Some(original) = asset.as_ref() {
                let mut components = Vec::new();
                for component in Path::new(original).components() {
                    match component {
                        std::path::Component::CurDir => (),
                        std::path::Component::Normal(part) => components.push(
                            part.to_str()
                                .ok_or_else(|| invalid("non-UTF8 import path"))?,
                        ),
                        _ => {
                            return Err(invalid(
                                "import asset must remain within its manifest directory",
                            ));
                        }
                    }
                }
                let path = components.join("/");
                let bytes = source.read(&path)?;
                total += bytes.len();
                if total > MAX_WORKSPACE_BYTES {
                    return Err(invalid("import exceeds workspace limit"));
                }
                assets.push((destination.to_owned(), path, bytes));
                *asset = Some(destination.into());
            }
        }
        // Manifest v1 page assets may be named startup.cjs or use ./ prefixes.
        // Normalize copied files without confusing them with the main-process increment.
        Manifest::parse(&json(&manifest)?)?;
        let root = Self::empty(root)?;
        let dest = Root::open(&root)?;
        let mut imports = BTreeMap::new();
        imports.insert(
            "manifest.json".into(),
            SourceReference {
                origin: SourceOrigin::User,
                path: parent.join(name).to_string_lossy().into_owned(),
                hash: content_hash(&manifest_bytes),
            },
        );
        for (destination, path, bytes) in assets {
            dest.write(&destination, &bytes, true)?;
            imports.insert(
                destination,
                SourceReference {
                    origin: SourceOrigin::User,
                    path: parent.join(path).to_string_lossy().into_owned(),
                    hash: content_hash(&bytes),
                },
            );
        }
        Self::finish(root, manifest, imports)
    }
    fn finish(
        root: PathBuf,
        manifest: Manifest,
        imports: BTreeMap<String, SourceReference>,
    ) -> Result<Self> {
        let dir = Root::open(&root)?;
        let config = Config {
            version: 1,
            id: unique_id()?,
            imports,
        };
        dir.write("manifest.json", &json(&manifest)?, true)?;
        dir.write(CONFIG, &json(&config)?, true)?;
        Self::open(&root)
    }
    pub fn open(root: &Path) -> Result<Self> {
        if fs::symlink_metadata(root)?.file_type().is_symlink() {
            return Err(invalid("workspace root must not be a symlink"));
        }
        let root = root.canonicalize()?;
        let dir = Root::open(&root)?;
        let config: Config =
            serde_json::from_slice(&dir.read(CONFIG)?).map_err(|e| invalid(e.to_string()))?;
        if config.version != 1
            || config.id.len() != 32
            || !config.id.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(invalid("unsupported or invalid workspace identity"));
        }
        let workspace = Self {
            root,
            config,
            lock: Mutex::new(()),
        };
        workspace.capture()?;
        Ok(workspace)
    }
    pub fn id(&self) -> &str {
        &self.config.id
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    fn file(&self, path: &str, bytes: &[u8], manifest: &Manifest) -> SourceFile {
        SourceFile {
            path: path.into(),
            origin: SourceOrigin::User,
            stage: manifest.stage(path),
            hash: content_hash(bytes),
            bytes: bytes.len(),
            editable: path != "manifest.json",
            imported_from: self.config.imports.get(path).cloned(),
        }
    }
    pub(crate) fn capture(&self) -> Result<Captured> {
        let dir = Root::open(&self.root)?;
        let manifest = Manifest::parse(&dir.read("manifest.json")?)?;
        let mut data = BTreeMap::new();
        let mut files = Vec::new();
        let mut total = 0;
        for path in dir.files()? {
            if path == CONFIG {
                continue;
            }
            user_path(&path)?;
            let bytes = dir.read(&path)?;
            total += bytes.len();
            if total > MAX_WORKSPACE_BYTES {
                return Err(invalid("workspace exceeds 8 MiB"));
            }
            files.push(self.file(&path, &bytes, &manifest));
            data.insert(path, bytes);
        }
        for asset in [&manifest.js, &manifest.css, &manifest.html]
            .into_iter()
            .flatten()
        {
            if !data.contains_key(asset) {
                return Err(invalid(format!("missing bundle asset: {asset}")));
            }
        }
        // Re-read entries and manifest to reject a changing source tree during capture.
        for (path, bytes) in &data {
            if dir.read(path)? != *bytes {
                return Err(invalid(
                    "workspace changed during snapshot; retry explicitly",
                ));
            }
        }
        let names: Vec<_> = dir.files()?.into_iter().filter(|p| p != CONFIG).collect();
        if names != data.keys().cloned().collect::<Vec<_>>() {
            return Err(invalid("workspace file set changed during snapshot"));
        }
        Ok((manifest, files, data))
    }
    pub fn files(&self) -> Result<Vec<SourceFile>> {
        Ok(self.capture()?.1)
    }
    pub fn read(&self, path: &str) -> Result<String> {
        user_path(path)?;
        if path == CONFIG {
            return Err(invalid("tool metadata is not a user source"));
        }
        String::from_utf8(Root::open(&self.root)?.read(path)?)
            .map_err(|_| invalid("source is not UTF-8"))
    }
    pub fn save(&self, path: &str, content: &str) -> Result<SourceFile> {
        self.save_inner(path, content, None, false)
            .map(|value| match value {
                SaveOutcome::Saved { file } => file,
                SaveOutcome::Conflict { .. } => unreachable!(),
            })
    }
    pub fn save_if_match(
        &self,
        path: &str,
        content: &str,
        expected_hash: Option<&str>,
    ) -> Result<SaveOutcome> {
        self.save_inner(path, content, expected_hash, true)
    }
    fn save_inner(
        &self,
        path: &str,
        content: &str,
        expected: Option<&str>,
        conditional: bool,
    ) -> Result<SaveOutcome> {
        user_path(path)?;
        if path == CONFIG || path == "manifest.json" {
            return Err(invalid(
                "workspace manifest and tool metadata are read-only through the editor",
            ));
        }
        if content.len() > MAX_FILE_BYTES {
            return Err(invalid("source exceeds 1 MiB"));
        }
        let _lock = self
            .lock
            .lock()
            .map_err(|_| invalid("workspace write lock poisoned"))?;
        let (manifest, files, _) = self.capture()?;
        let current = files.iter().find(|f| f.path == path);
        if conditional && current.map(|f| f.hash.as_str()) != expected {
            return Ok(SaveOutcome::Conflict {
                current_hash: current.map(|f| f.hash.clone()),
            });
        }
        let total: usize = files
            .iter()
            .filter(|f| f.path != path)
            .map(|f| f.bytes)
            .sum();
        if total + content.len() > MAX_WORKSPACE_BYTES
            || current.is_none() && files.len() >= crate::MAX_FILES - 1
        {
            return Err(invalid("workspace size/file limit exceeded"));
        }
        Root::open(&self.root)?.write(path, content.as_bytes(), false)?;
        Ok(SaveOutcome::Saved {
            file: self.file(path, content.as_bytes(), &manifest),
        })
    }
    pub fn snapshot(&self, cache_root: &Path) -> Result<Snapshot> {
        let _lock = self
            .lock
            .lock()
            .map_err(|_| invalid("workspace write lock poisoned"))?;
        Snapshot::capture(self, cache_root)
    }
}
