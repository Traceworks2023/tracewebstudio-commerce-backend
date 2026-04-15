use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{PaymentGateway, PaymentGatewayResponse};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct CreatePaymentGatewayRequest {
    pub name: String,
    pub gateway_type: String,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub webhook_secret: Option<String>,
    pub is_active: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePaymentGatewayRequest {
    pub name: Option<String>,
    pub gateway_type: Option<String>,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub webhook_secret: Option<String>,
    pub is_active: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListPaymentGatewaysQuery {
    pub tenant_id: Uuid,
    pub is_active: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn create_payment_gateway(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreatePaymentGatewayRequest>,
) -> Result<(StatusCode, Json<PaymentGatewayResponse>), StatusCode> {
    let tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();

    let gateway = sqlx::query_as::<_, PaymentGateway>(
        r#"
        INSERT INTO payment_gateways (tenant_id, name, gateway_type, api_key, api_secret, webhook_secret, is_active, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#,
    )
    .bind(tenant_id)
    .bind(&payload.name)
    .bind(&payload.gateway_type)
    .bind(&payload.api_key)
    .bind(&payload.api_secret)
    .bind(&payload.webhook_secret)
    .bind(payload.is_active.unwrap_or(true))
    .bind(&payload.metadata)
    .fetch_one(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create payment gateway: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, Json(PaymentGatewayResponse::from(gateway))))
}

pub async fn list_payment_gateways(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListPaymentGatewaysQuery>,
) -> Result<Json<Vec<PaymentGatewayResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);

    let gateways = sqlx::query_as::<_, PaymentGateway>(
        r#"
        SELECT * FROM payment_gateways
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
        tracing::error!("Failed to list payment gateways: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(gateways.into_iter().map(PaymentGatewayResponse::from).collect()))
}

pub async fn get_payment_gateway(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, gateway_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<PaymentGatewayResponse>, StatusCode> {
    let gateway = sqlx::query_as::<_, PaymentGateway>(
        "SELECT * FROM payment_gateways WHERE id = $1 AND tenant_id = $2",
    )
    .bind(gateway_id)
    .bind(tenant_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get payment gateway: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(PaymentGatewayResponse::from(gateway)))
}

pub async fn update_payment_gateway(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, gateway_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdatePaymentGatewayRequest>,
) -> Result<Json<PaymentGatewayResponse>, StatusCode> {
    let gateway = sqlx::query_as::<_, PaymentGateway>(
        r#"
        UPDATE payment_gateways SET
            name = COALESCE($3, name),
            gateway_type = COALESCE($4, gateway_type),
            api_key = COALESCE($5, api_key),
            api_secret = COALESCE($6, api_secret),
            webhook_secret = COALESCE($7, webhook_secret),
            is_active = COALESCE($8, is_active),
            metadata = COALESCE($9, metadata),
            updated_at = NOW()
        WHERE id = $1 AND tenant_id = $2
        RETURNING *
        "#,
    )
    .bind(gateway_id)
    .bind(tenant_id)
    .bind(&payload.name)
    .bind(&payload.gateway_type)
    .bind(&payload.api_key)
    .bind(&payload.api_secret)
    .bind(&payload.webhook_secret)
    .bind(payload.is_active)
    .bind(&payload.metadata)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update payment gateway: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(PaymentGatewayResponse::from(gateway)))
}

pub async fn delete_payment_gateway(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, gateway_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query(
        "DELETE FROM payment_gateways WHERE id = $1 AND tenant_id = $2",
    )
    .bind(gateway_id)
    .bind(tenant_id)
    .execute(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete payment gateway: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}
