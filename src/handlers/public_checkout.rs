use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{Cart, Order};
use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckoutRequest {
    pub shipping_address: serde_json::Value,
    pub billing_address: serde_json::Value,
    pub payment_method: String,
    pub shipping_method: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CheckoutResponse {
    pub order_id: Uuid,
    pub order_number: String,
    pub status: String,
    pub total: f64,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct CheckoutQuery {
    pub tenant_id: Option<String>,
}

pub async fn place_public_order(
    State(state): State<Arc<AppState>>,
    Query(params): Query<CheckoutQuery>,
    Json(payload): Json<CheckoutRequest>,
) -> Result<Json<CheckoutResponse>, StatusCode> {
    let tenant_id_str = params.tenant_id.as_deref().unwrap_or("00000000-0000-0000-0000-000000000001");
    let tenant_id = Uuid::parse_str(tenant_id_str).map_err(|_| {
        tracing::error!("Invalid tenant_id format");
        StatusCode::BAD_REQUEST
    })?;
    
    let cart = sqlx::query_as::<_, Cart>(
        "SELECT * FROM carts WHERE tenant_id = $1 ORDER BY updated_at DESC LIMIT 1",
    )
    .bind(tenant_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get cart: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let cart = if let Some(c) = cart {
        c
    } else {
        return Ok(Json(CheckoutResponse {
            order_id: Uuid::new_v4(),
            order_number: "NONE".to_string(),
            status: "failed".to_string(),
            total: 0.0,
            message: "Cart is empty".to_string(),
        }));
    };
    
    let order_number = format!("ORD-{}", chrono::Utc::now().timestamp());
    
    let order = sqlx::query_as::<_, Order>(
        r#"
        INSERT INTO orders (tenant_id, customer_id, order_number, status, financial_status, fulfillment_status, subtotal, tax_amount, shipping_amount, discount_amount, total, currency, shipping_address, billing_address, shipping_method, payment_method)
        VALUES ($1, $2, $3, 'pending', 'pending', 'unfulfilled', $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        RETURNING *
        "#,
    )
    .bind(tenant_id)
    .bind(Uuid::nil())
    .bind(&order_number)
    .bind(cart.subtotal)
    .bind(cart.tax_amount)
    .bind(cart.shipping_amount)
    .bind(cart.discount_amount)
    .bind(cart.total)
    .bind(&cart.currency)
    .bind(&payload.shipping_address)
    .bind(&payload.billing_address)
    .bind(payload.shipping_method.unwrap_or_else(|| "standard".to_string()))
    .bind(&payload.payment_method)
    .fetch_one(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create order: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    sqlx::query("DELETE FROM cart_items WHERE cart_id = $1")
        .bind(cart.id)
        .execute(&*state.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to clear cart items: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(Json(CheckoutResponse {
        order_id: order.id,
        order_number: order.order_number,
        status: format!("{:?}", order.status),
        total: order.total,
        message: "Order created successfully".to_string(),
    }))
}
