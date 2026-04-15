use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{Discount, DiscountResponse};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateDiscountRequest {
    pub name: String,
    pub code: Option<String>,
    pub discount_type: String,
    pub value: f64,
    pub minimum_order_amount: Option<f64>,
    pub maximum_discount: Option<f64>,
    pub usage_limit: Option<i32>,
    pub usage_count: Option<i32>,
    pub starts_at: Option<chrono::DateTime<chrono::Utc>>,
    pub ends_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_active: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDiscountRequest {
    pub name: Option<String>,
    pub code: Option<String>,
    pub discount_type: Option<String>,
    pub value: Option<f64>,
    pub minimum_order_amount: Option<f64>,
    pub maximum_discount: Option<f64>,
    pub usage_limit: Option<i32>,
    pub usage_count: Option<i32>,
    pub starts_at: Option<chrono::DateTime<chrono::Utc>>,
    pub ends_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_active: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListDiscountsQuery {
    pub tenant_id: Uuid,
    pub is_active: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn create_discount(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateDiscountRequest>,
) -> Result<(StatusCode, Json<DiscountResponse>), StatusCode> {
    let tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();

    let discount = sqlx::query_as::<_, Discount>(
        r#"
        INSERT INTO discounts (tenant_id, name, code, discount_type, value, minimum_order_amount, maximum_discount, usage_limit, usage_count, starts_at, ends_at, is_active, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        RETURNING *
        "#,
    )
    .bind(tenant_id)
    .bind(&payload.name)
    .bind(&payload.code)
    .bind(&payload.discount_type)
    .bind(payload.value)
    .bind(payload.minimum_order_amount)
    .bind(payload.maximum_discount)
    .bind(payload.usage_limit)
    .bind(payload.usage_count.unwrap_or(0))
    .bind(payload.starts_at)
    .bind(payload.ends_at)
    .bind(payload.is_active.unwrap_or(true))
    .bind(&payload.metadata)
    .fetch_one(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create discount: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, Json(DiscountResponse::from(discount))))
}

pub async fn list_discounts(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListDiscountsQuery>,
) -> Result<Json<Vec<DiscountResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);

    let discounts = sqlx::query_as::<_, Discount>(
        r#"
        SELECT * FROM discounts
        WHERE tenant_id = $1
        AND ($2::boolean IS NULL OR is_active = $2)
        ORDER BY created_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(params.tenant_id)
    .bind(params.is_active)
    .bind(limit)
    .bind(offset)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list discounts: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(discounts.into_iter().map(DiscountResponse::from).collect()))
}

pub async fn get_discount(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, discount_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<DiscountResponse>, StatusCode> {
    let discount = sqlx::query_as::<_, Discount>(
        "SELECT * FROM discounts WHERE id = $1 AND tenant_id = $2",
    )
    .bind(discount_id)
    .bind(tenant_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get discount: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(DiscountResponse::from(discount)))
}

pub async fn update_discount(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, discount_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateDiscountRequest>,
) -> Result<Json<DiscountResponse>, StatusCode> {
    let discount = sqlx::query_as::<_, Discount>(
        r#"
        UPDATE discounts SET
            name = COALESCE($3, name),
            code = COALESCE($4, code),
            discount_type = COALESCE($5, discount_type),
            value = COALESCE($6, value),
            minimum_order_amount = COALESCE($7, minimum_order_amount),
            maximum_discount = COALESCE($8, maximum_discount),
            usage_limit = COALESCE($9, usage_limit),
            usage_count = COALESCE($10, usage_count),
            starts_at = COALESCE($11, starts_at),
            ends_at = COALESCE($12, ends_at),
            is_active = COALESCE($13, is_active),
            metadata = COALESCE($14, metadata),
            updated_at = NOW()
        WHERE id = $1 AND tenant_id = $2
        RETURNING *
        "#,
    )
    .bind(discount_id)
    .bind(tenant_id)
    .bind(&payload.name)
    .bind(&payload.code)
    .bind(&payload.discount_type)
    .bind(payload.value)
    .bind(payload.minimum_order_amount)
    .bind(payload.maximum_discount)
    .bind(payload.usage_limit)
    .bind(payload.usage_count)
    .bind(payload.starts_at)
    .bind(payload.ends_at)
    .bind(payload.is_active)
    .bind(&payload.metadata)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update discount: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(DiscountResponse::from(discount)))
}

pub async fn delete_discount(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, discount_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query(
        "DELETE FROM discounts WHERE id = $1 AND tenant_id = $2",
    )
    .bind(discount_id)
    .bind(tenant_id)
    .execute(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete discount: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}
