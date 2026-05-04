//! WebSocket surface for the dashboard (phase29).
//!
//! A single multiplexed endpoint at `GET /ws/dashboard` replaces the
//! eight polling loops the React dashboard previously fired off (1–30 s
//! intervals on `/metrics/runtime`, `/stats`, `/status`, `/collections`,
//! `/logs`, etc.). REST endpoints stay live as a fallback for SDK
//! callers.

pub mod dashboard;

pub use dashboard::dashboard_ws_handler;
