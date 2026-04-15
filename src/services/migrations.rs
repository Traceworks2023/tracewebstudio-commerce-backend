use sqlx::PgPool;
use tracing::info;

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Running commerce database migrations...");
    
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS categories (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            tenant_id UUID NOT NULL,
            parent_id UUID REFERENCES categories(id),
            name VARCHAR(255) NOT NULL,
            slug VARCHAR(255) NOT NULL,
            description TEXT,
            image_url VARCHAR(500),
            sort_order INTEGER DEFAULT 0,
            is_active BOOLEAN DEFAULT true,
            metadata JSONB,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(tenant_id, slug)
        )
        "#,
    )
    .execute(pool)
    .await?;
    
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS products (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            tenant_id UUID NOT NULL,
            category_id UUID REFERENCES categories(id),
            name VARCHAR(255) NOT NULL,
            slug VARCHAR(255) NOT NULL,
            description TEXT,
            price DECIMAL(10, 2) NOT NULL,
            compare_at_price DECIMAL(10, 2),
            cost_price DECIMAL(10, 2),
            sku VARCHAR(100),
            barcode VARCHAR(100),
            inventory_quantity INTEGER DEFAULT 0,
            inventory_policy inventory_policy DEFAULT 'deny',
            weight DECIMAL(10, 2),
            weight_unit weight_unit DEFAULT 'kg',
            is_active BOOLEAN DEFAULT true,
            is_featured BOOLEAN DEFAULT false,
            metadata JSONB,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(tenant_id, slug)
        )
        "#,
    )
    .execute(pool)
    .await?;
    
    sqlx::query(
        r#"
        DO $$ BEGIN
            CREATE TYPE inventory_policy AS ENUM ('deny', 'continue');
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;
    
    sqlx::query(
        r#"
        DO $$ BEGIN
            CREATE TYPE weight_unit AS ENUM ('kg', 'lb', 'oz', 'g');
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;
    
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS customers (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            tenant_id UUID NOT NULL,
            email VARCHAR(255) NOT NULL,
            phone VARCHAR(50),
            first_name VARCHAR(100),
            last_name VARCHAR(100),
            orders_count INTEGER DEFAULT 0,
            total_spent DECIMAL(12, 2) DEFAULT 0,
            tags TEXT[] DEFAULT '{}',
            note TEXT,
            metadata JSONB,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(tenant_id, email)
        )
        "#,
    )
    .execute(pool)
    .await?;
    
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS carts (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            tenant_id UUID NOT NULL,
            customer_id UUID REFERENCES customers(id),
            session_id VARCHAR(255),
            currency VARCHAR(3) DEFAULT 'USD',
            subtotal DECIMAL(12, 2) DEFAULT 0,
            tax_amount DECIMAL(12, 2) DEFAULT 0,
            shipping_amount DECIMAL(12, 2) DEFAULT 0,
            discount_amount DECIMAL(12, 2) DEFAULT 0,
            total DECIMAL(12, 2) DEFAULT 0,
            metadata JSONB,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;
    
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS cart_items (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            cart_id UUID NOT NULL REFERENCES carts(id) ON DELETE CASCADE,
            product_id UUID NOT NULL REFERENCES products(id),
            variant_id UUID,
            name VARCHAR(255) NOT NULL,
            sku VARCHAR(100),
            price DECIMAL(10, 2) NOT NULL,
            quantity INTEGER NOT NULL DEFAULT 1,
            tax_amount DECIMAL(10, 2) DEFAULT 0,
            discount_amount DECIMAL(10, 2) DEFAULT 0,
            total DECIMAL(12, 2) NOT NULL,
            metadata JSONB
        )
        "#,
    )
    .execute(pool)
    .await?;
    
    sqlx::query(
        r#"
        DO $$ BEGIN
            CREATE TYPE order_status AS ENUM ('pending', 'confirmed', 'processing', 'shipped', 'delivered', 'cancelled', 'refunded');
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;
    
    sqlx::query(
        r#"
        DO $$ BEGIN
            CREATE TYPE financial_status AS ENUM ('pending', 'paid', 'partially_refunded', 'refunded', 'voided');
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;
    
    sqlx::query(
        r#"
        DO $$ BEGIN
            CREATE TYPE fulfillment_status AS ENUM ('unfulfilled', 'partially_fulfilled', 'fulfilled');
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;
    
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS orders (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            tenant_id UUID NOT NULL,
            customer_id UUID NOT NULL REFERENCES customers(id),
            order_number VARCHAR(50) NOT NULL,
            status order_status DEFAULT 'pending',
            financial_status financial_status DEFAULT 'pending',
            fulfillment_status fulfillment_status DEFAULT 'unfulfilled',
            subtotal DECIMAL(12, 2) NOT NULL,
            tax_amount DECIMAL(12, 2) DEFAULT 0,
            shipping_amount DECIMAL(12, 2) DEFAULT 0,
            discount_amount DECIMAL(12, 2) DEFAULT 0,
            total DECIMAL(12, 2) NOT NULL,
            currency VARCHAR(3) DEFAULT 'USD',
            shipping_address JSONB,
            billing_address JSONB,
            shipping_method VARCHAR(100),
            payment_method VARCHAR(100),
            payment_reference VARCHAR(255),
            notes TEXT,
            metadata JSONB,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(tenant_id, order_number)
        )
        "#,
    )
    .execute(pool)
    .await?;
    
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS order_items (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
            product_id UUID NOT NULL REFERENCES products(id),
            variant_id UUID,
            name VARCHAR(255) NOT NULL,
            sku VARCHAR(100),
            price DECIMAL(10, 2) NOT NULL,
            quantity INTEGER NOT NULL,
            tax_amount DECIMAL(10, 2) DEFAULT 0,
            discount_amount DECIMAL(10, 2) DEFAULT 0,
            total DECIMAL(12, 2) NOT NULL,
            metadata JSONB
        )
        "#,
    )
    .execute(pool)
    .await?;
    
    sqlx::query(
        r#"
        DO $$ BEGIN
            CREATE TYPE coupon_type AS ENUM ('percentage', 'fixed_amount', 'free_shipping');
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;
    
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS coupons (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            tenant_id UUID NOT NULL,
            code VARCHAR(50) NOT NULL,
            coupon_type coupon_type NOT NULL,
            value DECIMAL(10, 2) NOT NULL,
            minimum_order_amount DECIMAL(10, 2),
            maximum_discount DECIMAL(10, 2),
            usage_limit INTEGER,
            usage_count INTEGER DEFAULT 0,
            starts_at TIMESTAMPTZ DEFAULT NOW(),
            ends_at TIMESTAMPTZ,
            is_active BOOLEAN DEFAULT true,
            metadata JSONB,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(tenant_id, code)
        )
        "#,
    )
    .execute(pool)
    .await?;
    
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS shipping_rates (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            tenant_id UUID NOT NULL,
            name VARCHAR(100) NOT NULL,
            code VARCHAR(50) NOT NULL,
            price DECIMAL(10, 2) NOT NULL,
            weight_min DECIMAL(10, 2),
            weight_max DECIMAL(10, 2),
            is_active BOOLEAN DEFAULT true,
            metadata JSONB,
            UNIQUE(tenant_id, code)
        )
        "#,
    )
    .execute(pool)
    .await?;
    
    info!("Commerce database migrations completed successfully");
    Ok(())
}
