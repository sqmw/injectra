use crate::{
    GeneratedProgram, SourceFile, SourceMapping, SourceOrigin, Workspace, content_hash,
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
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub workspace_id: String,
    pub revision: String,
    pub root: PathBuf,
    pub manifest: PathBuf,
    pub startup: Option<PathBuf>,
    pub files: Vec<SourceFile>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    version: u32,
    workspace_id: String,
    revision: String,
    files: Vec<SourceFile>,
}
fn revision(id: &str, files: &[SourceFile]) -> Result<String> {
    Ok(content_hash(
        &serde_json::to_vec(&(id, files)).map_err(|e| invalid(e.to_string()))?,
    ))
}
// Resolve existing ancestors before creating a requested cache directory.
fn resolve_directory(path: &Path) -> Result<PathBuf> {
    let mut out = PathBuf::new();
    for component in std::path::absolute(path)?.components() {
        match component {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => (),
            other => {
                out.push(other.as_os_str());
                match fs::symlink_metadata(&out) {
                    Ok(_) => {
                        out = out.canonicalize()?;
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
                    Err(e) => return Err(e.into()),
                }
            }
        }
    }
    Ok(out)
}
impl Snapshot {
    pub(crate) fn capture(workspace: &Workspace, cache: &Path) -> Result<Self> {
        let (_, files, data) = workspace.capture()?;
        let cache = resolve_directory(cache)?;
        if cache.starts_with(workspace.root()) {
            return Err(invalid("snapshot cache must be outside workspace"));
        }
        fs::create_dir_all(&cache)?;
        if cache.starts_with(workspace.root()) {
            return Err(invalid("snapshot cache must be outside workspace"));
        }
        let revision = revision(workspace.id(), &files)?;
        let mut parent = cache;
        for name in ["workspaces", workspace.id(), "revisions"] {
            parent.push(name);
            match fs::symlink_metadata(&parent) {
                Ok(meta) if meta.is_dir() && !meta.file_type().is_symlink() => (),
                Ok(_) => {
                    return Err(invalid(
                        "snapshot cache component is not an owned directory",
                    ));
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    let mut builder = fs::DirBuilder::new();
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::DirBuilderExt;
                        builder.mode(0o700);
                    }
                    builder.create(&parent)?;
                }
                Err(error) => return Err(error.into()),
            }
        }
        let target = parent.join(&revision);
        if target.exists() {
            let found = Self::load(&target)?;
            if found.workspace_id != workspace.id() || found.revision != revision {
                return Err(invalid("snapshot cache identity mismatch"));
            }
            return Ok(found);
        }
        let pending = parent.join(format!(".pending-{}", unique_id()?));
        fs::create_dir(&pending)?;
        let payload = pending.join("files");
        fs::create_dir(&payload)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&pending, fs::Permissions::from_mode(0o700))?;
        }
        let dir = Root::open(&payload)?;
        for (path, bytes) in &data {
            dir.write(path, bytes, true)?;
        }
        let record = Record {
            version: 1,
            workspace_id: workspace.id().into(),
            revision: revision.clone(),
            files,
        };
        fs::write(pending.join("snapshot.json"), json(&record)?)?;
        // Publish only complete revisions. A competing publisher must never be replaced.
        #[cfg(unix)]
        {
            use rustix::fs::{CWD, RenameFlags, renameat_with};
            match renameat_with(CWD, &pending, CWD, &target, RenameFlags::NOREPLACE) {
                Ok(()) => (),
                Err(rustix::io::Errno::EXIST) => {
                    return Self::load(&target);
                }
                Err(e) => return Err(std::io::Error::from(e).into()),
            }
        }
        #[cfg(not(unix))]
        {
            return Err(injectra_core::Error::prepare(
                injectra_core::ErrorKind::Unsupported,
                "atomic snapshot publication requires supported Unix filesystem",
            ));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            for path in data.keys() {
                fs::set_permissions(
                    target.join("files").join(path),
                    fs::Permissions::from_mode(0o400),
                )?;
            }
            fs::set_permissions(
                target.join("snapshot.json"),
                fs::Permissions::from_mode(0o400),
            )?;
            fn seal(path: &Path) -> std::io::Result<()> {
                for entry in fs::read_dir(path)? {
                    let entry = entry?;
                    if entry.file_type()?.is_dir() {
                        seal(&entry.path())?;
                    }
                }
                fs::set_permissions(path, fs::Permissions::from_mode(0o500))
            }
            seal(&target)?;
        }
        Self::load(&target)
    }
    /// Load a published revision directory containing snapshot.json and files/.
    pub fn load(directory: &Path) -> Result<Self> {
        if fs::symlink_metadata(directory)?.file_type().is_symlink() {
            return Err(invalid("snapshot directory must not be a symlink"));
        }
        let directory = directory.canonicalize()?;
        let root = directory.join("files");
        let record: Record =
            serde_json::from_slice(&Root::open(&directory)?.read("snapshot.json")?)
                .map_err(|e| invalid(e.to_string()))?;
        if record.version != 1
            || record.revision != revision(&record.workspace_id, &record.files)?
            || directory.file_name().and_then(|s| s.to_str()) != Some(record.revision.as_str())
        {
            return Err(invalid("invalid snapshot descriptor or identity"));
        }
        let startup = record
            .files
            .iter()
            .any(|f| f.path == "startup.cjs")
            .then(|| root.join("startup.cjs"));
        let snapshot = Self {
            workspace_id: record.workspace_id,
            revision: record.revision,
            manifest: root.join("manifest.json"),
            root,
            startup,
            files: record.files,
        };
        snapshot.verify()?;
        Ok(snapshot)
    }
    fn contents(&self) -> Result<BTreeMap<String, Vec<u8>>> {
        if self.revision != revision(&self.workspace_id, &self.files)?
            || self.manifest != self.root.join("manifest.json")
            || self.startup
                != self
                    .files
                    .iter()
                    .any(|f| f.path == "startup.cjs")
                    .then(|| self.root.join("startup.cjs"))
        {
            return Err(invalid("snapshot metadata changed"));
        }
        let dir = Root::open(&self.root)?;
        let mut data = BTreeMap::new();
        let mut total = 0;
        if self.files.len() > crate::MAX_FILES {
            return Err(invalid("snapshot file limit exceeded"));
        }
        for file in &self.files {
            user_path(&file.path)?;
            if file.origin != SourceOrigin::User || data.contains_key(&file.path) {
                return Err(invalid("invalid snapshot source provenance"));
            }
            let bytes = dir.read(&file.path)?;
            total += bytes.len();
            if total > crate::MAX_WORKSPACE_BYTES
                || bytes.len() != file.bytes
                || content_hash(&bytes) != file.hash
            {
                return Err(invalid("snapshot contents changed"));
            }
            data.insert(file.path.clone(), bytes);
        }
        if dir.files()? != data.keys().cloned().collect::<Vec<_>>() {
            return Err(invalid("snapshot file set changed"));
        }
        Ok(data)
    }
    pub fn verify(&self) -> Result<()> {
        let bytes = self.contents()?;
        self.bundle_from(&bytes)?;
        Ok(())
    }
    fn bundle_from(&self, files: &BTreeMap<String, Vec<u8>>) -> Result<injectra_web::Bundle> {
        let manifest = Manifest::parse(
            files
                .get("manifest.json")
                .ok_or_else(|| invalid("snapshot has no manifest"))?,
        )?;
        let asset = |path: Option<String>| -> Result<String> {
            match path {
                None => Ok(String::new()),
                Some(path) => String::from_utf8(
                    files
                        .get(&path)
                        .ok_or_else(|| invalid("snapshot asset missing"))?
                        .clone(),
                )
                .map_err(|_| invalid("snapshot asset not UTF-8")),
            }
        };
        let bundle = injectra_web::Bundle {
            id: manifest.id,
            js: asset(manifest.js)?,
            css: asset(manifest.css)?,
            html: asset(manifest.html)?,
        };
        if bundle.js.is_empty() && bundle.css.is_empty() && bundle.html.is_empty() {
            return Err(invalid("bundle has no executable or display content"));
        }
        Ok(bundle)
    }
    pub fn bundle(&self) -> Result<injectra_web::Bundle> {
        self.bundle_from(&self.contents()?)
    }
    pub fn startup_source(&self) -> Result<Option<String>> {
        let files = self.contents()?;
        files
            .get("startup.cjs")
            .map(|bytes| {
                String::from_utf8(bytes.clone()).map_err(|_| invalid("startup source is not UTF-8"))
            })
            .transpose()
    }
    pub fn mapped_program(&self) -> Result<GeneratedProgram> {
        let files = self.contents()?;
        let manifest = Manifest::parse(&files["manifest.json"])?;
        let bundle = self.bundle_from(&files)?;
        let mut source = injectra_web::apply(&bundle)?;
        let source_url = format!(
            "injectra://{}/{}/generated/page-program.js",
            self.workspace_id, self.revision
        );
        let marker = format!("\n{}\n}})", bundle.js);
        let offset = source
            .rfind(&marker)
            .ok_or_else(|| invalid("shared runtime source mapping shape changed"))?
            + 1;
        let start = source[..offset].bytes().filter(|b| *b == b'\n').count() + 1;
        let user_lines = if bundle.js.is_empty() {
            0
        } else {
            bundle.js.bytes().filter(|b| *b == b'\n').count() + 1
        };
        let total = source.lines().count();
        let mut mappings = vec![SourceMapping {
            origin: SourceOrigin::Injectra,
            path: "web/runtime.js".into(),
            revision: self.revision.clone(),
            source_url: source_url.clone(),
            generated_start_line: 1,
            generated_end_line: start - 1,
            source_start_line: 1,
        }];
        if let Some(path) = manifest.js.filter(|_| user_lines > 0) {
            mappings.push(SourceMapping {
                origin: SourceOrigin::User,
                path,
                revision: self.revision.clone(),
                source_url: source_url.clone(),
                generated_start_line: start,
                generated_end_line: start + user_lines - 1,
                source_start_line: 1,
            });
        }
        mappings.push(SourceMapping {
            origin: SourceOrigin::Injectra,
            path: "generated/page-wrapper.js".into(),
            revision: self.revision.clone(),
            source_url: source_url.clone(),
            generated_start_line: start + user_lines,
            generated_end_line: total,
            source_start_line: 1,
        });
        source.push_str(&format!("\n//# sourceURL={source_url}\n"));
        Ok(GeneratedProgram { source, mappings })
    }

    pub fn page_program(&self) -> Result<String> {
        Ok(self.mapped_program()?.source)
    }
}
