// Test-owned input files, created outside the repository and removed on drop.
use std::{fs, path::PathBuf, sync::atomic::{AtomicU64, Ordering}, time::{SystemTime, UNIX_EPOCH}};
static NEXT: AtomicU64 = AtomicU64::new(1);
pub struct BundleFixture { pub manifest: PathBuf, root: PathBuf }
impl BundleFixture {
    pub fn new() -> Self {
        let nonce=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let root=std::env::temp_dir().join(format!("injectra-contract-{}-{nonce}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
        fs::create_dir(&root).unwrap();fs::create_dir(root.join("input")).unwrap();
        let manifest=root.join("input/manifest.json");
        fs::write(&manifest,r#"{"version":1,"id":"injectra.test.contract","css":"style.css","html":"content.html","js":"script.js"}"#).unwrap();
        fs::write(root.join("input/style.css"),"[data-injectra] { color: rgb(1, 2, 3); }").unwrap();
        fs::write(root.join("input/content.html"),"<span>Contract fixture</span>").unwrap();
        fs::write(root.join("input/script.js"),"context.root.dataset.executed = '1';").unwrap();
        fs::write(root.join("outside.css"),"/* escape sentinel */").unwrap();
        Self{manifest,root}
    }
}
impl Drop for BundleFixture { fn drop(&mut self){ let _=fs::remove_dir_all(&self.root); } }
