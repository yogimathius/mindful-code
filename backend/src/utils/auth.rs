use crate::error::{AppError, Result};
use axum::{
    extract::{FromRequestParts, Request},
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub user_id: Uuid,
    pub email: String,
    pub subscription_tier: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

impl Claims {
    pub fn new(user_id: Uuid, email: String, subscription_tier: String) -> Self {
        let iat = chrono::Utc::now().timestamp() as usize;
        let exp = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;

        Self {
            user_id,
            email,
            subscription_tier,
            exp,
            iat,
        }
    }

    pub fn is_premium(&self) -> bool {
        matches!(self.subscription_tier.as_str(), "premium" | "team")
    }

    pub fn is_team(&self) -> bool {
        self.subscription_tier == "team"
    }
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|header| header.to_str().ok())
            .ok_or_else(|| AppError::Authentication("Missing authorization header".to_string()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::Authentication(
                "Invalid authorization header format".to_string(),
            ));
        }

        let token = &auth_header[7..];
        
        // Get JWT secret from environment or state
        // For now, we'll use a placeholder - in production, this should come from app state
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-super-secret-jwt-key-change-in-production".to_string());

        validate_jwt_token(token, &jwt_secret)
    }
}

pub fn generate_jwt_token(claims: &Claims, secret: &str) -> Result<String> {
    let header = Header::default();
    let encoding_key = EncodingKey::from_secret(secret.as_ref());

    encode(&header, claims, &encoding_key)
        .map_err(|e| AppError::Authentication(format!("Failed to generate token: {}", e)))
}

pub fn validate_jwt_token(token: &str, secret: &str) -> Result<Claims> {
    let decoding_key = DecodingKey::from_secret(secret.as_ref());
    let validation = Validation::new(Algorithm::HS256);

    decode::<Claims>(token, &decoding_key, &validation)
        .map(|data| data.claims)
        .map_err(|e| AppError::Authentication(format!("Invalid token: {}", e)))
}

pub fn generate_refresh_token(user_id: Uuid, secret: &str) -> Result<String> {
    let claims = Claims {
        user_id,
        email: "refresh".to_string(), // Placeholder for refresh token
        subscription_tier: "refresh".to_string(),
        iat: chrono::Utc::now().timestamp() as usize,
        exp: (chrono::Utc::now() + chrono::Duration::days(30)).timestamp() as usize,
    };

    generate_jwt_token(&claims, secret)
}

pub fn hash_password(password: &str) -> Result<String> {
    use argon2::{Argon2, PasswordHasher};
    use argon2::password_hash::{rand_core::OsRng, SaltString};

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AppError::Internal(format!("Password hashing failed: {}", e)))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};

    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("Invalid password hash: {}", e)))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

use crate::state::AppState;

pub async fn auth_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip auth for certain paths
    let path = req.uri().path();
    
    if should_skip_auth(path) {
        return Ok(next.run(req).await);
    }

    // Extract and validate JWT token
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    if let Some(auth_header) = auth_header {
        if auth_header.starts_with("Bearer ") {
            let token = &auth_header[7..];
            let jwt_secret = std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-super-secret-jwt-key-change-in-production".to_string());

            if validate_jwt_token(token, &jwt_secret).is_ok() {
                return Ok(next.run(req).await);
            }
        }
    }

    // Return 401 for protected routes without valid auth
    Err(StatusCode::UNAUTHORIZED)
}

fn should_skip_auth(path: &str) -> bool {
    matches!(
        path,
        "/health" 
        | "/metrics" 
        | "/api/auth/register" 
        | "/api/auth/login" 
        | "/api/auth/refresh"
    )
}

// Rate limiting utilities
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window_duration: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_duration: Duration) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window_duration,
        }
    }

    pub fn check_rate_limit(&self, identifier: &str) -> bool {
        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();
        
        let user_requests = requests.entry(identifier.to_string()).or_insert_with(Vec::new);
        
        // Remove old requests outside the window
        user_requests.retain(|&time| now.duration_since(time) < self.window_duration);
        
        if user_requests.len() >= self.max_requests {
            false
        } else {
            user_requests.push(now);
            true
        }
    }

    pub fn cleanup_old_entries(&self) {
        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();
        
        requests.retain(|_, times| {
            times.retain(|&time| now.duration_since(time) < self.window_duration);
            !times.is_empty()
        });
    }
}

// Permission checking utilities
pub fn require_premium(claims: &Claims) -> Result<()> {
    if claims.is_premium() {
        Ok(())
    } else {
        Err(AppError::Authorization(
            "Premium subscription required".to_string(),
        ))
    }
}

pub fn require_team(claims: &Claims) -> Result<()> {
    if claims.is_team() {
        Ok(())
    } else {
        Err(AppError::Authorization(
            "Team subscription required".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_token_generation_and_validation() {
        let user_id = Uuid::new_v4();
        let claims = Claims::new(
            user_id,
            "test@example.com".to_string(),
            "free".to_string(),
        );
        let secret = "test-secret";

        let token = generate_jwt_token(&claims, secret).unwrap();
        let validated_claims = validate_jwt_token(&token, secret).unwrap();

        assert_eq!(claims.user_id, validated_claims.user_id);
        assert_eq!(claims.email, validated_claims.email);
    }

    #[test]
    fn test_password_hashing_and_verification() {
        let password = "test-password-123";
        let hash = hash_password(password).unwrap();
        
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong-password", &hash).unwrap());
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(2, Duration::from_secs(60));
        
        assert!(limiter.check_rate_limit("user1"));
        assert!(limiter.check_rate_limit("user1"));
        assert!(!limiter.check_rate_limit("user1")); // Should be rate limited
        
        assert!(limiter.check_rate_limit("user2")); // Different user, should pass
    }
}