//! mDNS service discovery — publishes `mboacaisse.local` on the LAN.
//!
//! Uses the `mdns-sd` crate to register the HTTP service so clients can
//! reach the application via `http://mboacaisse.local:PORT` without
//! configuring an IP address.

use mdns_sd::{ServiceDaemon, ServiceInfo};
use tracing::{info, warn};

/// Start the mDNS service daemon and register the MboaCaisse HTTP service.
///
/// Publishes `mboacaisse._http._tcp.local.` on the LAN.
/// Returns the `ServiceDaemon` handle so the caller can keep it alive.
pub fn start_mdns(port: u16) -> Option<ServiceDaemon> {
	let daemon = match ServiceDaemon::new() {
		Ok(d) => d,
		Err(e) => {
			warn!("mDNS daemon could not be started: {} — service discovery disabled", e);
			return None;
		}
	};

	// Resolve the actual local IP address instead of using "0.0.0.0".
	let host_ip = resolve_local_ip();

	let service_info = match ServiceInfo::new(
		"_http._tcp.local.",
		"mboacaisse",
		"mboacaisse.local.",
		&host_ip,
		port,
		None::<std::collections::HashMap<String, String>>,
	) {
		Ok(info) => info,
		Err(e) => {
			warn!("Failed to create mDNS service info: {}", e);
			return None;
		}
	};

	match daemon.register(service_info) {
		Ok(_) => {
			info!("mDNS service published: http://mboacaisse.local:{} (IP: {})", port, host_ip);
			Some(daemon)
		}
		Err(e) => {
			warn!("Failed to register mDNS service: {} — service discovery disabled", e);
			None
		}
	}
}

/// Resolve the local IP address on the default network route.
///
/// Uses UDP connect to a dummy address to determine the interface used for
/// outbound traffic. Falls back to "0.0.0.0" if resolution fails.
fn resolve_local_ip() -> String {
	let socket = match std::net::UdpSocket::bind("0.0.0.0:0") {
		Ok(s) => s,
		Err(_) => return "0.0.0.0".to_string(),
	};
	// Connect to a public DNS address — the OS picks the best local interface.
	// This does not actually send any packets.
	if socket.connect("8.8.8.8:80").is_ok() {
		if let Ok(addr) = socket.local_addr() {
			return addr.ip().to_string();
		}
	}
	"0.0.0.0".to_string()
}
