use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

pub const TENANT_ID_HEADER: &str = "x-tenant-id";
pub const ACTOR_ID_HEADER: &str = "x-actor-id";
pub const ACTOR_TYPE_HEADER: &str = "x-actor-type";

#[derive(Clone, Debug, PartialEq)]
pub enum ActorType {
    SuperAdmin,
    TenantOwner,
    TenantAdmin,
    TenantEditor,
    PublicVisitor,
}

pub async fn tenant_isolation_middleware(
    State(_state): State<Arc<super::AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();
    
    if is_public_path(path) {
        return Ok(next.run(request).await);
    }
    
    let tenant_id = request
        .headers()
        .get(TENANT_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    
    if tenant_id.is_none() && !is_public_path(path) {
        tracing::warn!(
            "Commerce: request without tenant context to {}",
            path
        );
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    Ok(next.run(request).await)
}

fn parse_actor_type(s: &str) -> ActorType {
    match s {
        "super_admin" => ActorType::SuperAdmin,
        "tenant_owner" => ActorType::TenantOwner,
        "tenant_admin" => ActorType::TenantAdmin,
        "tenant_editor" => ActorType::TenantEditor,
        _ => ActorType::PublicVisitor,
    }
}

fn is_public_path(path: &str) -> bool {
    matches!(
        path,
        "/health"
            | "/api/v1/products"
            | "/api/v1/categories"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_public_path() {
        assert!(is_public_path("/health"));
        assert!(is_public_path("/api/v1/products"));
        assert!(is_public_path("/api/v1/categories"));
        assert!(!is_public_path("/api/v1/orders"));
        assert!(!is_public_path("/api/v1/customers"));
    }

    #[test]
    fn test_parse_actor_type() {
        assert_eq!(parse_actor_type("super_admin"), ActorType::SuperAdmin);
        assert_eq!(parse_actor_type("tenant_admin"), ActorType::TenantAdmin);
        assert_eq!(parse_actor_type("unknown"), ActorType::PublicVisitor);
    }
}
