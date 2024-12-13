use std::sync::{Arc, Mutex};
use warp::Filter;
use crate::ICNCore;
use crate::websocket::WebSocketHandler;

#[tokio::main]
async fn main() {
    // Initialize ICNCore system
    let icn_core = Arc::new(ICNCore::new());

    // Create WebSocket handler for real-time updates
    let ws_handler = Arc::new(WebSocketHandler::new());

    // Define WebSocket route with DID header for user identification
    let ws_handler = ws_handler.clone();
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::header::<String>("X-DID"))
        .and(warp::any().map(move || ws_handler.clone()))
        .map(|ws: warp::ws::Ws, did: String, handler: Arc<WebSocketHandler>| {
            ws.on_upgrade(move |socket| async move {
                handler.handle_connection(socket, did).await;
            })
        });

    println!("Starting WebSocket server on localhost:8088");
    warp::serve(ws_route)
        .run(([127, 0, 0, 1], 8088))
        .await;
}
