use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{ShippingRate, ShippingRateResponse};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateShippingRateRequest {
    pub name: String,
    pub code: String,
    pub price: f64,
    pub weight_min: Option<f64>,
    pub weight_max: Option<f64>,
    pub is_active: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateShippingRateRequest {
    pub name: Option<String>,
    pub code: Option<String>,
    pub price: Option<f64>,
    pub weight_min: Option<f64>,
    pub weight_max: Option<f64>,
    pub is_active: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListShippingRatesQuery {
    pub tenant_id: Uuid,
    pub is_active: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn create_shipping_rate(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateShippingRateRequest>,
) -> Result<(StatusCode, Json<ShippingRateResponse>), StatusCode> {
    let tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();

    let rate = sqlx::query_as::<_, ShippingRate>(
        r#"
        INSERT INTO shipping_rates (tenant_id, name, code, price, weight_min, weight_max, is_active, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#,
    )
    .bind(tenant_id)
    .bind(&payload.name)
    .bind(&payload.code)
    .bind(payload.price)
    .bind(payload.weight_min)
    .bind(payload.weight_max)
    .bind(payload.is_active.unwrap_or(true))
    .bind(&payload.metadata)
    .fetch_one(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create shipping rate: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, Json(ShippingRateResponse::from(rate))))
}

pub async fn list_shipping_rates(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListShippingRatesQuery>,
) -> Result<Json<Vec<ShippingRateResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);

    let rates = sqlx::query_as::<_, ShippingRate>(
        r#"
        SELECT * FROM shipping_rates
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
        tracing::error!("Failed to list shipping rates: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(rates.into_iter().map(ShippingRateResponse::from).collect()))
}

pub async fn get_shipping_rate(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, rate_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ShippingRateResponse>, StatusCode> {
    let rate = sqlx::query_as::<_, ShippingRate>(
        "SELECT * FROM shipping_rates WHERE id = $1 AND tenant_id = $2",
    )
    .bind(rate_id)
    .bind(tenant_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get shipping rate: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(ShippingRateResponse::from(rate)))
}

pub async fn update_shipping_rate(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, rate_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateShippingRateRequest>,
) -> Result<Json<ShippingRateResponse>, StatusCode> {
    let rate = sqlx::query_as::<_, ShippingRate>(
        r#"
        UPDATE shipping_rates SET
            name = COALESCE($3, name),
            code = COALESCE($4, code),
            price = COALESCE($5, price),
            weight_min = COALESCE($6, weight_min),
            weight_max = COALESCE($7, weight_max),
            is_active = COALESCE($8, is_active),
            metadata = COALESCE($9, metadata),
            updated_at = NOW()
        WHERE id = $1 AND tenant_id = $2
        RETURNING *
        "#,
    )
    .bind(rate_id)
    .bind(tenant_id)
    .bind(&payload.name)
    .bind(&payload.code)
    .bind(payload.price)
    .bind(payload.weight_min)
    .bind(payload.weight_max)
    .bind(payload.is_active)
    .bind(&payload.metadata)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update shipping rate: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(ShippingRateResponse::from(rate)))
}

pub async fn delete_shipping_rate(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, rate_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query(
        "DELETE FROM shipping_rates WHERE id = $1 AND tenant_id = $2",
    )
    .bind(rate_id)
    .bind(tenant_id)
    .execute(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete shipping rate: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}
