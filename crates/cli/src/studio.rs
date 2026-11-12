use injectra_core::{Error, ErrorKind, Result};
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

fn failure(error: impl std::fmt::Display) -> Error {
    Error::prepare(ErrorKind::Io, error.to_string())
}
pub fn request(workspace: &Path, cache: Option<PathBuf>, path: &str, body: Value) -> Result<Value> {
    injectra_studio::server::client(
        workspace,
        &cache.unwrap_or_else(injectra_studio::endpoint::default_cache),
        path,
        body,
    )
    .map_err(failure)
}
pub fn serve(
    workspace: PathBuf,
    cache: Option<PathBuf>,
    assets: Option<PathBuf>,
) -> std::process::ExitCode {
    let result = (|| -> Result<()> {
        let cache = cache.unwrap_or_else(injectra_studio::endpoint::default_cache);
        let explicit_assets =
            assets.is_some() || std::env::var_os("INJECTRA_STUDIO_ASSETS").is_some();
        let assets = assets
            .or_else(|| std::env::var_os("INJECTRA_STUDIO_ASSETS").map(PathBuf::from))
            .unwrap_or_else(|| cache.join("studio-ui/dist"));
        if explicit_assets && !assets.is_dir() {
            return Err(failure("Configured Studio asset directory does not exist"));
        }
        let assets = assets.is_dir().then_some(assets);
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(failure)?;
        runtime.block_on(async move {
            let service = injectra_studio::Studio::new(
                injectra_studio::StudioOptions {
                    workspace,
                    cache,
                    assets,
                },
                None,
                Arc::new(injectra_bootstrap::Broker::new()),
            )
            .map_err(failure)?;
            println!("{}", service.ready());
            service
                .serve(async {
                    let _ = tokio::signal::ctrl_c().await;
                })
                .await
                .map_err(failure)
        })
    })();
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("injectra: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
