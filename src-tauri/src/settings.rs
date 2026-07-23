//! Configuration store — typed access to tauri_plugin_store settings.
//!
//! AD-12: All system configuration goes through tauri_plugin_store.
//!         Config keys: port, hostname, backup_interval_hours, headless, wallet_negative.
//!         No YAML/TOML files. Values persisted in `settings.json`.

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use tracing::info;

/// The set of keys that `Config` owns and manages.
const KNOWN_CONFIG_KEYS: &[&str] = &[
	"port",
	"hostname",
	"backup_interval_hours",
	"headless",
	"wallet_negative",
];

/// Typed representation of the persistent configuration.
///
/// Each field has a default that is used when the key is absent from the store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
	/// HTTP server port (3000–3099). Default: 3000.
	pub port: u16,
	/// mDNS hostname (e.g. "mboacaisse" → mboacaisse.local). Default: "mboacaisse".
	pub hostname: String,
	/// Interval between automatic database backups, in hours. Default: 24.
	pub backup_interval_hours: u64,
	/// When true, no window is shown on startup; the app runs in the tray. Default: false.
	pub headless: bool,
	/// When true, wallet payments are accepted even if the balance would go negative.
	/// Default: false (payments refused if balance < total).
	pub wallet_negative: bool,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			port: 3000,
			hostname: "mboacaisse".to_string(),
			backup_interval_hours: 24,
			headless: false,
			wallet_negative: false,
		}
	}
}

impl Config {
	/// Load configuration from the Tauri store.
	///
	/// Opens (or creates) `settings.json` in the app data directory,
	/// reads each known key, and falls back to `Default` for any missing key.
	pub fn load(app: &AppHandle) -> Self {
		let store = match app.store("settings.json") {
			Ok(s) => s,
			Err(e) => {
				tracing::warn!("Failed to open config store: {} — using defaults", e);
				return Self::default();
			}
		};

		let port: u16 = store
			.get("port")
			.and_then(|v| v.as_u64())
			.map(|v| v as u16)
			.filter(|p| (3000..=3099).contains(p))
			.unwrap_or(3000);

		let hostname: String = store
			.get("hostname")
			.and_then(|v| v.as_str().map(String::from))
			.filter(|s| !s.is_empty())
			.unwrap_or_else(|| "mboacaisse".to_string());

		let backup_interval_hours: u64 = store
			.get("backup_interval_hours")
			.and_then(|v| v.as_u64())
			.filter(|&v| v >= 1 && v <= 168)
			.unwrap_or(24);

		let headless: bool = store
			.get("headless")
			.and_then(|v| v.as_bool())
			.unwrap_or(false);

		let wallet_negative: bool = store
			.get("wallet_negative")
			.and_then(|v| v.as_bool())
			.unwrap_or(false);

		let cfg = Self {
			port,
			hostname,
			backup_interval_hours,
			headless,
			wallet_negative,
		};

		info!(
			"Config loaded — port: {}, hostname: {}, backup_interval: {}h, headless: {}, wallet_negative: {}",
			cfg.port, cfg.hostname, cfg.backup_interval_hours, cfg.headless, cfg.wallet_negative
		);
		cfg
	}

	/// Persist a single key-value pair to the Tauri store.
	///
	/// Returns `Ok(true)` if the value changed, `Ok(false)` if it was already the same,
	/// or `Err` if the store could not be saved.
	pub fn set(app: &AppHandle, key: &str, value: serde_json::Value) -> Result<bool, String> {
		let store = app.store("settings.json").map_err(|e| e.to_string())?;

		let changed = match store.get(key) {
			Some(existing) if existing == value => false,
			_ => true,
		};

		if changed {
			store.set(key.to_string(), value);
			store.save().map_err(|e| format!("Failed to save settings.json: {}", e))?;
			info!("Config updated: {} = {:?}", key, store.get(key));
		}

		Ok(changed)
	}

	/// Return `true` if the given key requires a restart to take effect.
	pub fn requires_restart(key: &str) -> bool {
		matches!(key, "port" | "hostname" | "headless")
	}

	/// Clear all known settings from the store, reverting to defaults.
	/// Only deletes keys that `Config` knows about; unknown keys are preserved.
	pub fn reset(app: &AppHandle) -> Result<(), String> {
		let store = app.store("settings.json").map_err(|e| e.to_string())?;
		for key in KNOWN_CONFIG_KEYS {
			store.delete(key);
		}
		store.save().map_err(|e| format!("Failed to save settings.json after reset: {}", e))?;
		info!("Config reset to defaults ({} keys cleared)", KNOWN_CONFIG_KEYS.len());
		Ok(())
	}
}
