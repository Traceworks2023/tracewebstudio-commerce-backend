use axum::{
    extract::{Query, Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{Product, Category, Order, Customer, Coupon, Discount, Tax, ShippingRate, Invoice, PaymentGateway, Cart, CartItem};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct TenantListQuery {
    pub tenant_id: Option<Uuid>,
    pub status: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct TenantProductResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub price: f64,
    pub sku: Option<String>,
    pub status: String,
    pub inventory_quantity: i32,
    pub category_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Product> for TenantProductResponse {
    fn from(p: Product) -> Self {
        Self {
            id: p.id,
            tenant_id: p.tenant_id,
            name: p.name,
            slug: p.slug,
            description: p.description,
            price: p.price,
            sku: p.sku,
            status: if p.is_active { "active".to_string() } else { "draft".to_string() },
            inventory_quantity: p.inventory_quantity,
            category_id: p.category_id,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

pub async fn tenant_list_products(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TenantListQuery>,
) -> Result<Json<Vec<TenantProductResponse>>, StatusCode> {
    let tenant_id = params.tenant_id.unwrap_or(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap());
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    
    let products = sqlx::query_as::<_, Product>(
        "SELECT * FROM products WHERE tenant_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(tenant_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list products: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(products.into_iter().map(TenantProductResponse::from).collect()))
}

pub async fn tenant_get_product(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, product_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<TenantProductResponse>, StatusCode> {
    let product = sqlx::query_as::<_, Product>(
        "SELECT * FROM products WHERE id = $1 AND tenant_id = $2",
    )
    .bind(product_id)
    .bind(tenant_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get product: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(TenantProductResponse::from(product)))
}

pub async fn tenant_create_product(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<TenantProductResponse>, StatusCode> {
    let tenant_id = payload.get("tenant_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap());
    
    let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let slug = payload.get("slug").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let price = payload.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
    
    let product = sqlx::query_as::<_, Product>(
        r#"
        INSERT INTO products (tenant_id, name, slug, price, is_active)
        VALUES ($1, $2, $3, $4, true)
        RETURNING *
        "#,
    )
    .bind(tenant_id)
    .bind(&name)
    .bind(&slug)
    .bind(price)
    .fetch_one(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create product: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(TenantProductResponse::from(product)))
}

pub async fn tenant_update_product(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, product_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<TenantProductResponse>, StatusCode> {
    let product = sqlx::query_as::<_, Product>(
        "SELECT * FROM products WHERE id = $1 AND tenant_id = $2",
    )
    .bind(product_id)
    .bind(tenant_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update product: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;
    
    let updated = sqlx::query_as::<_, Product>(
        r#"
        UPDATE products SET
            name = COALESCE($3, name),
            slug = COALESCE($4, slug),
            price = COALESCE($5, price),
            updated_at = NOW()
        WHERE id = $1 AND tenant_id = $2
        RETURNING *
        "#,
    )
    .bind(product_id)
    .bind(tenant_id)
    .bind(payload.get("name").and_then(|v| v.as_str()))
    .bind(payload.get("slug").and_then(|v| v.as_str()))
    .bind(payload.get("price").and_then(|v| v.as_f64()))
    .fetch_one(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update product: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(TenantProductResponse::from(updated)))
}

pub async fn tenant_delete_product(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, product_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query(
        "DELETE FROM products WHERE id = $1 AND tenant_id = $2",
    )
    .bind(product_id)
    .bind(tenant_id)
    .execute(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete product: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    
    Ok(StatusCode::NO_CONTENT)
}

pub async fn tenant_list_categories(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TenantListQuery>,
) -> Result<Json<Vec<Category>>, StatusCode> {
    let tenant_id = params.tenant_id.unwrap_or(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap());
    
    let categories = sqlx::query_as::<_, Category>(
        "SELECT * FROM categories WHERE tenant_id = $1 ORDER BY sort_order ASC",
    )
    .bind(tenant_id)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list categories: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(categories))
}

pub async fn tenant_list_orders(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TenantListQuery>,
) -> Result<Json<Vec<Order>>, StatusCode> {
    let tenant_id = params.tenant_id.unwrap_or(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap());
    
    let orders = sqlx::query_as::<_, Order>(
        "SELECT * FROM orders WHERE tenant_id = $1 ORDER BY created_at DESC",
    )
    .bind(tenant_id)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list orders: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(orders))
}

pub async fn tenant_list_promotions(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TenantListQuery>,
) -> Result<Json<Vec<Coupon>>, StatusCode> {
    let tenant_id = params.tenant_id.unwrap_or(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap());
    
    let coupons = sqlx::query_as::<_, Coupon>(
        "SELECT * FROM coupons WHERE tenant_id = $1 ORDER BY created_at DESC",
    )
    .bind(tenant_id)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list promotions: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(coupons))
}

#[derive(Debug, Serialize)]
pub struct TenantCartResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub items: Vec<CartItemResponse>,
    pub subtotal: f64,
    pub tax_amount: f64,
    pub shipping_amount: f64,
    pub discount_amount: f64,
    pub total: f64,
    pub currency: String,
}

#[derive(Debug, Serialize)]
pub struct CartItemResponse {
    pub id: Uuid,
    pub product_id: Uuid,
    pub product_name: String,
    pub quantity: i32,
    pub price: f64,
    pub total: f64,
}

pub async fn tenant_get_cart(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TenantListQuery>,
) -> Result<Json<TenantCartResponse>, StatusCode> {
    let tenant_id = params.tenant_id.unwrap_or(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap());
    
    let cart = sqlx::query_as::<_, Cart>(
        "SELECT * FROM carts WHERE tenant_id = $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(tenant_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get cart: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    if let Some(cart) = cart {
        let items = sqlx::query_as::<_, CartItem>(
            "SELECT * FROM cart_items WHERE cart_id = $1",
        )
        .bind(cart.id)
        .fetch_all(&*state.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get cart items: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        
        let item_responses: Vec<CartItemResponse> = items
            .into_iter()
            .map(|i| CartItemResponse {
                id: i.id,
                product_id: i.product_id,
                product_name: i.name,
                quantity: i.quantity,
                price: i.price,
                total: i.total,
            })
            .collect();
        
        Ok(Json(TenantCartResponse {
            id: cart.id,
            tenant_id: cart.tenant_id,
            items: item_responses,
            subtotal: cart.subtotal,
            tax_amount: cart.tax_amount,
            shipping_amount: cart.shipping_amount,
            discount_amount: cart.discount_amount,
            total: cart.total,
            currency: cart.currency,
        }))
    } else {
        Ok(Json(TenantCartResponse {
            id: Uuid::new_v4(),
            tenant_id,
            items: vec![],
            subtotal: 0.0,
            tax_amount: 0.0,
            shipping_amount: 0.0,
            discount_amount: 0.0,
            total: 0.0,
            currency: "INR".to_string(),
        }))
    }
}

#[derive(Debug, Deserialize)]
pub struct AddCartItemInput {
    pub product_id: Uuid,
    pub quantity: i32,
    pub price: f64,
}

pub async fn tenant_add_cart_item(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TenantListQuery>,
    Json(payload): Json<AddCartItemInput>,
) -> Result<Json<TenantCartResponse>, StatusCode> {
    let tenant_id = params.tenant_id.unwrap_or(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap());
    
    let cart = sqlx::query_as::<_, Cart>(
        "SELECT * FROM carts WHERE tenant_id = $1 ORDER BY created_at DESC LIMIT 1",
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
    
    let total = payload.price * payload.quantity as f64;
    
    sqlx::query(
        r#"
        INSERT INTO cart_items (cart_id, product_id, name, price, quantity, total)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(cart_id)
    .bind(payload.product_id)
    .bind(format!("Product {}", payload.product_id))
    .bind(payload.price)
    .bind(payload.quantity)
    .bind(total)
    .execute(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to add cart item: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    tenant_get_cart(State(state), Query(params)).await
}

#[derive(Debug, Deserialize)]
pub struct CheckoutInput {
    pub shipping_address: serde_json::Value,
    pub billing_address: serde_json::Value,
    pub payment_method: String,
}

#[derive(Debug, Serialize)]
pub struct CheckoutResponse {
    pub order_id: Uuid,
    pub order_number: String,
    pub status: String,
    pub total: f64,
    pub message: String,
}

pub async fn tenant_checkout(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TenantListQuery>,
    Json(payload): Json<CheckoutInput>,
) -> Result<Json<CheckoutResponse>, StatusCode> {
    let tenant_id = params.tenant_id.unwrap_or(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap());
    
    let cart = sqlx::query_as::<_, Cart>(
        "SELECT * FROM carts WHERE tenant_id = $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(tenant_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get cart: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    if cart.is_none() {
        return Ok(Json(CheckoutResponse {
            order_id: Uuid::new_v4(),
            order_number: "NONE".to_string(),
            status: "failed".to_string(),
            total: 0.0,
            message: "Cart is empty".to_string(),
        }));
    }
    
    let cart = cart.unwrap();
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
    .bind("standard")
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

pub async fn tenant_create_category(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Category>, StatusCode> {
    let tenant_id = payload.get("tenant_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap());
    
    let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let slug = payload.get("slug").and_then(|v| v.as_str()).unwrap_or("").to_string();
    
    let category = sqlx::query_as::<_, Category>(
        r#"
        INSERT INTO categories (tenant_id, name, slug, is_active)
        VALUES ($1, $2, $3, true)
        RETURNING *
        "#,
    )
    .bind(tenant_id)
    .bind(&name)
    .bind(&slug)
    .fetch_one(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create category: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(category))
}

pub async fn tenant_update_category(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, category_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Category>, StatusCode> {
    let category = sqlx::query_as::<_, Category>(
        "SELECT * FROM categories WHERE id = $1 AND tenant_id = $2",
    )
    .bind(category_id)
    .bind(tenant_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update category: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;
    
    let updated = sqlx::query_as::<_, Category>(
        r#"
        UPDATE categories SET
            name = COALESCE($3, name),
            slug = COALESCE($4, slug),
            updated_at = NOW()
        WHERE id = $1 AND tenant_id = $2
        RETURNING *
        "#,
    )
    .bind(category_id)
    .bind(tenant_id)
    .bind(payload.get("name").and_then(|v| v.as_str()))
    .bind(payload.get("slug").and_then(|v| v.as_str()))
    .fetch_one(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update category: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(updated))
}

pub async fn tenant_delete_category(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, category_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query(
        "DELETE FROM categories WHERE id = $1 AND tenant_id = $2",
    )
    .bind(category_id)
    .bind(tenant_id)
    .execute(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete category: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    
    Ok(StatusCode::NO_CONTENT)
}

pub async fn tenant_create_promotion(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Coupon>, StatusCode> {
    let tenant_id = payload.get("tenant_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap());
    
    let code = payload.get("code").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let discount_type = payload.get("discount_type").and_then(|v| v.as_str()).unwrap_or("percentage");
    let value = payload.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
    
    let coupon = sqlx::query_as::<_, Coupon>(
        r#"
        INSERT INTO coupons (tenant_id, code, coupon_type, value, is_active)
        VALUES ($1, $2, 'percentage', $3, true)
        RETURNING *
        "#,
    )
    .bind(tenant_id)
    .bind(&code)
    .bind(value)
    .fetch_one(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create promotion: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(coupon))
}

pub async fn tenant_update_promotion(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, promotion_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Coupon>, StatusCode> {
    let coupon = sqlx::query_as::<_, Coupon>(
        "SELECT * FROM coupons WHERE id = $1 AND tenant_id = $2",
    )
    .bind(promotion_id)
    .bind(tenant_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update promotion: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;
    
    let updated = sqlx::query_as::<_, Coupon>(
        r#"
        UPDATE coupons SET
            code = COALESCE($3, code),
            value = COALESCE($4, value),
            updated_at = NOW()
        WHERE id = $1 AND tenant_id = $2
        RETURNING *
        "#,
    )
    .bind(promotion_id)
    .bind(tenant_id)
    .bind(payload.get("code").and_then(|v| v.as_str()))
    .bind(payload.get("value").and_then(|v| v.as_f64()))
    .fetch_one(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update promotion: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(updated))
}

pub async fn tenant_delete_promotion(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, promotion_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query(
        "DELETE FROM coupons WHERE id = $1 AND tenant_id = $2",
    )
    .bind(promotion_id)
    .bind(tenant_id)
    .execute(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete promotion: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    
    Ok(StatusCode::NO_CONTENT)
}