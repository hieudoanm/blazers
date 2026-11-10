use super::*;

#[test]
fn event_constants_are_correct() {
    assert_eq!(EVENT_CRONJOB_CREATE, "cronjob.create");
    assert_eq!(EVENT_CRONJOB_UPDATE, "cronjob.update");
    assert_eq!(EVENT_CRONJOB_DELETE, "cronjob.delete");
}

#[test]
fn contains_cronjob_key() {
    let job = sample_cronjob();
    let data = webhook_cronjob_data(&job);
    assert!(data.get("cronjob").is_some());
}

#[test]
fn preserves_name() {
    let job = sample_cronjob();
    let data = webhook_cronjob_data(&job);
    assert_eq!(data["cronjob"]["name"], "Test Job");
}

#[test]
fn preserves_schedule() {
    let job = sample_cronjob();
    let data = webhook_cronjob_data(&job);
    assert_eq!(data["cronjob"]["schedule"], "*/5 * * * * *");
}

#[test]
fn preserves_command() {
    let job = sample_cronjob();
    let data = webhook_cronjob_data(&job);
    assert_eq!(data["cronjob"]["command"], "https://example.com/hook");
}

#[test]
fn preserves_is_active() {
    let job = sample_cronjob();
    let data = webhook_cronjob_data(&job);
    assert_eq!(data["cronjob"]["is_active"], true);
}

#[test]
fn preserves_method() {
    let job = sample_cronjob();
    let data = webhook_cronjob_data(&job);
    assert_eq!(data["cronjob"]["method"], "POST");
}

#[test]
fn webhook_cronjob_data_empty_values() {
    let job = CronJob {
        id: String::new(),
        name: String::new(),
        schedule: String::new(),
        command: String::new(),
        method: String::new(),
        headers: String::new(),
        body: String::new(),
        is_active: false,
        last_run_at: String::new(),
        last_run_status: String::new(),
        created_at: String::new(),
        updated_at: String::new(),
    };
    let data = webhook_cronjob_data(&job);
    let cronjob = &data["cronjob"];
    assert_eq!(cronjob["is_active"], false);
    assert_eq!(cronjob["name"], "");
    assert_eq!(cronjob["id"], "");
    assert_eq!(cronjob["schedule"], "");
    assert_eq!(cronjob["command"], "");
    assert_eq!(cronjob["method"], "");
}

#[test]
fn webhook_cronjob_data_inactive_job() {
    let job = CronJob {
        is_active: false,
        ..sample_cronjob()
    };
    let data = webhook_cronjob_data(&job);
    assert_eq!(data["cronjob"]["is_active"], false);
    assert_eq!(data["cronjob"]["name"], "Test Job");
}

#[test]
fn webhook_cronjob_data_special_characters() {
    let job = CronJob {
        name: "Job with spéçïal & ch@rs!".into(),
        command: "https://example.com/hook?q=test&lang=en".into(),
        headers: r#"[["X-Custom", "val=1; type=test"]]"#.into(),
        body: r#"{"message": "hello \"world\""}"#.into(),
        ..sample_cronjob()
    };
    let data = webhook_cronjob_data(&job);
    assert_eq!(data["cronjob"]["name"], "Job with spéçïal & ch@rs!");
    assert_eq!(
        data["cronjob"]["command"],
        "https://example.com/hook?q=test&lang=en"
    );
    assert_eq!(
        data["cronjob"]["headers"],
        r#"[["X-Custom", "val=1; type=test"]]"#
    );
    assert_eq!(data["cronjob"]["body"], r#"{"message": "hello \"world\""}"#);
}

#[test]
fn webhook_cronjob_data_all_fields_present() {
    let job = sample_cronjob();
    let data = webhook_cronjob_data(&job);
    let cronjob = &data["cronjob"];
    assert_eq!(cronjob["id"], "job-123");
    assert_eq!(cronjob["name"], "Test Job");
    assert_eq!(cronjob["schedule"], "*/5 * * * * *");
    assert_eq!(cronjob["command"], "https://example.com/hook");
    assert_eq!(cronjob["method"], "POST");
    assert_eq!(cronjob["is_active"], true);
    assert_eq!(cronjob["last_run_at"], "");
    assert_eq!(cronjob["last_run_status"], "");
    assert_eq!(cronjob["created_at"], "2025-01-01T00:00:00Z");
    assert_eq!(cronjob["updated_at"], "2025-01-01T00:00:00Z");
}

#[test]
fn webhook_cronjob_data_headers_and_body() {
    let job = CronJob {
        headers: r#"[["X-API-Key", "secret123"]]"#.into(),
        body: r#"{"data": "test"}"#.into(),
        ..sample_cronjob()
    };
    let data = webhook_cronjob_data(&job);
    assert_eq!(
        data["cronjob"]["headers"],
        r#"[["X-API-Key", "secret123"]]"#
    );
    assert_eq!(data["cronjob"]["body"], r#"{"data": "test"}"#);
}
