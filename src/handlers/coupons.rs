use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{Coupon, CouponResponse, CouponType};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateCouponRequest {
    pub code: String,
    pub coupon_type: CouponType,
    pub value: f64,
    pub minimum_order_amount: Option<f64>,
    pub maximum_discount: Option<f64>,
    pub usage_limit: Option<i32>,
    pub starts_at: chrono::DateTime<chrono::Utc>,
    pub ends_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_active: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCouponRequest {
    pub code: Option<String>,
    pub coupon_type: Option<CouponType>,
    pub value: Option<f64>,
    pub minimum_order_amount: Option<f64>,
    pub maximum_discount: Option<f64>,
    pub usage_limit: Option<i32>,
    pub starts_at: Option<chrono::DateTime<chrono::Utc>>,
    pub ends_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_active: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListCouponsQuery {
    pub tenant_id: Uuid,
    pub is_active: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn create_coupon(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateCouponRequest>,
) -> Result<(StatusCode, Json<CouponResponse>), StatusCode> {
    let tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();

    let coupon = sqlx::query_as::<_, Coupon>(
        r#"
        INSERT INTO coupons (tenant_id, code, coupon_type, value, minimum_order_amount, maximum_discount, usage_limit, starts_at, ends_at, is_active, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING *
        "#,
    )
    .bind(tenant_id)
    .bind(&payload.code)
    .bind(payload.coupon_type)
    .bind(payload.value)
    .bind(payload.minimum_order_amount)
    .bind(payload.maximum_discount)
    .bind(payload.usage_limit)
    .bind(payload.starts_at)
    .bind(payload.ends_at)
    .bind(payload.is_active.unwrap_or(true))
    .bind(&payload.metadata)
    .fetch_one(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create coupon: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, Json(CouponResponse::from(coupon))))
}

pub async fn list_coupons(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListCouponsQuery>,
) -> Result<Json<Vec<CouponResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);

    let coupons = sqlx::query_as::<_, Coupon>(
        r#"
        SELECT * FROM coupons
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
        tracing::error!("Failed to list coupons: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(coupons.into_iter().map(CouponResponse::from).collect()))
}

pub async fn get_coupon(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, coupon_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<CouponResponse>, StatusCode> {
    let coupon = sqlx::query_as::<_, Coupon>(
        "SELECT * FROM coupons WHERE id = $1 AND tenant_id = $2",
    )
    .bind(coupon_id)
    .bind(tenant_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get coupon: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(CouponResponse::from(coupon)))
}

pub async fn update_coupon(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, coupon_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateCouponRequest>,
) -> Result<Json<CouponResponse>, StatusCode> {
    let coupon = sqlx::query_as::<_, Coupon>(
        r#"
        UPDATE coupons SET
            code = COALESCE($3, code),
            coupon_type = COALESCE($4, coupon_type),
            value = COALESCE($5, value),
            minimum_order_amount = COALESCE($6, minimum_order_amount),
            maximum_discount = COALESCE($7, maximum_discount),
            usage_limit = COALESCE($8, usage_limit),
            starts_at = COALESCE($9, starts_at),
            ends_at = COALESCE($10, ends_at),
            is_active = COALESCE($11, is_active),
            metadata = COALESCE($12, metadata),
            updated_at = NOW()
        WHERE id = $1 AND tenant_id = $2
        RETURNING *
        "#,
    )
    .bind(coupon_id)
    .bind(tenant_id)
    .bind(&payload.code)
    .bind(payload.coupon_type)
    .bind(payload.value)
    .bind(payload.minimum_order_amount)
    .bind(payload.maximum_discount)
    .bind(payload.usage_limit)
    .bind(payload.starts_at)
    .bind(payload.ends_at)
    .bind(payload.is_active)
    .bind(&payload.metadata)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update coupon: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(CouponResponse::from(coupon)))
}

pub async fn delete_coupon(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, coupon_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query(
        "DELETE FROM coupons WHERE id = $1 AND tenant_id = $2",
    )
    .bind(coupon_id)
    .bind(tenant_id)
    .execute(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete coupon: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}
