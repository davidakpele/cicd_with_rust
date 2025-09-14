use warp::Filter;
use std::path::PathBuf;
use tokio::sync::oneshot;

pub struct ReactServer {
    pub(crate) port: u16,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl ReactServer {
    pub async fn serve_static_files(build_dir: PathBuf, port: u16) -> Result<Self, anyhow::Error> {
        // Serve files from dist/build
        let files = warp::fs::dir(build_dir.clone());

        // Fallback: serve index.html for SPA routing
        let index = warp::path::end().map(move || {
            warp::reply::html(
                std::fs::read_to_string(build_dir.join("index.html"))
                    .unwrap_or_else(|_| "index.html not found".into()),
            )
        });

        let routes = files.or(index);

        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        // ✅ warp::serve is a function, not a trait import
        let (_, server) = warp::serve(routes)
            .bind_with_graceful_shutdown(([0, 0, 0, 0], port), async {
                shutdown_rx.await.ok();
            });

        tokio::spawn(server);

        Ok(Self {
            port,
            shutdown_tx: Some(shutdown_tx),
        })
    }

    pub fn get_url(&self, site_name: &str) -> String {
        format!("http://{}.localhost:{}", site_name, self.port)
    }

    pub async fn shutdown(self) {
        if let Some(tx) = self.shutdown_tx {
            let _ = tx.send(());
        }
    }
}