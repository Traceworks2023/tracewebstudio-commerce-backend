use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::Category;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub parent_id: Option<Uuid>,
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct CategoryResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Category> for CategoryResponse {
    fn from(c: Category) -> Self {
        Self {
            id: c.id,
            tenant_id: c.tenant_id,
            parent_id: c.parent_id,
            name: c.name,
            slug: c.slug,
            description: c.description,
            image_url: c.image_url,
            sort_order: c.sort_order,
            is_active: c.is_active,
            metadata: c.metadata,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListCategoriesQuery {
    pub tenant_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub is_active: Option<bool>,
}

pub async fn create_category(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateCategoryRequest>,
) -> Result<(StatusCode, Json<CategoryResponse>), StatusCode> {
    let tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    
    let category = sqlx::query_as::<_, Category>(
        r#"
        INSERT INTO categories 
        (tenant_id, parent_id, name, slug, description, image_url, sort_order, is_active, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING *
        "#,
    )
    .bind(tenant_id)
    .bind(payload.parent_id)
    .bind(&payload.name)
    .bind(&payload.slug)
    .bind(&payload.description)
    .bind(&payload.image_url)
    .bind(payload.sort_order.unwrap_or(0))
    .bind(payload.is_active.unwrap_or(true))
    .bind(&payload.metadata)
    .fetch_one(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create category: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok((StatusCode::CREATED, Json(CategoryResponse::from(category))))
}

pub async fn list_categories(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListCategoriesQuery>,
) -> Result<Json<Vec<CategoryResponse>>, StatusCode> {
    let categories = sqlx::query_as::<_, Category>(
        r#"
        SELECT * FROM categories 
        WHERE tenant_id = $1 
        AND ($2::uuid IS NULL OR parent_id = $2)
        AND ($3::boolean IS NULL OR is_active = $3)
        ORDER BY sort_order, name
        "#,
    )
    .bind(params.tenant_id)
    .bind(params.parent_id)
    .bind(params.is_active)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list categories: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(categories.into_iter().map(CategoryResponse::from).collect()))
}

pub async fn get_category(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, category_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<CategoryResponse>, StatusCode> {
    let category = sqlx::query_as::<_, Category>(
        "SELECT * FROM categories WHERE id = $1 AND tenant_id = $2",
    )
    .bind(category_id)
    .bind(tenant_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get category: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(CategoryResponse::from(category)))
}

pub async fn update_category(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, category_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateCategoryRequest>,
) -> Result<Json<CategoryResponse>, StatusCode> {
    let category = sqlx::query_as::<_, Category>(
        r#"
        UPDATE categories SET
            parent_id = COALESCE($3, parent_id),
            name = COALESCE($4, name),
            slug = COALESCE($5, slug),
            description = COALESCE($6, description),
            image_url = COALESCE($7, image_url),
            sort_order = COALESCE($8, sort_order),
            is_active = COALESCE($9, is_active),
            metadata = COALESCE($10, metadata),
            updated_at = NOW()
        WHERE id = $1 AND tenant_id = $2
        RETURNING *
        "#,
    )
    .bind(category_id)
    .bind(tenant_id)
    .bind(payload.parent_id)
    .bind(&payload.name)
    .bind(&payload.slug)
    .bind(&payload.description)
    .bind(&payload.image_url)
    .bind(payload.sort_order)
    .bind(payload.is_active)
    .bind(&payload.metadata)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update category: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(CategoryResponse::from(category)))
}

pub async fn delete_category(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, category_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query(
        "DELETE FROM categories WHERE id = $1 AND tenant_id = $2",
    )
    .bind(category_id)
    .bind(tenant_id)
    .execute(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete category: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    
    Ok(StatusCode::NO_CONTENT)
}
