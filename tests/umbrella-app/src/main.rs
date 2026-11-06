//! The app's `main`: serve the service on a WebSocket.
//!
//! ```bash
//! cargo run -p umbrella-app -- 4460
//! synapse -P 4460 umbrella echo count --to 3
//! ```

use std::sync::Arc;

use plexus_rpc::prelude::*;

#[rt::main(crate = "rt")]
async fn main() -> plexus_rpc::__deps::anyhow::Result<()> {
    let port: u16 = std::env::args().nth(1).map(|p| p.parse()).transpose()?.unwrap_or(4460);
    let service = Arc::new(umbrella_app::service());
    TransportServer::builder(service, |s| DynamicService::arc_into_rpc_module(s).map_err(Into::into))
        .with_websocket(port)
        .build()
        .await?
        .serve()
        .await
}
