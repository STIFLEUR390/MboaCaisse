//! Print job domain — PrintJob struct only.
//!
//! AD-5: Print is a transverse service (src/print.rs).
//!       Queue async + writer ESC/POS. Called by Payment. Never blocks the order.
//!       No repository trait — print is a side-effect, not a data entity.
//!       Reporté P2.1. Ticket numérique comme fallback immédiat.

/// A print job for the thermal printer.
#[derive(Debug, Clone)]
pub struct PrintJob {
	pub id: String,
	pub order_id: String,
	pub content: String,
	pub created_at: String,
}
