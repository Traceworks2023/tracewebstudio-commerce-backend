use axum::{
    routing::{get, post, put, delete},
    Router,
    middleware as axum_middleware,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod handlers;
mod middleware;
mod models;
mod services;

use services::run_migrations;
use middleware::tenant_isolation_middleware;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<sqlx::PgPool>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    tracing::info!("Starting tracewebstudio-commerce-backend");
    
    dotenvy::dotenv().ok();
    
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;
    
    tracing::info!("Connected to PostgreSQL");
    
    run_migrations(&pool).await?;
    
    let state = Arc::new(AppState { db: Arc::new(pool) });
    
    let app = Router::new()
        .route("/health", get(health_check))
        // Products
        .route("/api/v1/products", post(handlers::create_product))
        .route("/api/v1/products", get(handlers::list_products))
        .route("/api/v1/products/:tenant_id/:id", get(handlers::get_product))
        .route("/api/v1/products/:tenant_id/:id", put(handlers::update_product))
        .route("/api/v1/products/:tenant_id/:id", delete(handlers::delete_product))
        // Categories
        .route("/api/v1/categories", post(handlers::create_category))
        .route("/api/v1/categories", get(handlers::list_categories))
        .route("/api/v1/categories/:tenant_id/:id", get(handlers::get_category))
        .route("/api/v1/categories/:tenant_id/:id", put(handlers::update_category))
        .route("/api/v1/categories/:tenant_id/:id", delete(handlers::delete_category))
        // Orders
        .route("/api/v1/orders", post(handlers::create_order))
        .route("/api/v1/orders", get(handlers::list_orders))
        .route("/api/v1/orders/:tenant_id/:id", get(handlers::get_order))
        .route("/api/v1/orders/:tenant_id/:id/status", put(handlers::update_order_status))
        .route("/api/v1/orders/:tenant_id/:id/items", get(handlers::get_order_items))
        // Customers
        .route("/api/v1/customers", post(handlers::create_customer))
        .route("/api/v1/customers", get(handlers::list_customers))
        .route("/api/v1/customers/:tenant_id/:id", get(handlers::get_customer))
        .route("/api/v1/customers/:tenant_id/:id", put(handlers::update_customer))
        .route("/api/v1/customers/:tenant_id/:id", delete(handlers::delete_customer))
        // Cart
        .route("/api/v1/carts", post(handlers::create_cart))
        .route("/api/v1/carts/:tenant_id/:id", get(handlers::get_cart))
        .route("/api/v1/carts/:tenant_id/:id/items", post(handlers::add_cart_item))
        .route("/api/v1/carts/:tenant_id/:id/items/:item_id", put(handlers::update_cart_item))
        .route("/api/v1/carts/:tenant_id/:id/items/:item_id", delete(handlers::remove_cart_item))
        .route("/api/v1/carts/:tenant_id/:id/coupon", post(handlers::apply_coupon))
        .route("/api/v1/carts/:tenant_id/:id/clear", post(handlers::clear_cart))
        // Coupons
        .route("/api/v1/coupons", post(handlers::create_coupon))
        .route("/api/v1/coupons", get(handlers::list_coupons))
        .route("/api/v1/coupons/:tenant_id/:id", get(handlers::get_coupon))
        .route("/api/v1/coupons/:tenant_id/:id", put(handlers::update_coupon))
        .route("/api/v1/coupons/:tenant_id/:id", delete(handlers::delete_coupon))
        // Discounts
        .route("/api/v1/discounts", post(handlers::create_discount))
        .route("/api/v1/discounts", get(handlers::list_discounts))
        .route("/api/v1/discounts/:tenant_id/:id", get(handlers::get_discount))
        .route("/api/v1/discounts/:tenant_id/:id", put(handlers::update_discount))
        .route("/api/v1/discounts/:tenant_id/:id", delete(handlers::delete_discount))
        // Taxes
        .route("/api/v1/taxes", post(handlers::create_tax))
        .route("/api/v1/taxes", get(handlers::list_taxes))
        .route("/api/v1/taxes/:tenant_id/:id", get(handlers::get_tax))
        .route("/api/v1/taxes/:tenant_id/:id", put(handlers::update_tax))
        .route("/api/v1/taxes/:tenant_id/:id", delete(handlers::delete_tax))
        // Shipping Rates
        .route("/api/v1/shipping-rates", post(handlers::create_shipping_rate))
        .route("/api/v1/shipping-rates", get(handlers::list_shipping_rates))
        .route("/api/v1/shipping-rates/:tenant_id/:id", get(handlers::get_shipping_rate))
        .route("/api/v1/shipping-rates/:tenant_id/:id", put(handlers::update_shipping_rate))
        .route("/api/v1/shipping-rates/:tenant_id/:id", delete(handlers::delete_shipping_rate))
        // Invoices
        .route("/api/v1/invoices", post(handlers::create_invoice))
        .route("/api/v1/invoices", get(handlers::list_invoices))
        .route("/api/v1/invoices/:tenant_id/:id", get(handlers::get_invoice))
        .route("/api/v1/invoices/:tenant_id/:id", put(handlers::update_invoice))
        // Payment Gateways
        .route("/api/v1/payment-gateways", post(handlers::create_payment_gateway))
        .route("/api/v1/payment-gateways", get(handlers::list_payment_gateways))
        .route("/api/v1/payment-gateways/:tenant_id/:id", get(handlers::get_payment_gateway))
        .route("/api/v1/payment-gateways/:tenant_id/:id", put(handlers::update_payment_gateway))
        .route("/api/v1/payment-gateways/:tenant_id/:id", delete(handlers::delete_payment_gateway))
        // Inventory
        .route("/api/v1/inventory/:tenant_id/:product_id", get(handlers::get_inventory))
        .route("/api/v1/inventory/:tenant_id/:product_id", put(handlers::update_inventory))
        // Admin routes (no tenant isolation - admin sees all tenants)
        .route("/api/v1/admin/overview", get(handlers::admin_get_overview))
        .route("/api/v1/admin/ecommerce-module", get(handlers::admin_list_ecommerce_modules))
        .route("/api/v1/admin/products", get(handlers::admin_list_products))
        .route("/api/v1/admin/products/:id", get(handlers::admin_get_product))
        .route("/api/v1/admin/customers", get(handlers::admin_list_customers))
        .route("/api/v1/admin/orders", get(handlers::admin_list_orders))
        .route("/api/v1/admin/coupons", get(handlers::admin_list_coupons))
        .route("/api/v1/admin/discounts", get(handlers::admin_list_discounts))
        .route("/api/v1/admin/discounts/:id", get(handlers::admin_get_discount))
        .route("/api/v1/admin/taxes", get(handlers::admin_list_taxes))
        .route("/api/v1/admin/taxes/:id", get(handlers::admin_get_tax))
        .route("/api/v1/admin/shipping", get(handlers::admin_list_shipping))
        .route("/api/v1/admin/shipping/:id", get(handlers::admin_get_shipping))
        .route("/api/v1/admin/invoices", get(handlers::admin_list_invoices))
        .route("/api/v1/admin/invoices/:id", get(handlers::admin_get_invoice))
        .route("/api/v1/admin/payment-gateways", get(handlers::admin_list_payment_gateways))
        .route("/api/v1/admin/payment-gateways/:id", get(handlers::admin_get_payment_gateway))
        .route("/api/v1/admin/inventory", get(handlers::admin_list_inventory))
        .route("/api/v1/admin/inventory/:id", get(handlers::admin_get_inventory))
        // Tenant commerce routes (tenant-scoped)
        .route("/api/commerce/products", get(handlers::tenant_list_products))
        .route("/api/commerce/products/:tenant_id/:id", get(handlers::tenant_get_product))
        .route("/api/commerce/products/:tenant_id/:id", put(handlers::tenant_update_product))
        .route("/api/commerce/products/:tenant_id/:id", delete(handlers::tenant_delete_product))
        .route("/api/commerce/categories", get(handlers::tenant_list_categories))
        .route("/api/commerce/categories/:tenant_id/:id", post(handlers::tenant_create_category))
        .route("/api/commerce/categories/:tenant_id/:id", put(handlers::tenant_update_category))
        .route("/api/commerce/categories/:tenant_id/:id", delete(handlers::tenant_delete_category))
        .route("/api/commerce/orders", get(handlers::tenant_list_orders))
        .route("/api/commerce/promotions", get(handlers::tenant_list_promotions))
        .route("/api/commerce/promotions/:tenant_id/:id", post(handlers::tenant_create_promotion))
        .route("/api/commerce/promotions/:tenant_id/:id", put(handlers::tenant_update_promotion))
        .route("/api/commerce/promotions/:tenant_id/:id", delete(handlers::tenant_delete_promotion))
        .route("/api/commerce/cart", get(handlers::tenant_get_cart))
        .route("/api/commerce/cart/items", post(handlers::tenant_add_cart_item))
        .route("/api/commerce/checkout", post(handlers::tenant_checkout))
        // Tenant isolation
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            tenant_isolation_middleware,
        ))
        .with_state(state);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8083")
        .await
        .expect("Failed to bind to port 8083");
    
    tracing::info!("Commerce service listening on 0.0.0.0:8083");
    
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
    
    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}
