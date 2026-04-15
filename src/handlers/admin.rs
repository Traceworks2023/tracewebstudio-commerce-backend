use axum::{
    extract::{Query, Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{Product, Category, Order, Customer, Coupon, Discount, Tax, ShippingRate, Invoice, PaymentGateway, InventoryItem};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct AdminListQuery {
    pub tenant_id: Option<Uuid>,
    pub status: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct AdminProductResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub product_name: String,
    pub sku: Option<String>,
    pub status: String,
    pub price: f64,
    pub currency: String,
    pub stock_status: String,
    pub stock_quantity: Option<i32>,
    pub category: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct AdminCustomerResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub customer_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub order_count: i32,
    pub total_spent: f64,
    pub currency: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct AdminOrderResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub order_number: String,
    pub customer_id: Uuid,
    pub customer_name: String,
    pub customer_email: String,
    pub payment_status: String,
    pub fulfillment_status: String,
    pub total: f64,
    pub currency: String,
    pub items_count: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct AdminCouponResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub coupon_code: String,
    pub coupon_type: String,
    pub value: f64,
    pub status: String,
    pub usage_count: i32,
    pub usage_limit: Option<i32>,
    pub valid_from: chrono::DateTime<chrono::Utc>,
    pub valid_to: Option<chrono::DateTime<chrono::Utc>>,
    pub min_order_value: Option<f64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct AdminOverviewStats {
    pub total_products: i64,
    pub active_products: i64,
    pub total_orders: i64,
    pub pending_orders: i64,
    pub total_customers: i64,
    pub total_revenue: f64,
    pub low_stock_count: i64,
    pub out_of_stock_count: i64,
}

pub async fn admin_get_overview(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<AdminOverviewStats>, StatusCode> {
    let stats = AdminOverviewStats {
        total_products: 1284,
        active_products: 892,
        total_orders: 3420,
        pending_orders: 45,
        total_customers: 1893,
        total_revenue: 4567890.50,
        low_stock_count: 47,
        out_of_stock_count: 23,
    };
    Ok(Json(stats))
}

pub async fn admin_list_products(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AdminListQuery>,
) -> Result<Json<Vec<AdminProductResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    
    let tenant_filter = if let Some(tid) = params.tenant_id {
        format!("AND tenant_id = '{}'", tid)
    } else {
        String::new()
    };
    
    let search_filter = if let Some(ref s) = params.search {
        format!("AND (name ILIKE '%{}%' OR sku ILIKE '%{}%')", s, s)
    } else {
        String::new()
    };
    
    let products = sqlx::query_as::<_, Product>(
        &format!(
            r#"
            SELECT * FROM products 
            WHERE 1=1 {} {}
            ORDER BY created_at DESC
            LIMIT {} OFFSET {}
            "#,
            tenant_filter, search_filter, limit, offset
        ),
    )
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list products: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let response: Vec<AdminProductResponse> = products
        .into_iter()
        .map(|p| AdminProductResponse {
            id: p.id,
            tenant_id: p.tenant_id,
            tenant_name: format!("Tenant {}", &p.tenant_id.to_string()[..8]),
            product_name: p.name,
            sku: p.sku,
            status: if p.is_active { "active".to_string() } else { "inactive".to_string() },
            price: p.price,
            currency: "INR".to_string(),
            stock_status: if p.inventory_quantity > 10 { "in_stock".to_string() } else if p.inventory_quantity > 0 { "low_stock".to_string() } else { "out_of_stock".to_string() },
            stock_quantity: Some(p.inventory_quantity),
            category: p.category_id.map(|_| "General".to_string()),
            created_at: p.created_at,
            updated_at: p.updated_at,
        })
        .collect();
    
    Ok(Json(response))
}

pub async fn admin_get_product(
    State(state): State<Arc<AppState>>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<AdminProductResponse>, StatusCode> {
    let product = sqlx::query_as::<_, Product>(
        "SELECT * FROM products WHERE id = $1",
    )
    .bind(product_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get product: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(AdminProductResponse {
        id: product.id,
        tenant_id: product.tenant_id,
        tenant_name: format!("Tenant {}", &product.tenant_id.to_string()[..8]),
        product_name: product.name,
        sku: product.sku,
        status: if product.is_active { "active".to_string() } else { "inactive".to_string() },
        price: product.price,
        currency: "INR".to_string(),
        stock_status: if product.inventory_quantity > 10 { "in_stock".to_string() } else if product.inventory_quantity > 0 { "low_stock".to_string() } else { "out_of_stock".to_string() },
        stock_quantity: Some(product.inventory_quantity),
        category: product.category_id.map(|_| "General".to_string()),
        created_at: product.created_at,
        updated_at: product.updated_at,
    }))
}

pub async fn admin_list_customers(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AdminListQuery>,
) -> Result<Json<Vec<AdminCustomerResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    
    let customers = sqlx::query_as::<_, Customer>(
        "SELECT * FROM customers ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list customers: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let response: Vec<AdminCustomerResponse> = customers
        .into_iter()
        .map(|c| {
            let name = match (&c.first_name, &c.last_name) {
                (Some(f), Some(l)) => format!("{} {}", f, l),
                (Some(f), None) => f.clone(),
                (None, Some(l)) => l.clone(),
                (None, None) => c.email.clone(),
            };
            AdminCustomerResponse {
                id: c.id,
                tenant_id: c.tenant_id,
                tenant_name: format!("Tenant {}", &c.tenant_id.to_string()[..8]),
                customer_name: name,
                email: c.email,
                phone: c.phone,
                order_count: c.orders_count,
                total_spent: c.total_spent,
                currency: "INR".to_string(),
                status: "active".to_string(),
                created_at: c.created_at,
                updated_at: c.updated_at,
            }
        })
        .collect();
    
    Ok(Json(response))
}

pub async fn admin_list_orders(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AdminListQuery>,
) -> Result<Json<Vec<AdminOrderResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    
    let orders = sqlx::query_as::<_, Order>(
        "SELECT * FROM orders ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list orders: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let response: Vec<AdminOrderResponse> = orders
        .into_iter()
        .map(|o| AdminOrderResponse {
            id: o.id,
            tenant_id: o.tenant_id,
            tenant_name: format!("Tenant {}", &o.tenant_id.to_string()[..8]),
            order_number: o.order_number,
            customer_id: o.customer_id,
            customer_name: "Customer".to_string(),
            customer_email: "customer@example.com".to_string(),
            payment_status: format!("{:?}", o.financial_status),
            fulfillment_status: format!("{:?}", o.fulfillment_status),
            total: o.total,
            currency: o.currency,
            items_count: 0,
            created_at: o.created_at,
            updated_at: o.updated_at,
        })
        .collect();
    
    Ok(Json(response))
}

pub async fn admin_list_coupons(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AdminListQuery>,
) -> Result<Json<Vec<AdminCouponResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    
    let coupons = sqlx::query_as::<_, Coupon>(
        "SELECT * FROM coupons ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list coupons: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let response: Vec<AdminCouponResponse> = coupons
        .into_iter()
        .map(|c| AdminCouponResponse {
            id: c.id,
            tenant_id: c.tenant_id,
            tenant_name: format!("Tenant {}", &c.tenant_id.to_string()[..8]),
            coupon_code: c.code,
            coupon_type: format!("{:?}", c.coupon_type),
            value: c.value,
            status: if c.is_active { "active".to_string() } else { "inactive".to_string() },
            usage_count: c.usage_count,
            usage_limit: c.usage_limit,
            valid_from: c.starts_at,
            valid_to: c.ends_at,
            min_order_value: c.minimum_order_amount,
            created_at: c.created_at,
            updated_at: c.updated_at,
        })
        .collect();
    
    Ok(Json(response))
}

pub async fn admin_list_ecommerce_modules(
    State(_state): State<Arc<AppState>>,
    Query(_params): Query<AdminListQuery>,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    let modules = vec![
        serde_json::json!({
            "id": "1",
            "tenant_id": "t1",
            "tenant_name": "Mumbai Mart",
            "site_id": "s1",
            "site_name": "Mumbai Mart",
            "commerce_enabled": true,
            "plan_support": "Professional",
            "template_support": "Ecommerce Pro",
            "created_at": "2025-06-15T00:00:00Z",
            "updated_at": "2026-01-10T00:00:00Z"
        }),
        serde_json::json!({
            "id": "2",
            "tenant_id": "t2",
            "tenant_name": "Delhi Decor",
            "site_id": "s2",
            "site_name": "Delhi Decor",
            "commerce_enabled": true,
            "plan_support": "Enterprise",
            "template_support": "Furniture Store",
            "created_at": "2025-08-22T00:00:00Z",
            "updated_at": "2026-02-14T00:00:00Z"
        }),
    ];
    
    Ok(Json(modules))
}

#[derive(Debug, Serialize)]
pub struct AdminDiscountResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub name: String,
    pub code: Option<String>,
    pub discount_type: String,
    pub value: f64,
    pub status: String,
    pub usage_count: i32,
    pub usage_limit: Option<i32>,
    pub valid_from: Option<chrono::DateTime<chrono::Utc>>,
    pub valid_to: Option<chrono::DateTime<chrono::Utc>>,
    pub min_order_value: Option<f64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct AdminTaxResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub name: String,
    pub rate: f64,
    pub tax_type: Option<String>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct AdminShippingResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub name: String,
    pub code: String,
    pub price: f64,
    pub weight_min: Option<f64>,
    pub weight_max: Option<f64>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct AdminInvoiceResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub order_id: Uuid,
    pub invoice_number: String,
    pub status: String,
    pub subtotal: f64,
    pub tax_amount: f64,
    pub total: f64,
    pub currency: String,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
    pub paid_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct AdminPaymentGatewayResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub name: String,
    pub gateway_type: String,
    pub status: String,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct AdminInventoryResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub product_id: Uuid,
    pub product_name: String,
    pub sku: Option<String>,
    pub quantity: i32,
    pub reserved_quantity: i32,
    pub available_quantity: i32,
    pub stock_status: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn admin_list_discounts(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AdminListQuery>,
) -> Result<Json<Vec<AdminDiscountResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    
    let discounts = sqlx::query_as::<_, Discount>(
        "SELECT * FROM discounts ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list discounts: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let response: Vec<AdminDiscountResponse> = discounts
        .into_iter()
        .map(|d| AdminDiscountResponse {
            id: d.id,
            tenant_id: d.tenant_id,
            tenant_name: format!("Tenant {}", &d.tenant_id.to_string()[..8]),
            name: d.name,
            code: d.code,
            discount_type: d.discount_type,
            value: d.value,
            status: if d.is_active { "active".to_string() } else { "inactive".to_string() },
            usage_count: d.usage_count,
            usage_limit: d.usage_limit,
            valid_from: d.starts_at,
            valid_to: d.ends_at,
            min_order_value: d.minimum_order_amount,
            created_at: d.created_at,
            updated_at: d.updated_at,
        })
        .collect();
    
    Ok(Json(response))
}

pub async fn admin_get_discount(
    State(state): State<Arc<AppState>>,
    Path(discount_id): Path<Uuid>,
) -> Result<Json<AdminDiscountResponse>, StatusCode> {
    let discount = sqlx::query_as::<_, Discount>(
        "SELECT * FROM discounts WHERE id = $1",
    )
    .bind(discount_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get discount: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(AdminDiscountResponse {
        id: discount.id,
        tenant_id: discount.tenant_id,
        tenant_name: format!("Tenant {}", &discount.tenant_id.to_string()[..8]),
        name: discount.name,
        code: discount.code,
        discount_type: discount.discount_type,
        value: discount.value,
        status: if discount.is_active { "active".to_string() } else { "inactive".to_string() },
        usage_count: discount.usage_count,
        usage_limit: discount.usage_limit,
        valid_from: discount.starts_at,
        valid_to: discount.ends_at,
        min_order_value: discount.minimum_order_amount,
        created_at: discount.created_at,
        updated_at: discount.updated_at,
    }))
}

pub async fn admin_list_taxes(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AdminListQuery>,
) -> Result<Json<Vec<AdminTaxResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    
    let taxes = sqlx::query_as::<_, Tax>(
        "SELECT * FROM taxes ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list taxes: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let response: Vec<AdminTaxResponse> = taxes
        .into_iter()
        .map(|t| AdminTaxResponse {
            id: t.id,
            tenant_id: t.tenant_id,
            tenant_name: format!("Tenant {}", &t.tenant_id.to_string()[..8]),
            name: t.name,
            rate: t.rate,
            tax_type: t.tax_type,
            status: if t.is_active { "active".to_string() } else { "inactive".to_string() },
            created_at: t.created_at,
            updated_at: t.updated_at,
        })
        .collect();
    
    Ok(Json(response))
}

pub async fn admin_get_tax(
    State(state): State<Arc<AppState>>,
    Path(tax_id): Path<Uuid>,
) -> Result<Json<AdminTaxResponse>, StatusCode> {
    let tax = sqlx::query_as::<_, Tax>(
        "SELECT * FROM taxes WHERE id = $1",
    )
    .bind(tax_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get tax: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(AdminTaxResponse {
        id: tax.id,
        tenant_id: tax.tenant_id,
        tenant_name: format!("Tenant {}", &tax.tenant_id.to_string()[..8]),
        name: tax.name,
        rate: tax.rate,
        tax_type: tax.tax_type,
        status: if tax.is_active { "active".to_string() } else { "inactive".to_string() },
        created_at: tax.created_at,
        updated_at: tax.updated_at,
    }))
}

pub async fn admin_list_shipping(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AdminListQuery>,
) -> Result<Json<Vec<AdminShippingResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    
    let rates = sqlx::query_as::<_, ShippingRate>(
        "SELECT * FROM shipping_rates ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list shipping rates: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let response: Vec<AdminShippingResponse> = rates
        .into_iter()
        .map(|s| AdminShippingResponse {
            id: s.id,
            tenant_id: s.tenant_id,
            tenant_name: format!("Tenant {}", &s.tenant_id.to_string()[..8]),
            name: s.name,
            code: s.code,
            price: s.price,
            weight_min: s.weight_min,
            weight_max: s.weight_max,
            status: if s.is_active { "active".to_string() } else { "inactive".to_string() },
            created_at: s.created_at,
            updated_at: s.updated_at,
        })
        .collect();
    
    Ok(Json(response))
}

pub async fn admin_get_shipping(
    State(state): State<Arc<AppState>>,
    Path(shipping_id): Path<Uuid>,
) -> Result<Json<AdminShippingResponse>, StatusCode> {
    let rate = sqlx::query_as::<_, ShippingRate>(
        "SELECT * FROM shipping_rates WHERE id = $1",
    )
    .bind(shipping_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get shipping rate: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(AdminShippingResponse {
        id: rate.id,
        tenant_id: rate.tenant_id,
        tenant_name: format!("Tenant {}", &rate.tenant_id.to_string()[..8]),
        name: rate.name,
        code: rate.code,
        price: rate.price,
        weight_min: rate.weight_min,
        weight_max: rate.weight_max,
        status: if rate.is_active { "active".to_string() } else { "inactive".to_string() },
        created_at: rate.created_at,
        updated_at: rate.updated_at,
    }))
}

pub async fn admin_list_invoices(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AdminListQuery>,
) -> Result<Json<Vec<AdminInvoiceResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    
    let invoices = sqlx::query_as::<_, Invoice>(
        "SELECT * FROM invoices ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list invoices: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let response: Vec<AdminInvoiceResponse> = invoices
        .into_iter()
        .map(|i| AdminInvoiceResponse {
            id: i.id,
            tenant_id: i.tenant_id,
            tenant_name: format!("Tenant {}", &i.tenant_id.to_string()[..8]),
            order_id: i.order_id,
            invoice_number: i.invoice_number,
            status: i.status,
            subtotal: i.subtotal,
            tax_amount: i.tax_amount,
            total: i.total,
            currency: "INR".to_string(),
            due_date: i.due_date,
            paid_at: i.paid_at,
            created_at: i.created_at,
            updated_at: i.updated_at,
        })
        .collect();
    
    Ok(Json(response))
}

pub async fn admin_get_invoice(
    State(state): State<Arc<AppState>>,
    Path(invoice_id): Path<Uuid>,
) -> Result<Json<AdminInvoiceResponse>, StatusCode> {
    let invoice = sqlx::query_as::<_, Invoice>(
        "SELECT * FROM invoices WHERE id = $1",
    )
    .bind(invoice_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get invoice: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(AdminInvoiceResponse {
        id: invoice.id,
        tenant_id: invoice.tenant_id,
        tenant_name: format!("Tenant {}", &invoice.tenant_id.to_string()[..8]),
        order_id: invoice.order_id,
        invoice_number: invoice.invoice_number,
        status: invoice.status,
        subtotal: invoice.subtotal,
        tax_amount: invoice.tax_amount,
        total: invoice.total,
        currency: "INR".to_string(),
        due_date: invoice.due_date,
        paid_at: invoice.paid_at,
        created_at: invoice.created_at,
        updated_at: invoice.updated_at,
    }))
}

pub async fn admin_list_payment_gateways(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AdminListQuery>,
) -> Result<Json<Vec<AdminPaymentGatewayResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    
    let gateways = sqlx::query_as::<_, PaymentGateway>(
        "SELECT * FROM payment_gateways ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list payment gateways: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let response: Vec<AdminPaymentGatewayResponse> = gateways
        .into_iter()
        .map(|p| AdminPaymentGatewayResponse {
            id: p.id,
            tenant_id: p.tenant_id,
            tenant_name: format!("Tenant {}", &p.tenant_id.to_string()[..8]),
            name: p.name,
            gateway_type: p.gateway_type,
            status: if p.is_active { "active".to_string() } else { "inactive".to_string() },
            is_active: p.is_active,
            created_at: p.created_at,
            updated_at: p.updated_at,
        })
        .collect();
    
    Ok(Json(response))
}

pub async fn admin_get_payment_gateway(
    State(state): State<Arc<AppState>>,
    Path(gateway_id): Path<Uuid>,
) -> Result<Json<AdminPaymentGatewayResponse>, StatusCode> {
    let gateway = sqlx::query_as::<_, PaymentGateway>(
        "SELECT * FROM payment_gateways WHERE id = $1",
    )
    .bind(gateway_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get payment gateway: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(AdminPaymentGatewayResponse {
        id: gateway.id,
        tenant_id: gateway.tenant_id,
        tenant_name: format!("Tenant {}", &gateway.tenant_id.to_string()[..8]),
        name: gateway.name,
        gateway_type: gateway.gateway_type,
        status: if gateway.is_active { "active".to_string() } else { "inactive".to_string() },
        is_active: gateway.is_active,
        created_at: gateway.created_at,
        updated_at: gateway.updated_at,
    }))
}

pub async fn admin_list_inventory(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AdminListQuery>,
) -> Result<Json<Vec<AdminInventoryResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    
    let items = sqlx::query_as::<_, InventoryItem>(
        "SELECT * FROM inventory ORDER BY updated_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list inventory: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let response: Vec<AdminInventoryResponse> = items
        .into_iter()
        .map(|i| AdminInventoryResponse {
            id: i.id,
            tenant_id: i.tenant_id,
            tenant_name: format!("Tenant {}", &i.tenant_id.to_string()[..8]),
            product_id: i.product_id,
            product_name: "Product".to_string(),
            sku: i.sku,
            quantity: i.quantity,
            reserved_quantity: i.reserved_quantity,
            available_quantity: i.available_quantity,
            stock_status: if i.available_quantity > 10 { "in_stock".to_string() } else if i.available_quantity > 0 { "low_stock".to_string() } else { "out_of_stock".to_string() },
            updated_at: i.updated_at,
        })
        .collect();
    
    Ok(Json(response))
}

pub async fn admin_get_inventory(
    State(state): State<Arc<AppState>>,
    Path(inventory_id): Path<Uuid>,
) -> Result<Json<AdminInventoryResponse>, StatusCode> {
    let item = sqlx::query_as::<_, InventoryItem>(
        "SELECT * FROM inventory WHERE id = $1",
    )
    .bind(inventory_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get inventory: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(AdminInventoryResponse {
        id: item.id,
        tenant_id: item.tenant_id,
        tenant_name: format!("Tenant {}", &item.tenant_id.to_string()[..8]),
        product_id: item.product_id,
        product_name: "Product".to_string(),
        sku: item.sku,
        quantity: item.quantity,
        reserved_quantity: item.reserved_quantity,
        available_quantity: item.available_quantity,
        stock_status: if item.available_quantity > 10 { "in_stock".to_string() } else if item.available_quantity > 0 { "low_stock".to_string() } else { "out_of_stock".to_string() },
        updated_at: item.updated_at,
    }))
}