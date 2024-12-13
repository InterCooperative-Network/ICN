


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

    // Health check route
    let health_route = warp::path("health")
        .and(warp::get())
        .map(|| "OK");

    let routes = ws_route.or(health_route);

    println!("Starting WebSocket server on localhost:8088");
    warp::serve(routes)
        .run(([127, 0, 0, 1], 8088))
        .await;
}
