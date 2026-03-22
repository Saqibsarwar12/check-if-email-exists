pub async fn run_warp_server(
    config: Arc<BackendConfig>,
) -> Result<Option<JobRunnerHandle>, anyhow::Error> {
    let host = "0.0.0.0"
        .parse::<IpAddr>()
        .expect("Invalid bind host");

    let port = env::var("PORT")
        .map(|port: String| {
            port.parse::<u16>()
                .unwrap_or_else(|_| panic!("Invalid port: {}", port))
        })
        .unwrap_or(config.http_port);

    let routes = create_routes(Arc::clone(&config));

    let is_bulk_enabled = env::var("RCH_ENABLE_BULK")
        .unwrap_or_else(|_| "0".into())
        == "1";

    let runner = if is_bulk_enabled {
        let pg_pool = config.get_pg_pool().expect(
            "Please set the RCH__STORAGE__POSTGRES__DB_URL environment when RCH_ENABLE_BULK is set",
        );
        let runner = v0::bulk::create_job_registry(&pg_pool).await?;
        Some(runner)
    } else {
        None
    };

    info!(target: LOG_TARGET, host = ?host, port = ?port, "Server is listening");

    warp::serve(routes).run((host, port)).await;

    Ok(runner)
}
