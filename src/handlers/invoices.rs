use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{Invoice, InvoiceResponse};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateInvoiceRequest {
    pub order_id: Uuid,
    pub invoice_number: Option<String>,
    pub status: Option<String>,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
    pub notes: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateInvoiceRequest {
    pub status: Option<String>,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
    pub notes: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListInvoicesQuery {
    pub tenant_id: Uuid,
    pub order_id: Option<Uuid>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn create_invoice(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateInvoiceRequest>,
) -> Result<(StatusCode, Json<InvoiceResponse>), StatusCode> {
    let tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();

    let invoice = sqlx::query_as::<_, Invoice>(
        r#"
        INSERT INTO invoices (tenant_id, order_id, invoice_number, status, due_date, notes, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#,
    )
    .bind(tenant_id)
    .bind(payload.order_id)
    .bind(&payload.invoice_number)
    .bind(&payload.status.unwrap_or_else(|| "pending".to_string()))
    .bind(payload.due_date)
    .bind(&payload.notes)
    .bind(&payload.metadata)
    .fetch_one(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create invoice: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, Json(InvoiceResponse::from(invoice))))
}

pub async fn list_invoices(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListInvoicesQuery>,
) -> Result<Json<Vec<InvoiceResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);

    let invoices = sqlx::query_as::<_, Invoice>(
        r#"
        SELECT * FROM invoices
        WHERE tenant_id = $1
        AND ($2::uuid IS NULL OR order_id = $2)
        AND ($3::text IS NULL OR status = $3)
        ORDER BY created_at DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(params.tenant_id)
    .bind(params.order_id)
    .bind(&params.status)
    .bind(limit)
    .bind(offset)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list invoices: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(invoices.into_iter().map(InvoiceResponse::from).collect()))
}

pub async fn get_invoice(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, invoice_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<InvoiceResponse>, StatusCode> {
    let invoice = sqlx::query_as::<_, Invoice>(
        "SELECT * FROM invoices WHERE id = $1 AND tenant_id = $2",
    )
    .bind(invoice_id)
    .bind(tenant_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get invoice: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(InvoiceResponse::from(invoice)))
}

pub async fn update_invoice(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, invoice_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateInvoiceRequest>,
) -> Result<Json<InvoiceResponse>, StatusCode> {
    let invoice = sqlx::query_as::<_, Invoice>(
        r#"
        UPDATE invoices SET
            status = COALESCE($3, status),
            due_date = COALESCE($4, due_date),
            notes = COALESCE($5, notes),
            metadata = COALESCE($6, metadata),
            updated_at = NOW()
        WHERE id = $1 AND tenant_id = $2
        RETURNING *
        "#,
    )
    .bind(invoice_id)
    .bind(tenant_id)
    .bind(&payload.status)
    .bind(payload.due_date)
    .bind(&payload.notes)
    .bind(&payload.metadata)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update invoice: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(InvoiceResponse::from(invoice)))
}
