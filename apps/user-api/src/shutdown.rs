use std::time::Duration;
use tokio::signal;

pub async fn shutdown_signal(timeout: Duration) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("received Ctrl+C, initiating graceful shutdown");
        }
        _ = terminate => {
            tracing::info!("received SIGTERM, initiating graceful shutdown");
        }
    }

    tracing::info!(
        timeout_secs = timeout.as_secs(),
        "allowing connections to drain"
    );
}
