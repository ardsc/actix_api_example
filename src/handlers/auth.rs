use actix_web::HttpMessage;
use actix_web::{HttpResponse, web};
use chrono::{Duration, Utc};
use md5;
use sqlx::MySqlPool;
use uuid::Uuid;
use validator::Validate;

use crate::models::user::{LoginForm, User, UserResponse};

pub async fn login(pool: web::Data<MySqlPool>, credentials: web::Json<LoginForm>) -> HttpResponse {
    // Input Validation
    if let Err(validation_errors) = credentials.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": validation_errors
        }));
    }

    // Hash password dengan MD5
    let password_hash = format!("{:x}", md5::compute(&credentials.password));

    // Find user by username dan password
    let user_result =
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ? AND password = ?")
            .bind(&credentials.username)
            .bind(&password_hash)
            .fetch_optional(pool.get_ref())
            .await;

    let user = match user_result {
        Ok(Some(user)) => user,
        Ok(None) => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Invalid credentials"}));
        }
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Database error"}));
        }
    };

    // Generate token UUID
    let token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::days(7);

    // Save token to database
    let insert_result = sqlx::query(
        "INSERT INTO personal_access_tokens (user_id, token, expires_at) VALUES (?, ?, ?)",
    )
    .bind(user.id)
    .bind(&token)
    .bind(expires_at)
    .execute(pool.get_ref())
    .await;

    if insert_result.is_err() {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Failed to create token"}));
    }

    // Return token response
    HttpResponse::Ok().json(serde_json::json!({
        "token": token,
        "token_type": "Bearer",
        "expires_at": expires_at
    }))
}

pub async fn logout(pool: web::Data<MySqlPool>, req: actix_web::HttpRequest) -> HttpResponse {
    // Extract token from Authorization header
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|auth_header| {
            if auth_header.starts_with("Bearer ") {
                Some(auth_header[7..].to_string())
            } else {
                None
            }
        });

    if let Some(token) = token {
        // Delete token from database
        let delete_result = sqlx::query("DELETE FROM personal_access_tokens WHERE token = ?")
            .bind(&token)
            .execute(pool.get_ref())
            .await;

        if delete_result.is_ok() {
            HttpResponse::Ok().json(serde_json::json!({"message": "Logged out successfully"}))
        } else {
            HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Failed to logout"}))
        }
    } else {
        HttpResponse::Unauthorized().json(serde_json::json!({"error": "Invalid or missing token"}))
    }
}

pub async fn get_user_info(req: actix_web::HttpRequest) -> HttpResponse {
    // Get user from request extensions (added by middleware)
    if let Some(user) = req.extensions().get::<UserResponse>() {
        HttpResponse::Ok().json(user)
    } else {
        HttpResponse::Unauthorized().json(serde_json::json!({"error": "User not found"}))
    }
}
