use super::*;

static JWT_ENV_LOCK: std::sync::LazyLock<std::sync::Mutex<()>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(()));

#[test]
fn jwt_secret_returns_default_when_env_var_not_set() {
    let _lock = JWT_ENV_LOCK.lock().unwrap();
    let saved = std::env::var("JWT_SECRET");
    unsafe { std::env::remove_var("JWT_SECRET") };
    assert_eq!(jwt_secret(), "dev-secret-change-in-production");
    if let Ok(v) = saved {
        unsafe { std::env::set_var("JWT_SECRET", v) };
    }
}

#[test]
fn jwt_secret_returns_env_var_when_set() {
    let _lock = JWT_ENV_LOCK.lock().unwrap();
    let saved = std::env::var("JWT_SECRET");
    unsafe { std::env::set_var("JWT_SECRET", "test-secret-123") };
    assert_eq!(jwt_secret(), "test-secret-123");
    match saved {
        Ok(v) => unsafe { std::env::set_var("JWT_SECRET", v) },
        Err(_) => unsafe { std::env::remove_var("JWT_SECRET") },
    }
}

#[test]
fn round_trip_generate_and_validate() {
    let _lock = JWT_ENV_LOCK.lock().unwrap();
    let saved = std::env::var("JWT_SECRET");
    unsafe { std::env::set_var("JWT_SECRET", "dev-secret-change-in-production") };
    let token = generate_token("user-1", "test@example.com").unwrap();
    let claims = validate_token(&token).unwrap();
    match saved {
        Ok(v) => unsafe { std::env::set_var("JWT_SECRET", v) },
        Err(_) => unsafe { std::env::remove_var("JWT_SECRET") },
    }
    assert_eq!(claims.user_id, "user-1");
    assert_eq!(claims.email, "test@example.com");
}

#[test]
fn invalid_token_format_returns_unauthorized() {
    let result = validate_token("not-a-jwt-token");
    match result.unwrap_err() {
        AppError::Unauthorized(_) => {}
        _ => panic!("expected Unauthorized"),
    }
}

#[test]
fn token_with_wrong_secret_returns_error() {
    let _lock = JWT_ENV_LOCK.lock().unwrap();
    let saved = std::env::var("JWT_SECRET");
    unsafe { std::env::set_var("JWT_SECRET", "secret-a") };
    let token = generate_token("user-1", "test@example.com").unwrap();
    unsafe { std::env::set_var("JWT_SECRET", "secret-b") };
    let result = validate_token(&token);
    match saved {
        Ok(v) => unsafe { std::env::set_var("JWT_SECRET", v) },
        Err(_) => unsafe { std::env::remove_var("JWT_SECRET") },
    }
    assert!(result.is_err());
}

#[test]
fn generate_token_with_empty_fields() {
    let _lock = JWT_ENV_LOCK.lock().unwrap();
    let saved = std::env::var("JWT_SECRET");
    unsafe { std::env::set_var("JWT_SECRET", "dev-secret-change-in-production") };
    let token = generate_token("", "").unwrap();
    let claims = validate_token(&token).unwrap();
    match saved {
        Ok(v) => unsafe { std::env::set_var("JWT_SECRET", v) },
        Err(_) => unsafe { std::env::remove_var("JWT_SECRET") },
    }
    assert_eq!(claims.user_id, "");
    assert_eq!(claims.email, "");
}
