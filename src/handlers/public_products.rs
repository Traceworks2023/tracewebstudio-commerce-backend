use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::Product;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct PublicProductResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub price: f64,
    pub image: Option<String>,
    pub stock: i32,
    pub rating: Option<f64>,
}

impl From<Product> for PublicProductResponse {
    fn from(p: Product) -> Self {
        Self {
            id: p.id,
            name: p.name,
            slug: p.slug,
            description: p.description,
            price: p.price,
            image: None,
            stock: p.inventory_quantity,
            rating: None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListProductsQuery {
    pub site_id: Option<Uuid>,
    pub tenant_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
}

pub async fn list_public_products(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListProductsQuery>,
) -> Result<Json<Vec<PublicProductResponse>>, StatusCode> {
    let tenant_filter = params
        .tenant_id
        .map(|tid| format!("AND tenant_id = '{}'", tid))
        .unwrap_or_default();
    
    let category_filter = params
        .category_id
        .map(|cid| format!("AND category_id = '{}'", cid))
        .unwrap_or_default();
    
    let products = sqlx::query_as::<_, Product>(
        &format!(
            r#"
            SELECT * FROM products 
            WHERE is_active = true {}
            {}
            ORDER BY created_at DESC
            "#,
            tenant_filter, category_filter
        ),
    )
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list public products: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let response: Vec<PublicProductResponse> = products
        .into_iter()
        .map(PublicProductResponse::from)
        .collect();
    
    Ok(Json(response))
}

pub async fn get_public_product(
    State(state): State<Arc<AppState>>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<PublicProductResponse>, StatusCode> {
    let product = sqlx::query_as::<_, Product>(
        "SELECT * FROM products WHERE id = $1 AND is_active = true",
    )
    .bind(product_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get public product: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(PublicProductResponse::from(product)))
}
