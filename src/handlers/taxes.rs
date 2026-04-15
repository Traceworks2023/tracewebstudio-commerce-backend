use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{Tax, TaxResponse};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateTaxRequest {
    pub name: String,
    pub rate: f64,
    pub tax_type: Option<String>,
    pub is_active: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaxRequest {
    pub name: Option<String>,
    pub rate: Option<f64>,
    pub tax_type: Option<String>,
    pub is_active: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListTaxesQuery {
    pub tenant_id: Uuid,
    pub is_active: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn create_tax(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTaxRequest>,
) -> Result<(StatusCode, Json<TaxResponse>), StatusCode> {
    let tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();

    let tax = sqlx::query_as::<_, Tax>(
        r#"
        INSERT INTO taxes (tenant_id, name, rate, tax_type, is_active, metadata)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(tenant_id)
    .bind(&payload.name)
    .bind(payload.rate)
    .bind(&payload.tax_type)
    .bind(payload.is_active.unwrap_or(true))
    .bind(&payload.metadata)
    .fetch_one(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create tax: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, Json(TaxResponse::from(tax))))
}

pub async fn list_taxes(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListTaxesQuery>,
) -> Result<Json<Vec<TaxResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);

    let taxes = sqlx::query_as::<_, Tax>(
        r#"
        SELECT * FROM taxes
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
        tracing::error!("Failed to list taxes: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(taxes.into_iter().map(TaxResponse::from).collect()))
}

pub async fn get_tax(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, tax_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<TaxResponse>, StatusCode> {
    let tax = sqlx::query_as::<_, Tax>(
        "SELECT * FROM taxes WHERE id = $1 AND tenant_id = $2",
    )
    .bind(tax_id)
    .bind(tenant_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get tax: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(TaxResponse::from(tax)))
}

pub async fn update_tax(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, tax_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateTaxRequest>,
) -> Result<Json<TaxResponse>, StatusCode> {
    let tax = sqlx::query_as::<_, Tax>(
        r#"
        UPDATE taxes SET
            name = COALESCE($3, name),
            rate = COALESCE($4, rate),
            tax_type = COALESCE($5, tax_type),
            is_active = COALESCE($6, is_active),
            metadata = COALESCE($7, metadata),
            updated_at = NOW()
        WHERE id = $1 AND tenant_id = $2
        RETURNING *
        "#,
    )
    .bind(tax_id)
    .bind(tenant_id)
    .bind(&payload.name)
    .bind(payload.rate)
    .bind(&payload.tax_type)
    .bind(payload.is_active)
    .bind(&payload.metadata)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update tax: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(TaxResponse::from(tax)))
}

pub async fn delete_tax(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, tax_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query(
        "DELETE FROM taxes WHERE id = $1 AND tenant_id = $2",
    )
    .bind(tax_id)
    .bind(tenant_id)
    .execute(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete tax: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}
