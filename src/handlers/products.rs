use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{Product, InventoryPolicy, WeightUnit};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub category_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub price: f64,
    pub compare_at_price: Option<f64>,
    pub cost_price: Option<f64>,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub inventory_quantity: Option<i32>,
    pub inventory_policy: Option<InventoryPolicy>,
    pub weight: Option<f64>,
    pub weight_unit: Option<WeightUnit>,
    pub is_active: Option<bool>,
    pub is_featured: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub category_id: Option<Uuid>,
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub price: Option<f64>,
    pub compare_at_price: Option<f64>,
    pub cost_price: Option<f64>,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub inventory_quantity: Option<i32>,
    pub inventory_policy: Option<InventoryPolicy>,
    pub weight: Option<f64>,
    pub weight_unit: Option<WeightUnit>,
    pub is_active: Option<bool>,
    pub is_featured: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ProductResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub category_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub price: f64,
    pub compare_at_price: Option<f64>,
    pub cost_price: Option<f64>,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub inventory_quantity: i32,
    pub inventory_policy: InventoryPolicy,
    pub weight: Option<f64>,
    pub weight_unit: WeightUnit,
    pub is_active: bool,
    pub is_featured: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Product> for ProductResponse {
    fn from(p: Product) -> Self {
        Self {
            id: p.id,
            tenant_id: p.tenant_id,
            category_id: p.category_id,
            name: p.name,
            slug: p.slug,
            description: p.description,
            price: p.price,
            compare_at_price: p.compare_at_price,
            cost_price: p.cost_price,
            sku: p.sku,
            barcode: p.barcode,
            inventory_quantity: p.inventory_quantity,
            inventory_policy: p.inventory_policy,
            weight: p.weight,
            weight_unit: p.weight_unit,
            is_active: p.is_active,
            is_featured: p.is_featured,
            metadata: p.metadata,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListProductsQuery {
    pub tenant_id: Uuid,
    pub category_id: Option<Uuid>,
    pub is_active: Option<bool>,
    pub is_featured: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn create_product(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateProductRequest>,
) -> Result<(StatusCode, Json<ProductResponse>), StatusCode> {
    let tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    
    let product = sqlx::query_as::<_, Product>(
        r#"
        INSERT INTO products 
        (tenant_id, category_id, name, slug, description, price, compare_at_price, cost_price, sku, barcode, inventory_quantity, inventory_policy, weight, weight_unit, is_active, is_featured, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
        RETURNING *
        "#,
    )
    .bind(tenant_id)
    .bind(payload.category_id)
    .bind(&payload.name)
    .bind(&payload.slug)
    .bind(&payload.description)
    .bind(payload.price)
    .bind(payload.compare_at_price)
    .bind(payload.cost_price)
    .bind(&payload.sku)
    .bind(&payload.barcode)
    .bind(payload.inventory_quantity.unwrap_or(0))
    .bind(payload.inventory_policy.unwrap_or(InventoryPolicy::Deny))
    .bind(payload.weight)
    .bind(payload.weight_unit.unwrap_or(WeightUnit::Kg))
    .bind(payload.is_active.unwrap_or(true))
    .bind(payload.is_featured.unwrap_or(false))
    .bind(&payload.metadata)
    .fetch_one(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create product: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok((StatusCode::CREATED, Json(ProductResponse::from(product))))
}

pub async fn list_products(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListProductsQuery>,
) -> Result<Json<Vec<ProductResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    
    let products = sqlx::query_as::<_, Product>(
        r#"
        SELECT * FROM products 
        WHERE tenant_id = $1 
        AND ($2::uuid IS NULL OR category_id = $2)
        AND ($3::boolean IS NULL OR is_active = $3)
        AND ($4::boolean IS NULL OR is_featured = $4)
        ORDER BY created_at DESC
        LIMIT $5 OFFSET $6
        "#,
    )
    .bind(params.tenant_id)
    .bind(params.category_id)
    .bind(params.is_active)
    .bind(params.is_featured)
    .bind(limit)
    .bind(offset)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list products: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(products.into_iter().map(ProductResponse::from).collect()))
}

pub async fn get_product(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, product_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ProductResponse>, StatusCode> {
    let product = sqlx::query_as::<_, Product>(
        "SELECT * FROM products WHERE id = $1 AND tenant_id = $2",
    )
    .bind(product_id)
    .bind(tenant_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get product: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(ProductResponse::from(product)))
}

pub async fn update_product(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, product_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateProductRequest>,
) -> Result<Json<ProductResponse>, StatusCode> {
    let product = sqlx::query_as::<_, Product>(
        r#"
        UPDATE products SET
            category_id = COALESCE($3, category_id),
            name = COALESCE($4, name),
            slug = COALESCE($5, slug),
            description = COALESCE($6, description),
            price = COALESCE($7, price),
            compare_at_price = COALESCE($8, compare_at_price),
            cost_price = COALESCE($9, cost_price),
            sku = COALESCE($10, sku),
            barcode = COALESCE($11, barcode),
            inventory_quantity = COALESCE($12, inventory_quantity),
            inventory_policy = COALESCE($13, inventory_policy),
            weight = COALESCE($14, weight),
            weight_unit = COALESCE($15, weight_unit),
            is_active = COALESCE($16, is_active),
            is_featured = COALESCE($17, is_featured),
            metadata = COALESCE($18, metadata),
            updated_at = NOW()
        WHERE id = $1 AND tenant_id = $2
        RETURNING *
        "#,
    )
    .bind(product_id)
    .bind(tenant_id)
    .bind(payload.category_id)
    .bind(&payload.name)
    .bind(&payload.slug)
    .bind(&payload.description)
    .bind(payload.price)
    .bind(payload.compare_at_price)
    .bind(payload.cost_price)
    .bind(&payload.sku)
    .bind(&payload.barcode)
    .bind(payload.inventory_quantity)
    .bind(payload.inventory_policy)
    .bind(payload.weight)
    .bind(payload.weight_unit)
    .bind(payload.is_active)
    .bind(payload.is_featured)
    .bind(&payload.metadata)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update product: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(ProductResponse::from(product)))
}

pub async fn delete_product(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, product_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query(
        "DELETE FROM products WHERE id = $1 AND tenant_id = $2",
    )
    .bind(product_id)
    .bind(tenant_id)
    .execute(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete product: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    
    Ok(StatusCode::NO_CONTENT)
}
