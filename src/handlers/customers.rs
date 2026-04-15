use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{Customer, CustomerResponse};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateCustomerRequest {
    pub email: String,
    pub phone: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub tags: Option<Vec<String>>,
    pub note: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCustomerRequest {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub tags: Option<Vec<String>>,
    pub note: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListCustomersQuery {
    pub tenant_id: Uuid,
    pub email: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn create_customer(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateCustomerRequest>,
) -> Result<(StatusCode, Json<CustomerResponse>), StatusCode> {
    let tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();

    let customer = sqlx::query_as::<_, Customer>(
        r#"
        INSERT INTO customers (tenant_id, email, phone, first_name, last_name, tags, note, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#,
    )
    .bind(tenant_id)
    .bind(&payload.email)
    .bind(&payload.phone)
    .bind(&payload.first_name)
    .bind(&payload.last_name)
    .bind(&payload.tags)
    .bind(&payload.note)
    .bind(&payload.metadata)
    .fetch_one(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create customer: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, Json(CustomerResponse::from(customer))))
}

pub async fn list_customers(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListCustomersQuery>,
) -> Result<Json<Vec<CustomerResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);

    let customers = sqlx::query_as::<_, Customer>(
        r#"
        SELECT * FROM customers
        WHERE tenant_id = $1
        AND ($2::text IS NULL OR email ILIKE '%' || $2 || '%')
        ORDER BY created_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(params.tenant_id)
    .bind(&params.email)
    .bind(limit)
    .bind(offset)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list customers: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(customers.into_iter().map(CustomerResponse::from).collect()))
}

pub async fn get_customer(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, customer_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<CustomerResponse>, StatusCode> {
    let customer = sqlx::query_as::<_, Customer>(
        "SELECT * FROM customers WHERE id = $1 AND tenant_id = $2",
    )
    .bind(customer_id)
    .bind(tenant_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get customer: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(CustomerResponse::from(customer)))
}

pub async fn update_customer(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, customer_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateCustomerRequest>,
) -> Result<Json<CustomerResponse>, StatusCode> {
    let customer = sqlx::query_as::<_, Customer>(
        r#"
        UPDATE customers SET
            email = COALESCE($3, email),
            phone = COALESCE($4, phone),
            first_name = COALESCE($5, first_name),
            last_name = COALESCE($6, last_name),
            tags = COALESCE($7, tags),
            note = COALESCE($8, note),
            metadata = COALESCE($9, metadata),
            updated_at = NOW()
        WHERE id = $1 AND tenant_id = $2
        RETURNING *
        "#,
    )
    .bind(customer_id)
    .bind(tenant_id)
    .bind(&payload.email)
    .bind(&payload.phone)
    .bind(&payload.first_name)
    .bind(&payload.last_name)
    .bind(&payload.tags)
    .bind(&payload.note)
    .bind(&payload.metadata)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update customer: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(CustomerResponse::from(customer)))
}

pub async fn delete_customer(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, customer_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query(
        "DELETE FROM customers WHERE id = $1 AND tenant_id = $2",
    )
    .bind(customer_id)
    .bind(tenant_id)
    .execute(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete customer: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}
