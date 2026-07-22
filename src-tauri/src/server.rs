//! Axum HTTP server — serves frontend assets and API routes.
//!
//! Started in a `tokio::spawn` during Tauri setup. Supports graceful shutdown
//! via a watch channel. On `ExitRequested`, the server stops accepting new
//! connections and drains existing ones within a 5-second grace period.
//!
//! AD-9: Server lifecycle bound to Tauri's on_event(ExitRequested).
//! AD-10: Axum 0.8, no WebSocket in V1.

use std::time::Duration;

use axum::Router;
use tokio::sync::watch;
use tower_http::{
	compression::CompressionLayer,
	cors::CorsLayer,
};
use tracing::{info, warn};

/// Start the Axum HTTP server on the given port.
///
/// Takes a pre-built full application router with all state injected.
/// The router must use `Router<()>` or be convertible via `into_make_service()`.
pub async fn start_server(
	port: u16,
	app: Router,
	shutdown_rx: watch::Receiver<bool>,
	ready_tx: std::sync::mpsc::Sender<()>,
) {
	// Bind to the port (or scan a range).
	let listener = match bind_with_fallback(port).await {
		Some(l) => l,
		None => {
			warn!("Could not bind to any port in range {}-{} — server not started", port, port + 5);
			return;
		}
	};
	let actual_port = listener.local_addr().unwrap().port();

	// Signal that the server is ready.
	let _ = ready_tx.send(());

	info!("Axum server listening on http://0.0.0.0:{}", actual_port);

	// Run with graceful shutdown.
	let serve_future = axum::serve(listener, app.into_make_service())
		.with_graceful_shutdown(async move {
			let mut rx = shutdown_rx;
			loop {
				rx.changed().await.ok();
				if *rx.borrow() {
					info!("Graceful shutdown initiated — draining in-flight requests");
					break;
				}
			}
		});

	match tokio::time::timeout(Duration::from_secs(5), serve_future).await {
		Ok(Ok(())) => info!("Axum server stopped cleanly"),
		Ok(Err(e)) => warn!("Axum server exited with error: {}", e),
		Err(_) => warn!("Axum server shutdown timed out after 5s — forcing stop"),
	}
}

/// Try to bind on `base_port`, then scan up to `base_port + 5`.
async fn bind_with_fallback(base_port: u16) -> Option<tokio::net::TcpListener> {
	for port in base_port..=base_port.saturating_add(5) {
		let addr = format!("0.0.0.0:{}", port);
		match tokio::net::TcpListener::bind(&addr).await {
			Ok(l) => return Some(l),
			Err(_) if port < base_port + 5 => {
				warn!("Port {} in use, trying next...", port);
				continue;
			}
			Err(e) => {
				warn!("Failed to bind on {}: {}", addr, e);
				continue;
			}
		}
	}
	None
}
