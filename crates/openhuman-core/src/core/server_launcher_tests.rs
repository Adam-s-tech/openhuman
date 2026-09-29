use super::*;

fn ok_launcher(_: ServeRequest) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>> {
    Box::pin(async { Ok(()) })
}

fn other_launcher(_: ServeRequest) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>> {
    Box::pin(async { anyhow::bail!("second launcher must not replace the first") })
}

#[tokio::test]
async fn first_installed_launcher_wins() {
    install_server_launcher(ok_launcher);
    install_server_launcher(other_launcher);
    let launcher = installed_server_launcher().expect("a launcher is installed");
    let request = ServeRequest {
        host: None,
        port: None,
        socketio_enabled: true,
        headless_api: false,
    };
    launcher(request).await.expect("the first launcher runs");
}
