//! MboaCaisse — Tauri application entry point.
//!
//! Initialises subsystems in order:
//!   1. Tracing subscriber (logging)
//!   2. Database pool + migrations
//!   3. Tauri plugins (shell, notification, os, fs, store)
//!   4. Axum server + mDNS (tokio::spawn)
//!   5. Tray icon (desktop only)
//!
//! AD-9: ExitRequested → shutdown_tx → Axum graceful shutdown (5s timeout) → backup BDD.

// Module declarations — flat structure per AD-3.
mod api;
mod db;
mod domain;
mod mdns;
mod server;

use std::sync::Arc;
use std::time::Duration;

use db::{init_pool, migrations, SqlitePool};
use tokio::sync::watch;
use tracing::{info, warn};

use tauri::Manager;

#[cfg(desktop)]
use tauri::{
	menu::{Menu, MenuItem},
	tray::TrayIcon,
	tray::TrayIconBuilder,
};

/// Shared application state accessible via Tauri's managed state.
pub struct AppState {
	pub db_pool: SqlitePool,
}

/// Application-wide handles that must outlive `setup()`.
pub struct AppHandles {
	#[cfg(desktop)]
	pub tray_handle: Arc<std::sync::Mutex<Option<TrayIcon>>>,
	pub mdns_daemon: Arc<std::sync::Mutex<Option<mdns_sd::ServiceDaemon>>>,
	pub shutdown_tx: watch::Sender<bool>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	// 1. Initialise tracing subscriber.
	// AD-18: tracing + tracing-subscriber. INFO level by default.
	tracing_subscriber::fmt()
		.with_env_filter(
			tracing_subscriber::EnvFilter::try_from_default_env()
				.unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
		)
		.init();

	info!("Starting MboaCaisse...");

	// 2. Initialise database pool and run migrations.
	let db_path = "mboacaisse.db";
	let pool = init_pool(db_path).expect("Failed to initialise database pool");
	{
		let mut conn = pool.get().expect("Failed to acquire connection for migrations");
		migrations::run(&mut conn).expect("Database migrations failed");
		db::seed::run(&mut conn).expect("Database seed failed");
	}
	let app_state = AppState { db_pool: pool.clone() };

	// 3. Create channels for server lifecycle.
	let (shutdown_tx, shutdown_rx) = watch::channel(false);
	let (ready_tx, ready_rx) = std::sync::mpsc::channel::<()>();

	// 4. Create handles for long-lived subsystems.
	#[cfg(desktop)]
	let tray_handle: Arc<std::sync::Mutex<Option<TrayIcon>>> = Arc::new(std::sync::Mutex::new(None));
	let mdns_daemon: Arc<std::sync::Mutex<Option<mdns_sd::ServiceDaemon>>> = Arc::new(std::sync::Mutex::new(None));

	info!("Database initialised successfully");

	// 5. Build and run the Tauri application.
	tauri::Builder::default()
		.setup(move |app| {
			// Store the pool handle in managed state.
			app.manage(app_state);

			// Store shared handles.
			app.manage(AppHandles {
				#[cfg(desktop)]
				tray_handle: tray_handle.clone(),
				mdns_daemon: mdns_daemon.clone(),
				shutdown_tx: shutdown_tx.clone(),
			});

			// 5a. Start the Axum HTTP server.
			let port = resolve_port();
			let srv_shutdown_rx = shutdown_rx.clone();
			tauri::async_runtime::spawn(async move {
				server::start_server(port, srv_shutdown_rx, ready_tx).await;
			});

			// Wait for the server to confirm it's listening before returning.
			// This prevents a blank window if binding is slow (fixes race condition).
			let _ = ready_rx.recv();

			info!("Axum server listening and ready");

			// 5b. Start mDNS service discovery (best-effort).
			let mdns_clone = mdns_daemon.clone();
			std::thread::spawn(move || {
				let daemon = mdns::start_mdns(port);
				if let Some(d) = daemon {
					*mdns_clone.lock().unwrap() = Some(d);
				}
			});

			// 5c. Tray icon (desktop only).
			#[cfg(desktop)]
			{
				let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
				let menu = Menu::with_items(app, &[&quit_i])?;

				let tray = TrayIconBuilder::new()
					.menu(&menu)
					.show_menu_on_left_click(true)
					.icon(app.default_window_icon().unwrap().clone())
					.on_menu_event(|app_handle, event| match event.id.as_ref() {
						"quit" => {
							info!("Quit requested via tray menu");
							app_handle.exit(0);
						}
						other => {
							warn!("Unhandled tray menu item: {}", other);
						}
					})
					.build(app)?;

				*tray_handle.lock().unwrap() = Some(tray);
				info!("Tray icon created");
			}

			info!("MboaCaisse setup complete");
			Ok(())
		})
		.plugin(tauri_plugin_shell::init())
		.plugin(tauri_plugin_notification::init())
		.plugin(tauri_plugin_os::init())
		.plugin(tauri_plugin_fs::init())
		.plugin(tauri_plugin_store::Builder::new().build())
		.build(tauri::generate_context!())
		.expect("error while building tauri application")
		.run(|app_handle, event| {
			if let tauri::RunEvent::ExitRequested { .. } = &event {
				info!("ExitRequested received — initiating graceful shutdown");
				let handles = app_handle.state::<AppHandles>();
				let _ = handles.shutdown_tx.send(true);
				// No sleep needed — the server drains in background.
				// The Exit event fires after the event loop continues.
			}

			if let tauri::RunEvent::Exit = &event {
				info!("Exit event received — creating pre-shutdown database backup");
				let state = app_handle.state::<AppState>();
				backup_database(&state.db_pool);
			}
		});
}

/// Resolve the HTTP port for the Axum server.
///
/// Priority:
/// 1. `TAURI_DEV_PORT` env var (set by scripts/tauri-dev.ts)
/// 2. Default: 3000
///
/// The server itself will scan a range if the port is busy.
fn resolve_port() -> u16 {
	if let Ok(port_str) = std::env::var("TAURI_DEV_PORT") {
		if let Ok(port) = port_str.parse::<u16>() {
			if (3000..=3099).contains(&port) {
				return port;
			}
		}
	}
	3000
}

/// Create a pre-shutdown backup of the SQLite database.
///
/// Performs a WAL checkpoint first to ensure consistency, then copies
/// the database file. Uses a 5-second timeout to avoid blocking exit.
fn backup_database(pool: &SqlitePool) {
	let src = "mboacaisse.db";
	let dst = "mboacaisse-before-shutdown.db";

	// Step 1: Checkpoint WAL to ensure the main file is consistent.
	if let Ok(conn) = pool.get() {
		let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
		// Connection drops here, releasing the lock.
	}

	// Step 2: Copy the database file with a timeout.
	let (tx, rx) = std::sync::mpsc::channel();
	std::thread::spawn(move || {
		let result = std::fs::copy(src, dst)
			.map(|size| (dst.to_string(), size))
			.map_err(|e| e.to_string());
		let _ = tx.send(result);
	});

	match rx.recv_timeout(Duration::from_secs(5)) {
		Ok(Ok((path, size))) => info!("Pre-shutdown backup created: {} ({} bytes)", path, size),
		Ok(Err(e)) => warn!("Pre-shutdown backup failed: {} — continuing shutdown", e),
		Err(_) => warn!("Pre-shutdown backup timed out after 5s — continuing shutdown"),
	}
}
