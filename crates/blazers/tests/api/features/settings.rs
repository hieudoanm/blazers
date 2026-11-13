use super::*;

#[tokio::test]
async fn test_settings_and_backup_run() {
    let svr = Svr::start().await;
    let admin = svr.register().await;

    // set smtp settings (pass gets encrypted)
    let r = svr
        .put_auth(
            "/api/settings",
            &admin,
            json!({"smtp_host": "smtp.example.com", "smtp_port": "587", "smtp_user": "u", "smtp_from": "noreply@example.com", "smtp_pass": "secret", "app_url": "https://app.example.com", "backup_schedule": "0 3 * * *"}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let body = r.json::<Value>().await.unwrap();
    assert_eq!(body["smtp_host"], "smtp.example.com");
    assert_eq!(body["smtp_pass"], "********");

    // backup run
    let r = svr.post_auth("/api/backup/run", &admin, json!({})).await;
    assert_eq!(r.status(), StatusCode::OK);
    let body = r.json::<Value>().await.unwrap();
    assert!(body["path"].as_str().unwrap().contains("blazers-backup-"));
}
