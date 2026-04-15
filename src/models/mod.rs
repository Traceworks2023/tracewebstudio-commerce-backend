use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub category_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub price: f64,
    pub compare_at_price: Option<f64>,
    pub cost_price: Option<f64>,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub inventory_quantity: i32,
    pub inventory_policy: InventoryPolicy,
    pub weight: Option<f64>,
    pub weight_unit: WeightUnit,
    pub is_active: bool,
    pub is_featured: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub category_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub price: f64,
    pub compare_at_price: Option<f64>,
    pub cost_price: Option<f64>,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub inventory_quantity: i32,
    pub inventory_policy: InventoryPolicy,
    pub weight: Option<f64>,
    pub weight_unit: WeightUnit,
    pub is_active: bool,
    pub is_featured: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Product> for ProductResponse {
    fn from(p: Product) -> Self {
        Self {
            id: p.id,
            tenant_id: p.tenant_id,
            category_id: p.category_id,
            name: p.name,
            slug: p.slug,
            description: p.description,
            price: p.price,
            compare_at_price: p.compare_at_price,
            cost_price: p.cost_price,
            sku: p.sku,
            barcode: p.barcode,
            inventory_quantity: p.inventory_quantity,
            inventory_policy: p.inventory_policy,
            weight: p.weight,
            weight_unit: p.weight_unit,
            is_active: p.is_active,
            is_featured: p.is_featured,
            metadata: p.metadata,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "inventory_policy", rename_all = "snake_case")]
pub enum InventoryPolicy {
    #[serde(rename = "deny")]
    Deny,
    #[serde(rename = "continue")]
    Continue,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "weight_unit", rename_all = "snake_case")]
pub enum WeightUnit {
    Kg,
    Lb,
    Oz,
    G,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CategoryResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Category> for CategoryResponse {
    fn from(c: Category) -> Self {
        Self {
            id: c.id,
            tenant_id: c.tenant_id,
            parent_id: c.parent_id,
            name: c.name,
            slug: c.slug,
            description: c.description,
            image_url: c.image_url,
            sort_order: c.sort_order,
            is_active: c.is_active,
            metadata: c.metadata,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Order {
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "order_status", rename_all = "snake_case")]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
    Refunded,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "financial_status", rename_all = "snake_case")]
pub enum FinancialStatus {
    Pending,
    Paid,
    PartiallyRefunded,
    Refunded,
    Voided,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "fulfillment_status", rename_all = "snake_case")]
pub enum FulfillmentStatus {
    Unfulfilled,
    PartiallyFulfilled,
    Fulfilled,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Cart {
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CartItem {
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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Customer {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub phone: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub orders_count: i32,
    pub total_spent: f64,
    pub tags: Vec<String>,
    pub note: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Coupon {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub code: String,
    pub coupon_type: CouponType,
    pub value: f64,
    pub minimum_order_amount: Option<f64>,
    pub maximum_discount: Option<f64>,
    pub usage_limit: Option<i32>,
    pub usage_count: i32,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "coupon_type", rename_all = "snake_case")]
pub enum CouponType {
    Percentage,
    FixedAmount,
    FreeShipping,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ShippingRate {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub code: String,
    pub price: f64,
    pub weight_min: Option<f64>,
    pub weight_max: Option<f64>,
    pub is_active: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Discount {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub code: Option<String>,
    pub discount_type: String,
    pub value: f64,
    pub minimum_order_amount: Option<f64>,
    pub maximum_discount: Option<f64>,
    pub usage_limit: Option<i32>,
    pub usage_count: i32,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tax {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub rate: f64,
    pub tax_type: Option<String>,
    pub is_active: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Invoice {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub order_id: Uuid,
    pub invoice_number: String,
    pub status: String,
    pub subtotal: f64,
    pub tax_amount: f64,
    pub total: f64,
    pub due_date: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentGateway {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub gateway_type: String,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub webhook_secret: Option<String>,
    pub is_active: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InventoryItem {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub sku: Option<String>,
    pub quantity: i32,
    pub reserved_quantity: i32,
    pub available_quantity: i32,
    pub metadata: Option<serde_json::Value>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub phone: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub orders_count: i32,
    pub total_spent: f64,
    pub tags: Vec<String>,
    pub note: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Customer> for CustomerResponse {
    fn from(c: Customer) -> Self {
        Self {
            id: c.id,
            tenant_id: c.tenant_id,
            email: c.email,
            phone: c.phone,
            first_name: c.first_name,
            last_name: c.last_name,
            orders_count: c.orders_count,
            total_spent: c.total_spent,
            tags: c.tags,
            note: c.note,
            metadata: c.metadata,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Cart> for CartResponse {
    fn from(c: Cart) -> Self {
        Self {
            id: c.id,
            tenant_id: c.tenant_id,
            customer_id: c.customer_id,
            session_id: c.session_id,
            currency: c.currency,
            subtotal: c.subtotal,
            tax_amount: c.tax_amount,
            shipping_amount: c.shipping_amount,
            discount_amount: c.discount_amount,
            total: c.total,
            metadata: c.metadata,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    fn from(ci: CartItem) -> Self {
        Self {
            id: ci.id,
            cart_id: ci.cart_id,
            product_id: ci.product_id,
            variant_id: ci.variant_id,
            name: ci.name,
            sku: ci.sku,
            price: ci.price,
            quantity: ci.quantity,
            tax_amount: ci.tax_amount,
            discount_amount: ci.discount_amount,
            total: ci.total,
            metadata: ci.metadata,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouponResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub code: String,
    pub coupon_type: CouponType,
    pub value: f64,
    pub minimum_order_amount: Option<f64>,
    pub maximum_discount: Option<f64>,
    pub usage_limit: Option<i32>,
    pub usage_count: i32,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Coupon> for CouponResponse {
    fn from(c: Coupon) -> Self {
        Self {
            id: c.id,
            tenant_id: c.tenant_id,
            code: c.code,
            coupon_type: c.coupon_type,
            value: c.value,
            minimum_order_amount: c.minimum_order_amount,
            maximum_discount: c.maximum_discount,
            usage_limit: c.usage_limit,
            usage_count: c.usage_count,
            starts_at: c.starts_at,
            ends_at: c.ends_at,
            is_active: c.is_active,
            metadata: c.metadata,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscountResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub code: Option<String>,
    pub discount_type: String,
    pub value: f64,
    pub minimum_order_amount: Option<f64>,
    pub maximum_discount: Option<f64>,
    pub usage_limit: Option<i32>,
    pub usage_count: i32,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Discount> for DiscountResponse {
    fn from(d: Discount) -> Self {
        Self {
            id: d.id,
            tenant_id: d.tenant_id,
            name: d.name,
            code: d.code,
            discount_type: d.discount_type,
            value: d.value,
            minimum_order_amount: d.minimum_order_amount,
            maximum_discount: d.maximum_discount,
            usage_limit: d.usage_limit,
            usage_count: d.usage_count,
            starts_at: d.starts_at,
            ends_at: d.ends_at,
            is_active: d.is_active,
            metadata: d.metadata,
            created_at: d.created_at,
            updated_at: d.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub rate: f64,
    pub tax_type: Option<String>,
    pub is_active: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Tax> for TaxResponse {
    fn from(t: Tax) -> Self {
        Self {
            id: t.id,
            tenant_id: t.tenant_id,
            name: t.name,
            rate: t.rate,
            tax_type: t.tax_type,
            is_active: t.is_active,
            metadata: t.metadata,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingRateResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub code: String,
    pub price: f64,
    pub weight_min: Option<f64>,
    pub weight_max: Option<f64>,
    pub is_active: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ShippingRate> for ShippingRateResponse {
    fn from(s: ShippingRate) -> Self {
        Self {
            id: s.id,
            tenant_id: s.tenant_id,
            name: s.name,
            code: s.code,
            price: s.price,
            weight_min: s.weight_min,
            weight_max: s.weight_max,
            is_active: s.is_active,
            metadata: s.metadata,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub order_id: Uuid,
    pub invoice_number: String,
    pub status: String,
    pub subtotal: f64,
    pub tax_amount: f64,
    pub total: f64,
    pub due_date: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Invoice> for InvoiceResponse {
    fn from(i: Invoice) -> Self {
        Self {
            id: i.id,
            tenant_id: i.tenant_id,
            order_id: i.order_id,
            invoice_number: i.invoice_number,
            status: i.status,
            subtotal: i.subtotal,
            tax_amount: i.tax_amount,
            total: i.total,
            due_date: i.due_date,
            paid_at: i.paid_at,
            notes: i.notes,
            metadata: i.metadata,
            created_at: i.created_at,
            updated_at: i.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentGatewayResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub gateway_type: String,
    pub is_active: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PaymentGateway> for PaymentGatewayResponse {
    fn from(p: PaymentGateway) -> Self {
        Self {
            id: p.id,
            tenant_id: p.tenant_id,
            name: p.name,
            gateway_type: p.gateway_type,
            is_active: p.is_active,
            metadata: p.metadata,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub sku: Option<String>,
    pub quantity: i32,
    pub reserved_quantity: i32,
    pub available_quantity: i32,
    pub metadata: Option<serde_json::Value>,
    pub updated_at: DateTime<Utc>,
}

impl From<InventoryItem> for InventoryResponse {
    fn from(i: InventoryItem) -> Self {
        Self {
            id: i.id,
            tenant_id: i.tenant_id,
            product_id: i.product_id,
            variant_id: i.variant_id,
            sku: i.sku,
            quantity: i.quantity,
            reserved_quantity: i.reserved_quantity,
            available_quantity: i.available_quantity,
            metadata: i.metadata,
            updated_at: i.updated_at,
        }
    }
}
