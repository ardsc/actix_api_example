use actix_web::{
    Error, HttpMessage,
    body::{BoxBody, EitherBody},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};
use futures_util::future::LocalBoxFuture;
use sqlx::MySqlPool;
use std::{
    future::{Ready, ready},
    rc::Rc,
};

use crate::models::user::UserResponse;

pub struct AuthMiddleware {
    pool: MySqlPool,
}

impl AuthMiddleware {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
            pool: self.pool.clone(),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    pool: MySqlPool,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut core::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        let pool = self.pool.clone();

        Box::pin(async move {
            // Skip auth for login endpoint
            if req.path() == "/api/login" {
                let res = svc.call(req).await?;
                return Ok(res.map_into_left_body());
            }

            // Extract token from Authorization header
            let token = extract_bearer_token(&req);

            if let Some(token) = token {
                // Validate token against database
                if let Ok(user) = validate_token(&pool, &token).await {
                    // Token is valid, attach user info to request
                    let req = req;
                    req.extensions_mut().insert(user);
                    let res = svc.call(req).await?;
                    return Ok(res.map_into_left_body());
                }
            }

            // Token is invalid or missing
            let (req, _pl) = req.into_parts();
            let response = actix_web::HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Invalid or missing token"}))
                .map_into_boxed_body();

            Ok(ServiceResponse::new(req, response).map_into_right_body())
        })
    }
}

// Helper function to extract bearer token
fn extract_bearer_token(req: &ServiceRequest) -> Option<String> {
    req.headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|auth_header| {
            if auth_header.starts_with("Bearer ") {
                Some(auth_header[7..].to_string())
            } else {
                None
            }
        })
}

// Validate token against database
async fn validate_token(pool: &MySqlPool, token: &str) -> Result<UserResponse, sqlx::Error> {
    let user = sqlx::query_as::<_, crate::models::user::User>(
        "SELECT u.* FROM users u
         INNER JOIN personal_access_tokens t ON u.id = t.user_id
         WHERE t.token = ? AND t.expires_at > UTC_TIMESTAMP()",
    )
    .bind(token)
    .fetch_one(pool)
    .await?;

    Ok(UserResponse::from(user))
}
