mod plain;
#[cfg(feature = "direct_tls")]
mod tls;
#[cfg(unix)]
mod unix;

use std::sync::Arc;

use axum_server::Handle as ServerHandle;
use conduwuit::{Result, err};
use conduwuit_service::Services;
use tokio::sync::broadcast;

use super::layers;

/// Serve clients
pub(super) async fn serve(
	services: Arc<Services>,
	handle: ServerHandle,
	mut shutdown: broadcast::Receiver<()>,
) -> Result {
	let server = &services.server;
	let config = &server.config;
	if !config.listening {
		return shutdown
			.recv()
			.await
			.map_err(|e| err!(error!("channel error: {e}")));
	}

	let addrs = config.get_bind_addrs();
	let (app, _guard) = layers::build(&services)?;
	#[cfg(unix)]
	if config.unix_socket_path.is_some() {
		return unix::serve(server, app, shutdown).await;
	}

	if config.tls.certs.is_some() {
		#[cfg(feature = "direct_tls")]
		return tls::serve(server, app, handle, addrs).await;

		#[cfg(not(feature = "direct_tls"))]
		return conduwuit::Err!(Config(
			"tls",
			"conduwuit was not built with direct TLS support (\"direct_tls\")"
		));
	}

	plain::serve(server, app, handle, addrs).await
}
