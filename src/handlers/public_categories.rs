use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::Category;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct PublicCategoryResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub sort_order: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Category> for PublicCategoryResponse {
    fn from(c: Category) -> Self {
        Self {
            id: c.id,
            tenant_id: c.tenant_id,
            name: c.name,
            slug: c.slug,
            description: c.description,
            image_url: c.image_url,
            sort_order: c.sort_order,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListCategoriesQuery {
    pub tenant_id: Option<Uuid>,
}

pub async fn list_public_categories(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListCategoriesQuery>,
) -> Result<Json<Vec<PublicCategoryResponse>>, StatusCode> {
    let tenant_id = params
        .tenant_id
        .unwrap_or(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap());
    
    let categories = sqlx::query_as::<_, Category>(
        "SELECT * FROM categories WHERE tenant_id = $1 AND is_active = true ORDER BY sort_order, name",
    )
    .bind(tenant_id)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list public categories: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let response: Vec<PublicCategoryResponse> = categories
        .into_iter()
        .map(PublicCategoryResponse::from)
        .collect();
    
    Ok(Json(response))
}
