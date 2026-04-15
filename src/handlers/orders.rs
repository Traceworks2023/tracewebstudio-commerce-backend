use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{Order, OrderItem, OrderStatus, FinancialStatus, FulfillmentStatus};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub customer_id: Uuid,
    pub subtotal: f64,
    pub tax_amount: f64,
    pub shipping_amount: f64,
    pub discount_amount: f64,
    pub total: f64,
    pub currency: Option<String>,
    pub shipping_address: Option<serde_json::Value>,
    pub billing_address: Option<serde_json::Value>,
    pub shipping_method: Option<String>,
    pub payment_method: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrderStatusRequest {
    pub status: Option<OrderStatus>,
    pub financial_status: Option<FinancialStatus>,
    pub fulfillment_status: Option<FulfillmentStatus>,
}

#[derive(Debug, Serialize)]
pub struct OrderResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub customer_id: Uuid,
    pub order_number: String,
    pub status: OrderStatus,
    pub financial_status: FinancialStatus,
    pub fulfillment_status: FulfillmentStatus,
    pub subtotal: f64,
    pub tax_amount: f64,
    pub shipping_amount: f64,
    pub discount_amount: f64,
    pub total: f64,
    pub currency: String,
    pub shipping_address: Option<serde_json::Value>,
    pub billing_address: Option<serde_json::Value>,
    pub shipping_method: Option<String>,
    pub payment_method: Option<String>,
    pub payment_reference: Option<String>,
    pub notes: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Order> for OrderResponse {
    fn from(o: Order) -> Self {
        Self {
            id: o.id,
            tenant_id: o.tenant_id,
            customer_id: o.customer_id,
            order_number: o.order_number,
            status: o.status,
            financial_status: o.financial_status,
            fulfillment_status: o.fulfillment_status,
            subtotal: o.subtotal,
            tax_amount: o.tax_amount,
            shipping_amount: o.shipping_amount,
            discount_amount: o.discount_amount,
            total: o.total,
            currency: o.currency,
            shipping_address: o.shipping_address,
            billing_address: o.billing_address,
            shipping_method: o.shipping_method,
            payment_method: o.payment_method,
            payment_reference: o.payment_reference,
            notes: o.notes,
            metadata: o.metadata,
            created_at: o.created_at,
            updated_at: o.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListOrdersQuery {
    pub tenant_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub status: Option<OrderStatus>,
    pub financial_status: Option<FinancialStatus>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn create_order(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateOrderRequest>,
) -> Result<(StatusCode, Json<OrderResponse>), StatusCode> {
    let tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let order_number = format!("ORD-{}", uuid::Uuid::new_v4().to_string()[..8].to_uppercase());
    
    let order = sqlx::query_as::<_, Order>(
        r#"
        INSERT INTO orders 
        (tenant_id, customer_id, order_number, subtotal, tax_amount, shipping_amount, discount_amount, total, currency, shipping_address, billing_address, shipping_method, payment_method, notes)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        RETURNING *
        "#,
    )
    .bind(tenant_id)
    .bind(payload.customer_id)
    .bind(&order_number)
    .bind(payload.subtotal)
    .bind(payload.tax_amount)
    .bind(payload.shipping_amount)
    .bind(payload.discount_amount)
    .bind(payload.total)
    .bind(payload.currency.unwrap_or_else(|| "USD".to_string()))
    .bind(&payload.shipping_address)
    .bind(&payload.billing_address)
    .bind(&payload.shipping_method)
    .bind(&payload.payment_method)
    .bind(&payload.notes)
    .fetch_one(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create order: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok((StatusCode::CREATED, Json(OrderResponse::from(order))))
}

pub async fn list_orders(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListOrdersQuery>,
) -> Result<Json<Vec<OrderResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    
    let orders = sqlx::query_as::<_, Order>(
        r#"
        SELECT * FROM orders 
        WHERE tenant_id = $1 
        AND ($2::uuid IS NULL OR customer_id = $2)
        AND ($3::order_status IS NULL OR status = $3)
        AND ($4::financial_status IS NULL OR financial_status = $4)
        ORDER BY created_at DESC
        LIMIT $5 OFFSET $6
        "#,
    )
    .bind(params.tenant_id)
    .bind(params.customer_id)
    .bind(params.status)
    .bind(params.financial_status)
    .bind(limit)
    .bind(offset)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list orders: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(orders.into_iter().map(OrderResponse::from).collect()))
}

pub async fn get_order(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, order_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<OrderResponse>, StatusCode> {
    let order = sqlx::query_as::<_, Order>(
        "SELECT * FROM orders WHERE id = $1 AND tenant_id = $2",
    )
    .bind(order_id)
    .bind(tenant_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get order: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(OrderResponse::from(order)))
}

pub async fn update_order_status(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, order_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateOrderStatusRequest>,
) -> Result<Json<OrderResponse>, StatusCode> {
    let order = sqlx::query_as::<_, Order>(
        r#"
        UPDATE orders SET
            status = COALESCE($3, status),
            financial_status = COALESCE($4, financial_status),
            fulfillment_status = COALESCE($5, fulfillment_status),
            updated_at = NOW()
        WHERE id = $1 AND tenant_id = $2
        RETURNING *
        "#,
    )
    .bind(order_id)
    .bind(tenant_id)
    .bind(payload.status)
    .bind(payload.financial_status)
    .bind(payload.fulfillment_status)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update order status: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(OrderResponse::from(order)))
}

pub async fn get_order_items(
    State(state): State<Arc<AppState>>,
    Path((_tenant_id, order_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<OrderItem>>, StatusCode> {
    let items = sqlx::query_as::<_, OrderItem>(
        "SELECT * FROM order_items WHERE order_id = $1",
    )
    .bind(order_id)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get order items: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(items))
}
