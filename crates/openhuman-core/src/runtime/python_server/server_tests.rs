use super::*;

#[tokio::test]
async fn prepare_launch_rejects_disabled_backends() {
    let mut config = Config::default();
    config.runtime_python.enabled = false;
    let err = prepare_launch(&config).await.unwrap_err().to_string();
    assert!(err.contains("no runtime python server backends enabled"));
}
