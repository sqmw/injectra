use crate::{MAX_FILE_BYTES, MAX_FILES, invalid, unique_id};
use injectra_core::Result;
use std::{
    fs::File,
    io::{Read, Write},
    path::{Component, Path},
};

pub(crate) fn validate_path(path: &str) -> Result<()> {
    if path.is_empty()
        || path.len() > 512
        || path.contains('\\')
        || path.chars().any(|c| c.is_control())
        || path.contains(':')
    {
        return Err(invalid("invalid workspace relative path"));
    }
    if Path::new(path)
        .components()
        .any(|c| !matches!(c, Component::Normal(_)))
        || path.split('/').any(|p| p.is_empty())
    {
        return Err(invalid(
            "workspace path must be relative without dot components",
        ));
    }
    Ok(())
}

pub(crate) fn user_path(path: &str) -> Result<()> {
    validate_path(path)?;
    if path.split('/').any(|part| part.starts_with('.')) {
        return Err(invalid("hidden paths are not user increments"));
    }
    if !matches!(
        Path::new(path).extension().and_then(|s| s.to_str()),
        Some("js" | "cjs" | "mjs" | "css" | "html" | "json")
    ) {
        return Err(invalid(
            "workspace supports text JS/CJS/MJS/CSS/HTML/JSON only",
        ));
    }
    Ok(())
}

#[cfg(unix)]
mod platform {
    use super::*;
    use rustix::fs::{
        AtFlags, Dir, FileType, Mode, OFlags, fstat, mkdirat, open, openat, renameat, statat,
        unlinkat,
    };
    pub(crate) struct Root {
        file: File,
    }
    impl Root {
        pub fn open(path: &Path) -> Result<Self> {
            let fd = open(
                path,
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(std::io::Error::from)?;
            Ok(Self {
                file: File::from(fd),
            })
        }
        fn parent(&self, path: &str, create: bool) -> Result<(File, String)> {
            validate_path(path)?;
            let mut parts = path.split('/').collect::<Vec<_>>();
            let name = parts.pop().unwrap().to_owned();
            let mut dir = self.file.try_clone()?;
            for component in parts {
                if create {
                    match mkdirat(&dir, component, Mode::from_raw_mode(0o700)) {
                        Ok(()) => (),
                        Err(rustix::io::Errno::EXIST) => (),
                        Err(e) => return Err(std::io::Error::from(e).into()),
                    }
                }
                dir = File::from(
                    openat(
                        &dir,
                        component,
                        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                        Mode::empty(),
                    )
                    .map_err(std::io::Error::from)?,
                );
            }
            Ok((dir, name))
        }
        pub fn read(&self, path: &str) -> Result<Vec<u8>> {
            let (dir, name) = self.parent(path, false)?;
            let file = File::from(
                openat(
                    &dir,
                    name.as_str(),
                    OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
                    Mode::empty(),
                )
                .map_err(std::io::Error::from)?,
            );
            let stat = fstat(&file).map_err(std::io::Error::from)?;
            if FileType::from_raw_mode(stat.st_mode) != FileType::RegularFile {
                return Err(invalid("workspace asset is not a regular file"));
            }
            let mut bytes = Vec::new();
            file.take(MAX_FILE_BYTES as u64 + 1)
                .read_to_end(&mut bytes)?;
            if bytes.len() > MAX_FILE_BYTES {
                return Err(invalid("workspace file exceeds 1 MiB"));
            }
            std::str::from_utf8(&bytes).map_err(|_| invalid("workspace file is not UTF-8"))?;
            Ok(bytes)
        }
        pub fn write(&self, path: &str, bytes: &[u8], exclusive: bool) -> Result<()> {
            if bytes.len() > MAX_FILE_BYTES {
                return Err(invalid("workspace file exceeds 1 MiB"));
            }
            let (dir, name) = self.parent(path, true)?;
            match statat(&dir, name.as_str(), AtFlags::SYMLINK_NOFOLLOW) {
                Ok(stat) => {
                    if exclusive {
                        return Err(invalid("destination already exists"));
                    }
                    if FileType::from_raw_mode(stat.st_mode) != FileType::RegularFile {
                        return Err(invalid("refusing non-regular workspace destination"));
                    }
                }
                Err(rustix::io::Errno::NOENT) => (),
                Err(e) => return Err(std::io::Error::from(e).into()),
            }
            let temp = format!(".injectra-write-{}", unique_id()?);
            let mut file = File::from(
                openat(
                    &dir,
                    temp.as_str(),
                    OFlags::WRONLY
                        | OFlags::CREATE
                        | OFlags::EXCL
                        | OFlags::NOFOLLOW
                        | OFlags::CLOEXEC,
                    Mode::from_raw_mode(0o600),
                )
                .map_err(std::io::Error::from)?,
            );
            let result = (|| -> Result<()> {
                file.write_all(bytes)?;
                file.sync_all()?;
                renameat(&dir, temp.as_str(), &dir, name.as_str()).map_err(std::io::Error::from)?;
                dir.sync_all()?;
                Ok(())
            })();
            if result.is_err() {
                let _ = unlinkat(&dir, temp.as_str(), AtFlags::empty());
            }
            result
        }
        pub fn files(&self) -> Result<Vec<String>> {
            fn walk(
                dir: &File,
                prefix: &str,
                depth: usize,
                out: &mut Vec<String>,
                visited: &mut usize,
            ) -> Result<()> {
                if depth > 16 || *visited > 512 {
                    return Err(invalid("workspace directory limit exceeded"));
                }
                for entry in Dir::read_from(dir).map_err(std::io::Error::from)? {
                    let entry = entry.map_err(std::io::Error::from)?;
                    let name = entry
                        .file_name()
                        .to_str()
                        .map_err(|_| invalid("non-UTF8 workspace filename"))?;
                    if name.starts_with('.') {
                        continue;
                    }
                    *visited += 1;
                    let path = if prefix.is_empty() {
                        name.to_owned()
                    } else {
                        format!("{prefix}/{name}")
                    };
                    validate_path(&path)?;
                    let stat = statat(dir, name, AtFlags::SYMLINK_NOFOLLOW)
                        .map_err(std::io::Error::from)?;
                    match FileType::from_raw_mode(stat.st_mode) {
                        FileType::Directory => {
                            let child = File::from(
                                openat(
                                    dir,
                                    name,
                                    OFlags::RDONLY
                                        | OFlags::DIRECTORY
                                        | OFlags::NOFOLLOW
                                        | OFlags::CLOEXEC,
                                    Mode::empty(),
                                )
                                .map_err(std::io::Error::from)?,
                            );
                            walk(&child, &path, depth + 1, out, visited)?;
                        }
                        FileType::RegularFile => {
                            out.push(path);
                            if out.len() > MAX_FILES {
                                return Err(invalid("workspace file limit exceeded"));
                            }
                        }
                        _ => {
                            return Err(invalid(
                                "workspace symlinks and special files are unsupported",
                            ));
                        }
                    }
                }
                Ok(())
            }
            let mut out = Vec::new();
            walk(&self.file, "", 0, &mut out, &mut 0)?;
            out.sort();
            Ok(out)
        }
    }
}
#[cfg(unix)]
pub(crate) use platform::Root;
#[cfg(not(unix))]
pub(crate) struct Root;
#[cfg(not(unix))]
impl Root {
    pub fn open(_: &Path) -> Result<Self> {
        Err(injectra_core::Error::prepare(
            injectra_core::ErrorKind::Unsupported,
            "workspace filesystem currently requires Unix directory-handle operations",
        ))
    }
    pub fn read(&self, _: &str) -> Result<Vec<u8>> {
        unreachable!()
    }
    pub fn write(&self, _: &str, _: &[u8], _: bool) -> Result<()> {
        unreachable!()
    }
    pub fn files(&self) -> Result<Vec<String>> {
        unreachable!()
    }
}
