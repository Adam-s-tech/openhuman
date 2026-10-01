use super::*;

fn sample_snapshot() -> ConfigSnapshotFields {
    ConfigSnapshotFields {
        config: json!({
            "api_url": "https://api.example.com",
            "default_model": "gpt-4",
            "default_temperature": 0.7,
            "memory": {"enabled": true, "limit": 1000},
            "runtime": {"debug": false, "workers": 4},
            "browser": {"allow_all": false},
        }),
        workspace_dir: "/tmp/ws".into(),
        config_path: "/tmp/config.toml".into(),
    }
}
