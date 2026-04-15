use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{InventoryItem, InventoryResponse};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct UpdateInventoryRequest {
    pub quantity: i32,
    pub adjustment_reason: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

pub async fn get_inventory(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, product_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<InventoryResponse>>, StatusCode> {
    let items = sqlx::query_as::<_, InventoryItem>(
        "SELECT * FROM inventory WHERE product_id = $1 AND tenant_id = $2",
    )
    .bind(product_id)
    .bind(tenant_id)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get inventory: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(items.into_iter().map(InventoryResponse::from).collect()))
}

pub async fn update_inventory(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, product_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateInventoryRequest>,
) -> Result<Json<InventoryResponse>, StatusCode> {
    let item = sqlx::query_as::<_, InventoryItem>(
        r#"
        UPDATE inventory SET
            quantity = $3,
            available_quantity = $3 - reserved_quantity,
            metadata = COALESCE($4, metadata),
            updated_at = NOW()
        WHERE product_id = $1 AND tenant_id = $2
        RETURNING *
        "#,
    )
    .bind(product_id)
    .bind(tenant_id)
    .bind(payload.quantity)
    .bind(&payload.metadata)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update inventory: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(InventoryResponse::from(item)))
}
