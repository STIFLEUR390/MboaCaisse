//! MboaCaisse — Tauri application entry point.
//!
//! Initialises subsystems in order:
//!   1. Tracing subscriber
//!   2. Database pool + migrations + seed
//!   3. JWT secret generation
//!   4. Full application router (API + static files + middleware)
//!   5. Tauri plugins
//!   6. Axum server + mDNS
//!   7. Tray icon

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

use api::AppApiState;
use db::users::DbUserRepository;
use domain::user::UserRepository;

#[cfg(desktop)]
use tauri::{
	menu::{Menu, MenuItem},
	tray::TrayIcon,
	tray::TrayIconBuilder,
};

pub struct AppState {
	pub db_pool: SqlitePool,
}

pub struct AppHandles {
	#[cfg(desktop)]
	pub tray_handle: Arc<std::sync::Mutex<Option<TrayIcon>>>,
	pub mdns_daemon: Arc<std::sync::Mutex<Option<mdns_sd::ServiceDaemon>>>,
	pub shutdown_tx: watch::Sender<bool>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	tracing_subscriber::fmt()
		.with_env_filter(
			tracing_subscriber::EnvFilter::try_from_default_env()
				.unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
		)
		.init();

	info!("Starting MboaCaisse...");

	let db_path = "mboacaisse.db";
	let pool = init_pool(db_path).expect("Failed to initialise database pool");
	{
		let mut conn = pool.get().expect("Failed to acquire connection for migrations");
		migrations::run(&mut conn).expect("Database migrations failed");
		db::seed::run(&mut conn).expect("Database seed failed");
	}
	let app_state = AppState { db_pool: pool.clone() };

	// JWT secret
	let jwt_secret = load_or_generate_jwt_secret();
	info!("JWT secret initialised ({} bytes)", jwt_secret.len());

	// Build the full application router
	let user_repo: Arc<dyn UserRepository> = Arc::new(DbUserRepository::new(pool.clone()));
	let api_state = AppApiState {
		user_repo,
		jwt_secret,
	};
	let app_router = api::build_app(api_state);

	// Channels for server lifecycle
	let (shutdown_tx, shutdown_rx) = watch::channel(false);
	let (ready_tx, ready_rx) = std::sync::mpsc::channel::<()>();

	// Handles
	#[cfg(desktop)]
	let tray_handle: Arc<std::sync::Mutex<Option<TrayIcon>>> = Arc::new(std::sync::Mutex::new(None));
	let mdns_daemon: Arc<std::sync::Mutex<Option<mdns_sd::ServiceDaemon>>> = Arc::new(std::sync::Mutex::new(None));

	info!("Database initialised successfully");

	tauri::Builder::default()
		.setup(move |app| {
			app.manage(app_state);

			app.manage(AppHandles {
				#[cfg(desktop)]
				tray_handle: tray_handle.clone(),
				mdns_daemon: mdns_daemon.clone(),
				shutdown_tx: shutdown_tx.clone(),
			});

			// Start the Axum HTTP server
			let port = resolve_port();
			let srv_shutdown_rx = shutdown_rx.clone();
			let app_router_clone = app_router.clone();
			tauri::async_runtime::spawn(async move {
				server::start_server(port, app_router_clone, srv_shutdown_rx, ready_tx).await;
			});

			let _ = ready_rx.recv();
			info!("Axum server listening and ready");

			// Start mDNS
			let mdns_clone = mdns_daemon.clone();
			std::thread::spawn(move || {
				let daemon = mdns::start_mdns(port);
				if let Some(d) = daemon {
					*mdns_clone.lock().unwrap() = Some(d);
				}
			});

			// Tray icon
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
						other => warn!("Unhandled tray menu item: {}", other),
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
			}
			if let tauri::RunEvent::Exit = &event {
				info!("Exit event received — creating pre-shutdown database backup");
				let state = app_handle.state::<AppState>();
				backup_database(&state.db_pool);
			}
		});
}

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

fn load_or_generate_jwt_secret() -> Arc<Vec<u8>> {
	use domain::jwt::generate_secret;
	Arc::new(generate_secret())
}

fn backup_database(pool: &SqlitePool) {
	let src = "mboacaisse.db";
	let dst = "mboacaisse-before-shutdown.db";
	if let Ok(conn) = pool.get() {
		let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
	}
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
