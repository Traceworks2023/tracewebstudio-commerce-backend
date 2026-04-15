use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{Cart, CartItem};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateCartRequest {
    pub customer_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub currency: Option<String>,
    pub metadata: Option<serde_json::Value>,
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

#[derive(Debug, Deserialize)]
pub struct UpdateCartItemRequest {
    pub quantity: Option<i32>,
    pub price: Option<f64>,
    pub tax_amount: Option<f64>,
    pub discount_amount: Option<f64>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct CartItemResponse {
    pub id: Uuid,
    pub cart_id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub name: String,
    pub sku: Option<String>,
    pub price: f64,
    pub quantity: i32,
    pub tax_amount: f64,
    pub discount_amount: f64,
    pub total: f64,
    pub metadata: Option<serde_json::Value>,
}

impl From<CartItem> for CartItemResponse {
    fn from(item: CartItem) -> Self {
        Self {
            id: item.id,
            cart_id: item.cart_id,
            product_id: item.product_id,
            variant_id: item.variant_id,
            name: item.name,
            sku: item.sku,
            price: item.price,
            quantity: item.quantity,
            tax_amount: item.tax_amount,
            discount_amount: item.discount_amount,
            total: item.total,
            metadata: item.metadata,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CartResponse {
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
    pub metadata: Option<serde_json::Value>,
    pub items: Vec<CartItemResponse>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<(Cart, Vec<CartItem>)> for CartResponse {
    fn from((cart, items): (Cart, Vec<CartItem>)) -> Self {
        Self {
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
            metadata: cart.metadata,
            items: items.into_iter().map(CartItemResponse::from).collect(),
            created_at: cart.created_at,
            updated_at: cart.updated_at,
        }
    }
}

pub async fn create_cart(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateCartRequest>,
) -> Result<(StatusCode, Json<CartResponse>), StatusCode> {
    let tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();

    let cart = sqlx::query_as::<_, Cart>(
        r#"
        INSERT INTO carts (tenant_id, customer_id, session_id, currency, metadata)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(tenant_id)
    .bind(payload.customer_id)
    .bind(&payload.session_id)
    .bind(payload.currency.unwrap_or_else(|| "USD".to_string()))
    .bind(&payload.metadata)
    .fetch_one(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create cart: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, Json(CartResponse::from((cart, vec![])))))
}

pub async fn get_cart(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, cart_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<CartResponse>, StatusCode> {
    let cart = sqlx::query_as::<_, Cart>(
        "SELECT * FROM carts WHERE id = $1 AND tenant_id = $2",
    )
    .bind(cart_id)
    .bind(tenant_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get cart: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

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

    Ok(Json(CartResponse::from((cart, items))))
}

pub async fn add_cart_item(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, cart_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<AddCartItemRequest>,
) -> Result<Json<CartResponse>, StatusCode> {
    let cart = sqlx::query_as::<_, Cart>(
        "SELECT * FROM carts WHERE id = $1 AND tenant_id = $2",
    )
    .bind(cart_id)
    .bind(tenant_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get cart: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

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

    Ok(Json(CartResponse::from((updated_cart, items))))
}

pub async fn update_cart_item(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, cart_id, item_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(payload): Json<UpdateCartItemRequest>,
) -> Result<Json<CartItemResponse>, StatusCode> {
    let existing = sqlx::query_as::<_, CartItem>(
        "SELECT * FROM cart_items WHERE id = $1 AND cart_id = $2",
    )
    .bind(item_id)
    .bind(cart_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get cart item: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    let quantity = payload.quantity.unwrap_or(existing.quantity);
    let price = payload.price.unwrap_or(existing.price);
    let tax_amount = payload.tax_amount.unwrap_or(existing.tax_amount);
    let discount_amount = payload.discount_amount.unwrap_or(existing.discount_amount);
    let total = price * quantity as f64 + tax_amount - discount_amount;

    let item = sqlx::query_as::<_, CartItem>(
        r#"
        UPDATE cart_items SET
            quantity = $3,
            price = $4,
            tax_amount = $5,
            discount_amount = $6,
            total = $7,
            metadata = COALESCE($8, metadata)
        WHERE id = $1 AND cart_id = $2
        RETURNING *
        "#,
    )
    .bind(item_id)
    .bind(cart_id)
    .bind(quantity)
    .bind(price)
    .bind(tax_amount)
    .bind(discount_amount)
    .bind(total)
    .bind(&payload.metadata)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update cart item: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(CartItemResponse::from(item)))
}

pub async fn remove_cart_item(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, cart_id, item_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query(
        "DELETE FROM cart_items WHERE id = $1 AND cart_id = $2",
    )
    .bind(item_id)
    .bind(cart_id)
    .execute(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to remove cart item: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn apply_coupon(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, cart_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<CartResponse>, StatusCode> {
    let cart = sqlx::query_as::<_, Cart>(
        "SELECT * FROM carts WHERE id = $1 AND tenant_id = $2",
    )
    .bind(cart_id)
    .bind(tenant_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get cart: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

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

    Ok(Json(CartResponse::from((cart, items))))
}

pub async fn clear_cart(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, cart_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query(
        "DELETE FROM cart_items WHERE cart_id = $1",
    )
    .bind(cart_id)
    .execute(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to clear cart: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(StatusCode::NO_CONTENT)
}
