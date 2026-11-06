//! PLEXUS-44: the app that lists only `plexus-rpc` and `tokio` builds, serves
//! its activation and calls it.
//!
//! That this file compiles is most of the test: the crate's extern prelude
//! holds `plexus_rpc` and `rt` (tokio, renamed) and nothing else, so every
//! name a macro emits had to resolve through the umbrella. The tests then
//! check that what was built works — in process, over a WebSocket, and
//! through the jsonrpsee server trait the macro writes — and that the
//! manifest still lists only those two crates.

use std::sync::Arc;
use std::time::Duration;

use plexus_rpc::__deps::futures::StreamExt;
use plexus_rpc::__deps::serde_json::{self, json, Value};
use plexus_rpc::client::ws::WsTransport;
use plexus_rpc::prelude::*;
use umbrella_app::{
    service, Echo, EchoActivation, EchoActivationRpcServer, EchoClient, EchoHandle, Missing, Pong, SignedIn, Thing,
    Tick,
};

/// Every form, through the activation's own trait.
async fn all_forms(c: &dyn Echo) {
    assert_eq!(c.ping().await, Pong { message: "pong".into() });
    let ticks: Vec<Tick> = c.count(3).collect().await;
    assert_eq!(ticks, vec![Tick { n: 1 }, Tick { n: 2 }, Tick { n: 3 }]);
    assert_eq!(c.lookup("a1".into()).await, Ok(Thing { id: "a1".into() }));
    assert_eq!(c.lookup("missing".into()).await, Err(Missing::NotFound { id: "missing".into() }));
}

#[rt::test(crate = "rt")]
async fn the_activation_answers_through_its_trait() {
    all_forms(&EchoActivation).await;
}

#[rt::test(crate = "rt")]
async fn its_client_calls_the_service_in_process() {
    let transport = ServiceTransport::anonymous(Arc::new(service()));
    all_forms(&EchoClient::new(transport)).await;
}

#[rt::test(crate = "rt", flavor = "multi_thread")]
async fn its_client_calls_the_service_over_a_websocket() {
    let port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let server = TransportServer::builder(Arc::new(service()), |s| {
        DynamicService::arc_into_rpc_module(s).map_err(Into::into)
    })
    .with_websocket(port)
    .without_registry()
    .build()
    .await
    .expect("the server builds");
    let serving = rt::spawn(server.serve());

    let url = format!("ws://127.0.0.1:{port}");
    let mut transport = None;
    for _ in 0..100 {
        match WsTransport::connect(&url, "umbrella").await {
            Ok(t) => {
                transport = Some(t);
                break;
            }
            Err(_) => rt::time::sleep(Duration::from_millis(50)).await,
        }
    }
    let transport = transport.expect("the service is listening");
    all_forms(&EchoClient::new(transport)).await;
    serving.abort();
}

/// The notifications of one subscription made by a raw request, up to its
/// `done` item.
async fn subscribe(request: Value) -> (Value, Vec<Value>) {
    let module = EchoActivation.into_rpc();
    let (answer, mut rx) = module.raw_json_request(&request.to_string(), 64).await.expect("a request");
    let answer: Value = serde_json::from_str(answer.get()).unwrap();
    let mut items = Vec::new();
    if answer.get("error").is_none() {
        loop {
            let next = rt::time::timeout(Duration::from_secs(5), rx.recv()).await.expect("an item in time");
            let Some(raw) = next else { break };
            let item: Value = serde_json::from_str(raw.get()).unwrap();
            let result = item["params"]["result"].clone();
            let done = result["type"] == "done";
            items.push(result);
            if done {
                break;
            }
        }
    }
    (answer, items)
}

fn ticks(items: &[Value]) -> Vec<u64> {
    items.iter().filter_map(|i| i["content"]["n"].as_u64()).collect()
}

/// The jsonrpsee server trait the macro writes (PLEXUS-44: written by
/// plexus-macros, no longer by jsonrpsee's `#[rpc]`) registers the same
/// names and decodes parameters the same ways.
#[rt::test(crate = "rt")]
async fn the_server_trait_registers_and_decodes_as_jsonrpsee_did() {
    let module = EchoActivation.into_rpc();
    let mut names: Vec<&str> = module.method_names().collect();
    names.sort();
    assert_eq!(
        names,
        vec![
            "echo_count",
            "echo_lookup",
            "echo_ping",
            "echo_unsubscribe_count",
            "echo_unsubscribe_lookup",
            "echo_unsubscribe_ping",
        ]
    );

    // By position.
    let (_, items) = subscribe(json!({"jsonrpc": "2.0", "id": 1, "method": "echo_count", "params": [2]})).await;
    assert_eq!(ticks(&items), vec![1, 2]);
    // By name.
    let (_, items) = subscribe(json!({"jsonrpc": "2.0", "id": 2, "method": "echo_count", "params": {"to": 3}})).await;
    assert_eq!(ticks(&items), vec![1, 2, 3]);
    // No parameters at all.
    let (_, items) = subscribe(json!({"jsonrpc": "2.0", "id": 3, "method": "echo_ping"})).await;
    assert!(items.iter().any(|i| i["content"]["message"] == "pong"), "{items:?}");
    // A parameter that does not decode rejects the subscription with
    // jsonrpsee's invalid-params error.
    let (answer, items) =
        subscribe(json!({"jsonrpc": "2.0", "id": 4, "method": "echo_count", "params": {"to": "three"}})).await;
    assert_eq!(answer["error"]["code"], -32602, "{answer}");
    assert!(items.is_empty());
    let (answer, _) = subscribe(json!({"jsonrpc": "2.0", "id": 5, "method": "echo_count", "params": []})).await;
    assert_eq!(answer["error"]["code"], -32602, "{answer}");
}

/// The derives' expansions resolved through the umbrella too.
#[test]
fn the_derives_work() {
    use plexus_rpc::auth_core::HasCredentialMarkers;
    let markers = <SignedIn as HasCredentialMarkers>::credential_markers();
    assert_eq!(markers.len(), 1);
    assert_eq!(markers[0].field, "session");

    let handle = EchoHandle::Said { id: "a1".into() }.to_handle();
    assert_eq!(handle.method, "said");
    assert_eq!(handle.plugin_id, EchoActivation::PLUGIN_ID);
    assert!(matches!(EchoHandle::try_from(&handle), Ok(EchoHandle::Said { id }) if id == "a1"));
    assert_eq!(EchoHandle::Ping.to_handle().method, "ping");

    let schema = EchoActivation.plugin_schema();
    assert_eq!(schema.namespace, "echo");
    for m in ["ping", "count", "lookup"] {
        assert!(schema.methods.iter().any(|s| s.name == m), "{m} in the schema");
    }
}

/// The manifest lists `plexus-rpc` and tokio and nothing else — no
/// dev-dependency or build-dependency either, since those enter the extern
/// prelude of these tests too. A crate added here to make an escaped macro
/// path resolve fails this test.
#[test]
fn the_manifest_lists_only_plexus_rpc_and_tokio() {
    let manifest = include_str!("../Cargo.toml");
    let mut section = String::new();
    let mut deps = Vec::new();
    for line in manifest.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            section = line.trim_matches(|c| c == '[' || c == ']').to_string();
            assert!(
                !section.contains("dependencies") || section == "dependencies",
                "only [dependencies]: found [{section}]"
            );
            continue;
        }
        if section == "dependencies" {
            let key = line.split('=').next().unwrap().trim().to_string();
            let package = line
                .split("package")
                .nth(1)
                .and_then(|rest| rest.split('"').nth(1))
                .map(str::to_string)
                .unwrap_or_else(|| key.clone());
            deps.push(package);
        }
    }
    deps.sort();
    assert_eq!(deps, vec!["plexus-rpc".to_string(), "tokio".to_string()]);
}
