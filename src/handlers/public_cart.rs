use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{Cart, CartItem};
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct PublicCartItemResponse {
    pub id: Uuid,
    pub product_id: Uuid,
    pub name: String,
    pub sku: Option<String>,
    pub price: f64,
    pub quantity: i32,
    pub total: f64,
}

impl From<CartItem> for PublicCartItemResponse {
    fn from(item: CartItem) -> Self {
        Self {
            id: item.id,
            product_id: item.product_id,
            name: item.name,
            sku: item.sku,
            price: item.price,
            quantity: item.quantity,
            total: item.total,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PublicCartResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub currency: String,
    pub subtotal: f64,
    pub tax_amount: f64,
    pub shipping_amount: f64,
    pub discount_amount: f64,
    pub total: f64,
    pub items: Vec<PublicCartItemResponse>,
}

#[derive(Debug, Deserialize)]
pub struct CartQueryParams {
    pub tenant_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct AddCartItemRequest {
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub name: String,
    pub sku: Option<String>,
    pub price: f64,
    pub quantity: i32,
    pub tax_amount: Option<f64>,
    pub discount_amount: Option<f64>,
    pub metadata: Option<serde_json::Value>,
}

pub async fn get_public_cart(
    State(state): State<Arc<AppState>>,
    Query(params): Query<CartQueryParams>,
) -> Result<Json<PublicCartResponse>, StatusCode> {
    let tenant_id = params
        .tenant_id
        .unwrap_or(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap());
    
    let cart = sqlx::query_as::<_, Cart>(
        "SELECT * FROM carts WHERE tenant_id = $1 ORDER BY updated_at DESC LIMIT 1",
    )
    .bind(tenant_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get public cart: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let (cart, items) = if let Some(c) = cart {
        let items = sqlx::query_as::<_, CartItem>(
            "SELECT * FROM cart_items WHERE cart_id = $1",
        )
        .bind(c.id)
        .fetch_all(&*state.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get cart items: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        (c, items)
    } else {
        let new_cart = sqlx::query_as::<_, Cart>(
            r#"
            INSERT INTO carts (tenant_id, currency, subtotal, tax_amount, shipping_amount, discount_amount, total)
            VALUES ($1, 'INR', 0, 0, 0, 0, 0)
            RETURNING *
            "#,
        )
        .bind(tenant_id)
        .fetch_one(&*state.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create cart: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        (new_cart, vec![])
    };
    
    let items_response: Vec<PublicCartItemResponse> = items
        .into_iter()
        .map(PublicCartItemResponse::from)
        .collect();
    
    let cart_response = PublicCartResponse {
        id: cart.id,
        tenant_id: cart.tenant_id,
        customer_id: cart.customer_id,
        session_id: cart.session_id,
        currency: cart.currency,
        subtotal: cart.subtotal,
        tax_amount: cart.tax_amount,
        shipping_amount: cart.shipping_amount,
        discount_amount: cart.discount_amount,
        total: cart.total,
        items: items_response,
    };
    
    Ok(Json(cart_response))
}

pub async fn add_to_public_cart(
    State(state): State<Arc<AppState>>,
    Query(params): Query<CartQueryParams>,
    Json(payload): Json<AddCartItemRequest>,
) -> Result<Json<PublicCartResponse>, StatusCode> {
    let tenant_id = params
        .tenant_id
        .unwrap_or(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap());
    
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
    
    let cart_id = if let Some(c) = cart {
        c.id
    } else {
        let new_cart = sqlx::query_as::<_, Cart>(
            r#"
            INSERT INTO carts (tenant_id, currency, subtotal, tax_amount, shipping_amount, discount_amount, total)
            VALUES ($1, 'INR', 0, 0, 0, 0, 0)
            RETURNING *
            "#,
        )
        .bind(tenant_id)
        .fetch_one(&*state.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create cart: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        new_cart.id
    };
    
    let tax_amount = payload.tax_amount.unwrap_or(0.0);
    let discount_amount = payload.discount_amount.unwrap_or(0.0);
    let total = payload.price * payload.quantity as f64 + tax_amount - discount_amount;
    
    let _item = sqlx::query_as::<_, CartItem>(
        r#"
        INSERT INTO cart_items (cart_id, product_id, variant_id, name, sku, price, quantity, tax_amount, discount_amount, total, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING *
        "#,
    )
    .bind(cart_id)
    .bind(payload.product_id)
    .bind(payload.variant_id)
    .bind(&payload.name)
    .bind(&payload.sku)
    .bind(payload.price)
    .bind(payload.quantity)
    .bind(tax_amount)
    .bind(discount_amount)
    .bind(total)
    .bind(&payload.metadata)
    .fetch_one(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to add cart item: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let items = sqlx::query_as::<_, CartItem>(
        "SELECT * FROM cart_items WHERE cart_id = $1",
    )
    .bind(cart_id)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get cart items: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let updated_cart = sqlx::query_as::<_, Cart>(
        "SELECT * FROM carts WHERE id = $1",
    )
    .bind(cart_id)
    .fetch_one(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get updated cart: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let items_response: Vec<PublicCartItemResponse> = items
        .into_iter()
        .map(PublicCartItemResponse::from)
        .collect();
    
    let cart_response = PublicCartResponse {
        id: updated_cart.id,
        tenant_id: updated_cart.tenant_id,
        customer_id: updated_cart.customer_id,
        session_id: updated_cart.session_id,
        currency: updated_cart.currency,
        subtotal: updated_cart.subtotal,
        tax_amount: updated_cart.tax_amount,
        shipping_amount: updated_cart.shipping_amount,
        discount_amount: updated_cart.discount_amount,
        total: updated_cart.total,
        items: items_response,
    };
    
    Ok(Json(cart_response))
}
