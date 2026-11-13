pub fn paths() -> serde_json::Map<String, serde_json::Value> {
    serde_json::json!({
            "/api/auth/register": {
                "post": {
                    "summary": "Register a new user",
                    "tags": ["Auth"],
                    "security": [],
                    "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {
                        "email": {"type": "string", "format": "email"},
                        "password": {"type": "string", "minLength": 6}
                    }, "required": ["email", "password"]}}}},
                    "responses": {
                        "200": {"description": "User registered"},
                        "400": {"description": "Bad request"}
                    }
                }
            },
            "/api/auth/me": {
                "get": {
                    "summary": "Get current user profile",
                    "tags": ["Auth"],
                    "responses": {"200": {"description": "User profile with custom data and verified flag"}, "401": {"description": "Unauthorized"}}
                },
                "patch": {
                    "summary": "Update custom data fields and/or password",
                    "tags": ["Auth"],
                    "requestBody": {"content": {"application/json": {"schema": {"type": "object", "properties": {
                        "data": {"type": "object"},
                        "password": {"type": "string"},
                        "current_password": {"type": "string"}
                    }}}}},
                    "responses": {"200": {"description": "Updated user"}, "401": {"description": "Wrong current password"}}
                }
            },
            "/api/auth/forgot-password": {
                "post": {
                    "summary": "Request a password reset email",
                    "tags": ["Auth"],
                    "security": [],
                    "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {"email": {"type": "string"}}, "required": ["email"]}}}},
                    "responses": {"200": {"description": "Reset email sent (dev_token included when SMTP is not configured)"}}
                }
            },
            "/api/auth/reset-password": {
                "post": {
                    "summary": "Reset password with a reset token",
                    "tags": ["Auth"],
                    "security": [],
                    "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {
                        "token": {"type": "string"},
                        "password": {"type": "string", "minLength": 6}
                    }, "required": ["token", "password"]}}}},
                    "responses": {"200": {"description": "Password updated"}, "400": {"description": "Invalid or expired token"}}
                }
            },
            "/api/auth/verify-email": {
                "post": {
                    "summary": "Request an email verification link",
                    "tags": ["Auth"],
                    "responses": {"200": {"description": "Verification email sent (dev_token included when SMTP is not configured)"}}
                }
            },
            "/api/auth/verify-email/confirm": {
                "post": {
                    "summary": "Confirm email with a verification token",
                    "tags": ["Auth"],
                    "security": [],
                    "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {"token": {"type": "string"}}, "required": ["token"]}}}},
                    "responses": {"200": {"description": "Email verified"}, "400": {"description": "Invalid or expired token"}}
                }
            },
            "/api/auth/users": {
                "get": {
                    "summary": "List users (managed auth collection; admins see all, others filtered by list_rule/self)",
                    "tags": ["Auth"],
                    "parameters": [
                        {"name": "page", "in": "query", "schema": {"type": "integer"}},
                        {"name": "per_page", "in": "query", "schema": {"type": "integer"}},
                        {"name": "filter", "in": "query", "schema": {"type": "string"}},
                        {"name": "sort", "in": "query", "schema": {"type": "string"}}
                    ],
                    "responses": {"200": {"description": "Paginated users"}, "401": {"description": "Unauthorized"}}
                }
            },
            "/api/auth/users/rules": {
                "get": {
                    "summary": "Get users-collection rule expressions (admin)",
                    "tags": ["Auth"],
                    "responses": {"200": {"description": "{list_rule, view_rule, create_rule, update_rule, delete_rule}"}}
                },
                "patch": {
                    "summary": "Set users-collection rule expressions (admin)",
                    "tags": ["Auth"],
                    "requestBody": {"content": {"application/json": {"schema": {"type": "object", "properties": {
                        "list_rule": {"type": "string"}, "view_rule": {"type": "string"},
                        "create_rule": {"type": "string"}, "update_rule": {"type": "string"},
                        "delete_rule": {"type": "string"}
                    }}}}},
                    "responses": {"200": {"description": "Updated rules"}, "400": {"description": "Invalid rule expression"}}
                }
            },
            "/api/auth/users/{id}": {
                "get": {
                    "summary": "Get a user (self/admin, or view_rule match)",
                    "tags": ["Auth"],
                    "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}],
                    "responses": {"200": {"description": "User profile"}, "403": {"description": "Rule denied"}, "404": {"description": "Not found"}}
                }
            },
            "/api/batch": {
                "post": {
                    "summary": "Batch requests — array of {method, url, body} dispatched through the API, returns [{status, body}]. Also available as ?batch=1 on any /api/* request with a JSON array body",
                    "tags": ["System"],
                    "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {
                        "requests": {"type": "array", "items": {"type": "object", "properties": {
                            "method": {"type": "string"}, "url": {"type": "string"}, "body": {}
                        }}}
                    }, "required": ["requests"]}}}},
                    "responses": {"200": {"description": "Array of {status, body} in request order"}}
                }
            },
            "/api/auth/request-otp": {
                "post": {
                    "summary": "Request a one-time sign-in code (emailed; dev_otp returned when SMTP is not configured)",
                    "tags": ["Auth"],
                    "security": [],
                    "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {"email": {"type": "string"}}, "required": ["email"]}}}},
                    "responses": {"200": {"description": "Code sent"}}
                }
            },
            "/api/auth/auth-with-otp": {
                "post": {
                    "summary": "Exchange email + OTP code for a JWT (passwordless login)",
                    "tags": ["Auth"],
                    "security": [],
                    "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {
                        "email": {"type": "string"}, "otp": {"type": "string"}
                    }, "required": ["email", "otp"]}}}},
                    "responses": {"200": {"description": "Login success, returns token"}, "401": {"description": "Invalid or expired code"}}
                }
            },
            "/api/auth/oauth/{provider}": {
                "get": {
                    "summary": "Start OAuth2 sign-in — 302 redirect to the provider (google, github)",
                    "tags": ["Auth"],
                    "security": [],
                    "parameters": [{"name": "provider", "in": "path", "required": true, "schema": {"type": "string", "enum": ["google", "github"]}}],
                    "responses": {"302": {"description": "Redirect to provider authorize page"}, "400": {"description": "Provider not configured"}, "404": {"description": "Unknown provider"}}
                }
            },
            "/api/auth/oauth/{provider}/callback": {
                "get": {
                    "summary": "OAuth2 callback — exchanges code, finds-or-creates user, returns JWT",
                    "tags": ["Auth"],
                    "security": [],
                    "parameters": [
                        {"name": "provider", "in": "path", "required": true, "schema": {"type": "string"}},
                        {"name": "code", "in": "query", "schema": {"type": "string"}},
                        {"name": "state", "in": "query", "schema": {"type": "string"}}
                    ],
                    "responses": {"200": {"description": "Login success, returns token"}, "400": {"description": "Invalid state/code or provider error"}}
                }
            },
            "/api/auth/users/{id}/impersonate": {
                "post": {
                    "summary": "Impersonate a user — admin-only, returns a user-scoped JWT",
                    "tags": ["Auth"],
                    "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}],
                    "responses": {"200": {"description": "Login response for the target user"}, "403": {"description": "Admin role required"}, "404": {"description": "User not found"}}
                }
            },
            "/api/auth/refresh": {
                "post": {
                    "summary": "Refresh the session — valid Bearer token exchanged for a fresh JWT",
                    "tags": ["Auth"],
                    "responses": {"200": {"description": "LoginResponse with fresh token"}, "401": {"description": "Unauthorized"}}
                }
            },
            "/api/auth/mfa/enable": {
                "post": {
                    "summary": "Start TOTP MFA enrollment — returns secret + otpauth URL",
                    "tags": ["Auth"],
                    "responses": {"200": {"description": "{secret, otpauth_url}"}, "400": {"description": "MFA already enabled"}}
                }
            },
            "/api/auth/mfa/confirm": {
                "post": {
                    "summary": "Confirm MFA enrollment with a TOTP code",
                    "tags": ["Auth"],
                    "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {"code": {"type": "string"}}, "required": ["code"]}}}},
                    "responses": {"200": {"description": "MFA enabled"}, "401": {"description": "Invalid code"}}
                }
            },
            "/api/auth/mfa/disable": {
                "post": {
                    "summary": "Disable MFA — requires a valid TOTP code",
                    "tags": ["Auth"],
                    "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {"code": {"type": "string"}}, "required": ["code"]}}}},
                    "responses": {"200": {"description": "MFA disabled"}, "401": {"description": "Invalid code"}}
                }
            },
            "/api/auth/mfa/login": {
                "post": {
                    "summary": "Login with password + TOTP code (used when /api/auth/login returns mfa_required)",
                    "tags": ["Auth"],
                    "security": [],
                    "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {
                        "email": {"type": "string"}, "password": {"type": "string"}, "code": {"type": "string"}
                    }, "required": ["email", "password", "code"]}}}},
                    "responses": {"200": {"description": "LoginResponse with token"}, "401": {"description": "Invalid credentials or code"}}
                }
            },
            "/api/auth/login": {
                "post": {
                    "summary": "Login and get JWT token",
                    "tags": ["Auth"],
                    "security": [],
                    "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {
                        "email": {"type": "string"},
                        "password": {"type": "string"}
                    }, "required": ["email", "password"]}}}},
                    "responses": {
                        "200": {"description": "Login success, returns token"},
                        "401": {"description": "Invalid credentials"}
                    }
                }
            }
    }).as_object().unwrap().clone()
}
