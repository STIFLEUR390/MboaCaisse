//! MboaCaisse — Tauri application entry point.
//!
//! Initialises subsystems in order:
//!   1. Tracing subscriber (logging)
//!   2. Database pool + migrations
//!   3. Tauri plugins (shell, notification, os, fs, store)
//!   4. Tray icon (desktop only)
//!   5. Axum server (future story — server.rs)
//!
//! AD-9: on_event(ExitRequested) → shutdown_tx → Axum graceful → backup DB.
//!       Timeout 5s. Better to lose a backup than corrupt the DB.

// Module declarations — flat structure per AD-3.
mod api;
mod db;
mod domain;

use std::sync::Arc;

use db::{init_pool, migrations, SqlitePool};

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

	tracing::info!("Starting MboaCaisse...");

	// 2. Initialise database pool and run migrations.
	// The DB file lives next to the binary for now.
	let db_path = "mboacaisse.db";
	let pool = init_pool(db_path).expect("Failed to initialise database pool");
	{
		let mut conn = pool.get().expect("Failed to acquire connection for migrations");
		migrations::run(&mut conn).expect("Database migrations failed");
		db::seed::run(&mut conn).expect("Database seed failed");
	}
	let app_state = AppState { db_pool: pool.clone() };

	// Create tray handle before setup so it outlives the setup closure.
	// On non-desktop platforms the handle is unused.
	#[cfg(desktop)]
	let tray_handle: Arc<std::sync::Mutex<Option<TrayIcon>>> = Arc::new(std::sync::Mutex::new(None));

	tracing::info!("Database initialised successfully");

	// 3. Build Tauri application.
	tauri::Builder::default()
		.setup(|app| {
			// Store the pool handle in Tauri managed state so API handlers can access it.
			app.manage(app_state);

			#[cfg(desktop)]
			app.manage(tray_handle.clone());

			#[cfg(desktop)]
			{
				let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
				let menu = Menu::with_items(app, &[&quit_i])?;

				// Build the tray icon. Keep the TrayIcon handle alive for the app's lifetime
				// by storing it in managed state. Without this, the TrayIcon's Drop impl
				// removes the icon from the system tray as soon as setup returns.
				let tray = TrayIconBuilder::new()
					.menu(&menu)
					.show_menu_on_left_click(true)
					.icon(app.default_window_icon().unwrap().clone())
					.on_menu_event(|app_handle, event| match event.id.as_ref() {
						"quit" => {
							tracing::info!("Quit requested via tray menu");
							app_handle.exit(0);
						}
						other => {
							tracing::warn!("Unhandled tray menu item: {}", other);
						}
					})
					.build(app)?;

				*tray_handle.lock().unwrap() = Some(tray);

				tracing::info!("Tray icon created");
			}

			tracing::info!("MboaCaisse setup complete");
			Ok(())
		})
		.plugin(tauri_plugin_shell::init())
		.plugin(tauri_plugin_notification::init())
		.plugin(tauri_plugin_os::init())
		.plugin(tauri_plugin_fs::init())
		.plugin(tauri_plugin_store::Builder::new().build())
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
