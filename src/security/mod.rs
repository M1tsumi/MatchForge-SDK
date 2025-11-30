//! Security and anti-abuse mechanisms for MatchForge SDK
//! 
//! This module provides rate limiting, anti-abuse protection, and security features
//! to ensure fair and safe matchmaking operations.

pub mod rate_limiter;
pub mod anti_abuse;
pub mod security;

pub use rate_limiter::{RateLimiter, RateLimitConfig, RateLimitResult};
pub use anti_abuse::{AntiAbuseSystem, AbuseDetection, AbuseAction, AbuseReport};
pub use security::{SecurityConfig, SecurityManager, SecurityContext};
