//! Security management for MatchForge SDK
//! 
//! Provides comprehensive security features including authentication, authorization,
/// and security context management.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{rate_limiter::RateLimiter, anti_abuse::AntiAbuseSystem};

/// Security configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Enable authentication
    pub enable_authentication: bool,
    
    /// Enable authorization
    pub enable_authorization: bool,
    
    /// Session timeout
    pub session_timeout: Duration,
    
    /// Maximum concurrent sessions per user
    pub max_concurrent_sessions: usize,
    
    /// Require HTTPS
    pub require_https: bool,
    
    /// Allowed origins for CORS
    pub allowed_origins: Vec<String>,
    
    /// Rate limiting configuration
    pub rate_limit_config: Option<super::rate_limiter::RateLimitConfig>,
    
    /// Anti-abuse configuration
    pub anti_abuse_config: Option<super::anti_abuse::AntiAbuseConfig>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_authentication: true,
            enable_authorization: true,
            session_timeout: Duration::from_hours(24),
            max_concurrent_sessions: 5,
            require_https: true,
            allowed_origins: vec!["*".to_string()],
            rate_limit_config: Some(super::rate_limiter::RateLimitConfig::default()),
            anti_abuse_config: Some(super::anti_abuse::AntiAbuseConfig::default()),
        }
    }
}

/// Security manager
pub struct SecurityManager {
    config: SecurityConfig,
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    rate_limiter: Option<RateLimiter>,
    anti_abuse_system: Option<AntiAbuseSystem>,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new(config: SecurityConfig) -> Self {
        let rate_limiter = config.rate_limit_config.clone().map(RateLimiter::new);
        let anti_abuse_system = config.anti_abuse_config.clone().map(AntiAbuseSystem::new);
        
        Self {
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            rate_limiter,
            anti_abuse_system,
        }
    }
    
    /// Create a security context for a request
    pub async fn create_context(&self, request: &SecurityRequest) -> Result<SecurityContext, SecurityError> {
        // Check rate limiting
        if let Some(ref rate_limiter) = self.rate_limiter {
            let client_id = self.extract_client_id(request)?;
            match rate_limiter.check_rate_limit(client_id).await {
                super::rate_limiter::RateLimitResult::Allowed => {}
                super::rate_limiter::RateLimitResult::Denied { reason, .. } => {
                    return Err(SecurityError::RateLimitExceeded(reason));
                }
            }
        }
        
        // Authenticate if required
        let user_id = if self.config.enable_authentication {
            self.authenticate(request)?
        } else {
            None
        };
        
        // Authorize if required
        if self.config.enable_authorization {
            self.authorize(request, user_id)?;
        }
        
        // Check for abuse
        if let Some(ref anti_abuse) = self.anti_abuse_system {
            if let Some(user_id) = user_id {
                let detection = anti_abuse.detect_abuse(user_id).await;
                if detection.abuse_level >= super::anti_abuse::AbuseLevel::High {
                    return Err(SecurityError::AbuseDetected(format!(
                        "Abuse detected: {:?}", detection.detected_activities
                    )));
                }
            }
        }
        
        // Create or update session
        let session_id = if let Some(user_id) = user_id {
            let session_id = self.create_or_update_session(user_id).await?;
            Some(session_id)
        } else {
            None
        };
        
        Ok(SecurityContext {
            user_id,
            session_id,
            client_id: self.extract_client_id(request).ok(),
            permissions: self.get_permissions(user_id).await,
            created_at: Utc::now(),
        })
    }
    
    /// Validate a security context
    pub async fn validate_context(&self, context: &SecurityContext) -> Result<(), SecurityError> {
        // Check session if it exists
        if let Some(ref session_id) = context.session_id {
            let sessions = self.sessions.read().await;
            if let Some(session) = sessions.get(session_id) {
                if session.expires_at < Utc::now() {
                    return Err(SecurityError::SessionExpired);
                }
            } else {
                return Err(SecurityError::InvalidSession);
            }
        }
        
        // Check rate limiting
        if let Some(client_id) = context.client_id {
            if let Some(ref rate_limiter) = self.rate_limiter {
                match rate_limiter.check_rate_limit(client_id).await {
                    super::rate_limiter::RateLimitResult::Allowed => {}
                    super::rate_limiter::RateLimitResult::Denied { reason, .. } => {
                        return Err(SecurityError::RateLimitExceeded(reason));
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Revoke a session
    pub async fn revoke_session(&self, session_id: &str) -> Result<(), SecurityError> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
        Ok(())
    }
    
    /// Revoke all sessions for a user
    pub async fn revoke_user_sessions(&self, user_id: Uuid) -> Result<(), SecurityError> {
        let mut sessions = self.sessions.write().await;
        sessions.retain(|_, session| session.user_id != user_id);
        Ok(())
    }
    
    /// Get active sessions for a user
    pub async fn get_user_sessions(&self, user_id: Uuid) -> Vec<Session> {
        let sessions = self.sessions.read().await;
        sessions.values()
            .filter(|session| session.user_id == user_id)
            .cloned()
            .collect()
    }
    
    /// Clean up expired sessions
    pub async fn cleanup_sessions(&self) -> Result<(), SecurityError> {
        let now = Utc::now();
        let mut sessions = self.sessions.write().await;
        sessions.retain(|_, session| session.expires_at > now);
        Ok(())
    }
    
    /// Extract client ID from request
    fn extract_client_id(&self, request: &SecurityRequest) -> Result<Uuid, SecurityError> {
        // Try to get client ID from various sources
        if let Some(client_id) = request.headers.get("X-Client-ID") {
            Ok(Uuid::parse_str(client_id).map_err(|_| SecurityError::InvalidClientId)?)
        } else if let Some(ip) = &request.remote_addr {
            // Use IP address as client ID (in production, this should be more sophisticated)
            let hash = std::collections::hash_map::DefaultHasher::new();
            Ok(Uuid::new_v4()) // Placeholder - in real implementation, hash the IP
        } else {
            Err(SecurityError::MissingClientId)
        }
    }
    
    /// Authenticate a request
    fn authenticate(&self, request: &SecurityRequest) -> Result<Option<Uuid>, SecurityError> {
        // Try to authenticate using various methods
        if let Some(auth_header) = request.headers.get("Authorization") {
            if auth_header.starts_with("Bearer ") {
                let token = &auth_header[7..];
                self.validate_token(token)
            } else {
                Err(SecurityError::InvalidAuthFormat)
            }
        } else if let Some(session_id) = request.headers.get("X-Session-ID") {
            self.validate_session(session_id)
        } else {
            Err(SecurityError::MissingAuth)
        }
    }
    
    /// Validate an authentication token
    fn validate_token(&self, token: &str) -> Result<Option<Uuid>, SecurityError> {
        // In a real implementation, this would validate JWT tokens or other auth mechanisms
        // For now, we'll just check if it's a valid UUID
        match Uuid::parse_str(token) {
            Ok(user_id) => Ok(Some(user_id)),
            Err(_) => Err(SecurityError::InvalidToken),
        }
    }
    
    /// Validate a session
    fn validate_session(&self, session_id: &str) -> Result<Option<Uuid>, SecurityError> {
        // This would need to be async in a real implementation
        // For now, we'll just return a placeholder
        Ok(None)
    }
    
    /// Authorize a request
    fn authorize(&self, request: &SecurityRequest, user_id: Option<Uuid>) -> Result<(), SecurityError> {
        // Check if the user has permission to perform the requested action
        let required_permission = match request.method.as_str() {
            "GET" => Permission::Read,
            "POST" => Permission::Create,
            "PUT" | "PATCH" => Permission::Update,
            "DELETE" => Permission::Delete,
            _ => return Err(SecurityError::InvalidMethod),
        };
        
        let user_permissions = self.get_permissions_sync(user_id);
        
        if !user_permissions.contains(&required_permission) {
            return Err(SecurityError::InsufficientPermissions);
        }
        
        Ok(())
    }
    
    /// Get permissions for a user
    async fn get_permissions(&self, user_id: Option<Uuid>) -> HashSet<Permission> {
        self.get_permissions_sync(user_id)
    }
    
    /// Get permissions for a user (sync version)
    fn get_permissions_sync(&self, user_id: Option<Uuid>) -> HashSet<Permission> {
        // In a real implementation, this would look up permissions from a database
        // For now, we'll return basic permissions based on whether the user is authenticated
        if user_id.is_some() {
            [Permission::Read, Permission::Create, Permission::Update].iter().cloned().collect()
        } else {
            [Permission::Read].iter().cloned().collect()
        }
    }
    
    /// Create or update a session
    async fn create_or_update_session(&self, user_id: Uuid) -> Result<String, SecurityError> {
        let mut sessions = self.sessions.write().await;
        
        // Check user's current sessions
        let user_sessions: Vec<_> = sessions.values()
            .filter(|s| s.user_id == user_id && s.expires_at > Utc::now())
            .collect();
        
        // Enforce max concurrent sessions
        if user_sessions.len() >= self.config.max_concurrent_sessions {
            // Remove the oldest session
            if let Some(oldest_session) = user_sessions.iter().min_by_key(|s| s.created_at) {
                let oldest_id = oldest_session.id.clone();
                sessions.remove(&oldest_id);
            }
        }
        
        // Create new session
        let session_id = Uuid::new_v4().to_string();
        let session = Session {
            id: session_id.clone(),
            user_id,
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::from_std(self.config.session_timeout).unwrap(),
            last_activity: Utc::now(),
        };
        
        sessions.insert(session_id.clone(), session);
        Ok(session_id)
    }
}

/// Security request context
#[derive(Debug, Clone)]
pub struct SecurityRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub remote_addr: Option<String>,
    pub user_agent: Option<String>,
}

/// Security context for authenticated requests
#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub user_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub client_id: Option<Uuid>,
    pub permissions: HashSet<Permission>,
    pub created_at: DateTime<Utc>,
}

/// Session information
#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

/// Permission types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Permission {
    Read,
    Create,
    Update,
    Delete,
    Admin,
}

/// Security errors
#[derive(Debug, Clone)]
pub enum SecurityError {
    MissingAuth,
    InvalidAuthFormat,
    InvalidToken,
    InvalidSession,
    SessionExpired,
    InsufficientPermissions,
    RateLimitExceeded(String),
    AbuseDetected(String),
    InvalidClientId,
    MissingClientId,
    InvalidMethod,
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityError::MissingAuth => write!(f, "Missing authentication"),
            SecurityError::InvalidAuthFormat => write!(f, "Invalid authentication format"),
            SecurityError::InvalidToken => write!(f, "Invalid authentication token"),
            SecurityError::InvalidSession => write!(f, "Invalid session"),
            SecurityError::SessionExpired => write!(f, "Session has expired"),
            SecurityError::InsufficientPermissions => write!(f, "Insufficient permissions"),
            SecurityError::RateLimitExceeded(reason) => write!(f, "Rate limit exceeded: {}", reason),
            SecurityError::AbuseDetected(reason) => write!(f, "Abuse detected: {}", reason),
            SecurityError::InvalidClientId => write!(f, "Invalid client ID"),
            SecurityError::MissingClientId => write!(f, "Missing client ID"),
            SecurityError::InvalidMethod => write!(f, "Invalid HTTP method"),
        }
    }
}

impl std::error::Error for SecurityError {}

/// Security middleware for web frameworks
pub struct SecurityMiddleware {
    security_manager: Arc<SecurityManager>,
}

impl SecurityMiddleware {
    /// Create a new security middleware
    pub fn new(security_manager: Arc<SecurityManager>) -> Self {
        Self { security_manager }
    }
    
    /// Process a request through security checks
    pub async fn process_request(&self, request: SecurityRequest) -> Result<SecurityContext, SecurityError> {
        let context = self.security_manager.create_context(&request).await?;
        self.security_manager.validate_context(&context).await?;
        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_security_context_creation() {
        let config = SecurityConfig::default();
        let manager = SecurityManager::new(config);
        
        let request = SecurityRequest {
            method: "GET".to_string(),
            path: "/api/queues".to_string(),
            headers: HashMap::new(),
            remote_addr: Some("127.0.0.1".to_string()),
            user_agent: Some("test-agent".to_string()),
        };
        
        // Should fail without authentication
        assert!(matches!(manager.create_context(&request).await, Err(SecurityError::MissingAuth)));
    }
    
    #[tokio::test]
    async fn test_session_management() {
        let config = SecurityConfig {
            enable_authentication: false,
            ..Default::default()
        };
        let manager = SecurityManager::new(config);
        
        let user_id = Uuid::new_v4();
        let session_id = manager.create_or_update_session(user_id).await.unwrap();
        
        let sessions = manager.get_user_sessions(user_id).await;
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, session_id);
        
        manager.revoke_session(&session_id).await.unwrap();
        let sessions = manager.get_user_sessions(user_id).await;
        assert_eq!(sessions.len(), 0);
    }
}
