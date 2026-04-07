# 电商平台微服务示例

本示例展示如何使用 Aether 构建一个完整的电商微服务系统，包括：
- 用户认证服务
- 商品目录服务
- 订单处理服务
- 支付网关集成
- 库存管理
- 异步消息队列

## 项目结构

```
ecommerce/
├── Aether.toml
├── src/
│   ├── main.ae
│   ├── lib.ae
│   ├── config.ae
│   ├── models/
│   │   ├── user.ae
│   │   ├── product.ae
│   │   ├── order.ae
│   │   └── payment.ae
│   ├── services/
│   │   ├── auth.ae
│   │   ├── catalog.ae
│   │   ├── order_service.ae
│   │   ├── payment_gateway.ae
│   │   └── inventory.ae
│   ├── api/
│   │   ├── routes.ae
│   │   ├── handlers.ae
│   │   └── middleware.ae
│   ├── db/
│   │   ├── connection.ae
│   │   ├── repositories/
│   │   └── migrations/
│   └── utils/
│       ├── logging.ae
│       ├── errors.ae
│       └── validation.ae
├── tests/
│   ├── integration/
│   └── unit/
└── migrations/
```

## Aether.toml

```toml
[package]
name = "ecommerce-platform"
version = "1.0.0"
edition = "2024"
authors = ["E-Commerce Team"]

[dependencies]
aether-std = "1.0"
hyperion = { version = "2.1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio"] }
redis = "0.23"
jwt-validator = "0.5"
argon2 = "0.5"
uuid = { version = "1.0", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = "0.3"
prometheus = "0.13"

[dev-dependencies]
aether-test = "1.0"
mockall = "0.11"

[features]
default = ["postgres"]
postgres = []
mysql = []
```

## 核心代码示例

### 1. 数据模型 (src/models/user.ae)

```ae
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    Customer,
    Vendor,
    Admin,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthToken {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

impl User {
    pub fn new(request: CreateUserRequest, password_hash: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            email: request.email,
            password_hash,
            first_name: request.first_name,
            last_name: request.last_name,
            role: UserRole::Customer,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}
```

### 2. 认证服务 (src/services/auth.ae)

```ae
use crate::models::{User, UserRole, CreateUserRequest, LoginRequest, AuthToken};
use crate::db::repositories::UserRepository;
use crate::utils::errors::{AppError, Result};
use argon2::{Argon2, PasswordVerifier, PasswordHasher};
use jwt_validator::{Validator, Claims};
use uuid::Uuid;
use std::sync::Arc;
use std::time::Duration;

pub struct AuthService {
    user_repo: Arc<UserRepository>,
    jwt_validator: Validator,
    argon2: Argon2<'static>,
}

impl AuthService {
    pub fn new(user_repo: Arc<UserRepository>, jwt_secret: String) -> Self {
        Self {
            user_repo,
            jwt_validator: Validator::new(jwt_secret.as_bytes()),
            argon2: Argon2::default(),
        }
    }
    
    pub async fn register(&self, request: CreateUserRequest) -> Result<User> {
        // 验证邮箱格式
        if !self.validate_email(&request.email) {
            return Err(AppError::ValidationError("Invalid email".to_string()));
        }
        
        // 检查邮箱是否已存在
        if self.user_repo.exists_by_email(&request.email).await? {
            return Err(AppError::Conflict("Email already registered".to_string()));
        }
        
        // 哈希密码
        let password_hash = self.hash_password(&request.password)?;
        
        // 创建用户
        let user = User::new(request, password_hash);
        self.user_repo.create(&user).await?;
        
        Ok(user)
    }
    
    pub async fn login(&self, request: LoginRequest) -> Result<AuthToken> {
        // 查找用户
        let user = self.user_repo.find_by_email(&request.email)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        
        // 验证密码
        if !self.verify_password(&request.password, &user.password_hash)? {
            return Err(AppError::Unauthorized("Invalid credentials".to_string()));
        }
        
        // 生成 JWT
        let claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            role: user.role.to_string(),
            exp: (Utc::now() + Duration::hours(1)).timestamp(),
        };
        
        let access_token = self.jwt_validator.sign(&claims)?;
        let refresh_token = self.generate_refresh_token(user.id)?;
        
        Ok(AuthToken {
            access_token,
            refresh_token,
            expires_in: 3600,
        })
    }
    
    pub async fn get_current_user(&self, token: &str) -> Result<User> {
        let claims = self.jwt_validator.validate(token)?;
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;
        
        self.user_repo.find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }
    
    fn hash_password(&self, password: &str) -> Result<String> {
        use std::vec::Vec;
        let salt = argon2::password_hash::SaltString::generate(&mut rand::thread_rng());
        let password_hash = self.argon2.hash_password(password.as_bytes(), &salt)?;
        Ok(password_hash.to_string())
    }
    
    fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = argon2::password_hash::PasswordHash::new(hash)?;
        Ok(self.argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }
    
    fn validate_email(&self, email: &str) -> bool {
        // 简化的邮箱验证
        email.contains('@') && email.contains('.')
    }
    
    fn generate_refresh_token(&self, user_id: Uuid) -> Result<String> {
        // 生成刷新令牌逻辑
        Ok(Uuid::new_v4().to_string())
    }
}
```

### 3. 商品服务 (src/services/catalog.ae)

```ae
use crate::models::{Product, ProductFilter, ProductPage};
use crate::db::repositories::ProductRepository;
use crate::utils::errors::Result;
use std::sync::Arc;

pub struct CatalogService {
    product_repo: Arc<ProductRepository>,
}

impl CatalogService {
    pub fn new(product_repo: Arc<ProductRepository>) -> Self {
        Self { product_repo }
    }
    
    pub async fn list_products(&self, filter: ProductFilter) -> Result<ProductPage> {
        self.product_repo.find_with_filter(filter).await
    }
    
    pub async fn get_product(&self, id: Uuid) -> Result<Product> {
        self.product_repo.find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Product not found".to_string()))
    }
    
    pub async fn search_products(&self, query: &str, limit: u32) -> Result<Vec<Product>> {
        self.product_repo.search(query, limit).await
    }
    
    pub async fn get_categories(&self) -> Result<Vec<Category>> {
        self.product_repo.list_categories().await
    }
}

#[derive(Debug, Clone)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub price: Decimal,
    pub currency: String,
    pub stock_quantity: i32,
    pub category_id: Uuid,
    pub images: Vec<String>,
    pub attributes: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct ProductFilter {
    pub category_id: Option<Uuid>,
    pub min_price: Option<Decimal>,
    pub max_price: Option<Decimal>,
    pub in_stock: Option<bool>,
    pub sort_by: ProductSort,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ProductSort {
    #[default]
    Relevance,
    PriceAsc,
    PriceDesc,
    Newest,
    Popular,
}

#[derive(Debug, Clone)]
pub struct ProductPage {
    pub products: Vec<Product>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}
```

### 4. 订单服务 (src/services/order_service.ae)

```ae
use crate::models::{Order, OrderItem, OrderStatus, CreateOrderRequest};
use crate::services::{InventoryService, PaymentGateway};
use crate::db::repositories::OrderRepository;
use crate::utils::errors::{AppError, Result};
use std::sync::Arc;
use std::async::channel;

pub struct OrderService {
    order_repo: Arc<OrderRepository>,
    inventory_service: Arc<InventoryService>,
    payment_gateway: Arc<PaymentGateway>,
    event_sender: channel::Sender<OrderEvent>,
}

impl OrderService {
    pub fn new(
        order_repo: Arc<OrderRepository>,
        inventory_service: Arc<InventoryService>,
        payment_gateway: Arc<PaymentGateway>,
        event_sender: channel::Sender<OrderEvent>,
    ) -> Self {
        Self {
            order_repo,
            inventory_service,
            payment_gateway,
            event_sender,
        }
    }
    
    pub async fn create_order(&self, request: CreateOrderRequest, user_id: Uuid) -> Result<Order> {
        // 开启事务
        let mut tx = self.order_repo.begin_transaction().await?;
        
        // 验证并锁定库存
        for item in &request.items {
            self.inventory_service.reserve_stock(item.product_id, item.quantity)
                .await
                .map_err(|e| {
                    tx.rollback().await.ok();
                    e
                })?;
        }
        
        // 创建订单
        let mut order = Order::from_request(request, user_id);
        order = self.order_repo.create(order, &mut tx).await?;
        
        // 提交事务
        tx.commit().await?;
        
        // 发送事件
        self.event_sender.send(OrderEvent::OrderCreated(order.clone())).await?;
        
        Ok(order)
    }
    
    pub async fn process_payment(&self, order_id: Uuid, payment_method: PaymentMethod) -> Result<PaymentResult> {
        let order = self.order_repo.find_by_id(order_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Order not found".to_string()))?;
        
        if order.status != OrderStatus::Pending {
            return Err(AppError::BadRequest("Order cannot be paid".to_string()));
        }
        
        // 处理支付
        let payment_result = self.payment_gateway.charge(order.total_amount, payment_method).await?;
        
        if payment_result.success {
            // 更新订单状态
            let updated_order = self.order_repo.update_status(order_id, OrderStatus::Paid).await?;
            
            // 发送事件
            self.event_sender.send(OrderEvent::PaymentSuccess(updated_order)).await?;
        } else {
            // 释放库存
            for item in &order.items {
                self.inventory_service.release_stock(item.product_id, item.quantity).await?;
            }
            
            self.order_repo.update_status(order_id, OrderStatus::PaymentFailed).await?;
        }
        
        Ok(payment_result)
    }
    
    pub async fn cancel_order(&self, order_id: Uuid, reason: String) -> Result<()> {
        let order = self.order_repo.find_by_id(order_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Order not found".to_string()))?;
        
        if order.status != OrderStatus::Pending && order.status != OrderStatus::Paid {
            return Err(AppError::BadRequest("Cannot cancel order".to_string()));
        }
        
        // 释放库存
        for item in &order.items {
            self.inventory_service.release_stock(item.product_id, item.quantity).await?;
        }
        
        // 如果是已支付订单，发起退款
        if order.status == OrderStatus::Paid {
            self.payment_gateway.refund(order.total_amount, order.payment_id.as_ref().unwrap()).await?;
        }
        
        self.order_repo.update_status(order_id, OrderStatus::Cancelled).await?;
        
        self.event_sender.send(OrderEvent::OrderCancelled(order_id, reason)).await?;
        
        Ok(())
    }
    
    pub async fn get_user_orders(&self, user_id: Uuid, status: Option<OrderStatus>) -> Result<Vec<Order>> {
        self.order_repo.find_by_user(user_id, status).await
    }
}

#[derive(Debug, Clone)]
pub enum OrderEvent {
    OrderCreated(Order),
    PaymentSuccess(Order),
    OrderShipped(Order),
    OrderCancelled(Uuid, String),
}
```

### 5. API 路由 (src/api/routes.ae)

```ae
use hyperion::{Router, Request, Response, Method};
use crate::api::handlers::*;
use crate::api::middleware::{auth_middleware, logging_middleware, rate_limit_middleware};

pub fn create_router() -> Router {
    let mut router = Router::new();
    
    // 全局中间件
    router.use(logging_middleware);
    router.use(rate_limit_middleware(100)); // 100 requests per minute
    
    // 健康检查
    router.get("/health", health_check);
    
    // 认证路由
    let mut auth_routes = Router::new();
    auth_routes.post("/register", register_handler);
    auth_routes.post("/login", login_handler);
    auth_routes.post("/refresh", refresh_token_handler);
    router.mount("/auth", auth_routes);
    
    // 需要认证的路由
    let mut protected_routes = Router::new();
    
    // 用户资料
    protected_routes.get("/profile", get_profile_handler);
    protected_routes.put("/profile", update_profile_handler);
    
    // 商品目录
    let mut catalog_routes = Router::new();
    catalog_routes.get("/", list_products_handler);
    catalog_routes.get("/:id", get_product_handler);
    catalog_routes.get("/search", search_products_handler);
    catalog_routes.get("/categories", list_categories_handler);
    protected_routes.mount("/products", catalog_routes);
    
    // 订单
    let mut order_routes = Router::new();
    order_routes.get("/", list_user_orders_handler);
    order_routes.post("/", create_order_handler);
    order_routes.get("/:id", get_order_handler);
    order_routes.post("/:id/cancel", cancel_order_handler);
    order_routes.post("/:id/pay", process_payment_handler);
    protected_routes.mount("/orders", order_routes);
    
    // 应用认证中间件到保护的路由
    router.mount("/api/v1", protected_routes.with_middleware(auth_middleware));
    
    // 管理员路由
    let mut admin_routes = Router::new();
    admin_routes.use(admin_auth_middleware);
    admin_routes.get("/users", list_users_handler);
    admin_routes.get("/orders", list_all_orders_handler);
    admin_routes.post("/products", create_product_handler);
    admin_routes.put("/products/:id", update_product_handler);
    admin_routes.delete("/products/:id", delete_product_handler);
    router.mount("/admin", admin_routes);
    
    router
}

// 404 处理器
async fn not_found_handler(req: Request) -> Response {
    Response::json(404, json!({
        "error": "Not Found",
        "message": format!("Path {} not found", req.path)
    }))
}
```

### 6. 主程序 (src/main.ae)

```ae
use ecommerce_platform::{config, create_app, db, services};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // 加载配置
    let config = config::Config::from_env()?;
    tracing::info!("Starting E-Commerce Platform v{}", env!("CARGO_PKG_VERSION"));
    
    // 数据库连接
    let db_pool = db::create_pool(&config.database_url).await?;
    db::run_migrations(&db_pool).await?;
    tracing::info!("Database connected and migrated");
    
    // Redis 连接（缓存和会话）
    let redis_client = redis::Client::open(config.redis_url.as_str())?;
    
    // 初始化仓库
    let user_repo = Arc::new(db::repositories::UserRepository::new(db_pool.clone()));
    let product_repo = Arc::new(db::repositories::ProductRepository::new(db_pool.clone()));
    let order_repo = Arc::new(db::repositories::OrderRepository::new(db_pool.clone()));
    
    // 初始化服务
    let auth_service = Arc::new(services::AuthService::new(
        user_repo.clone(),
        config.jwt_secret.clone(),
    ));
    
    let inventory_service = Arc::new(services::InventoryService::new(
        product_repo.clone(),
        redis_client.clone(),
    ));
    
    let payment_gateway = Arc::new(services::PaymentGateway::new(
        config.stripe_api_key.clone(),
    ));
    
    // 事件通道
    let (event_tx, event_rx) = std::async::channel::unbounded();
    
    let order_service = Arc::new(services::OrderService::new(
        order_repo.clone(),
        inventory_service.clone(),
        payment_gateway.clone(),
        event_tx,
    ));
    
    // 启动事件处理器
    tokio::spawn(handle_order_events(event_rx, db_pool.clone()));
    
    // 创建应用
    let app = create_app(
        auth_service,
        catalog_service,
        order_service,
        config.clone(),
    );
    
    // 启动服务器
    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("Server listening on {}", addr);
    
    hyperion::Server::bind(&addr.parse()?)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}

async fn handle_order_events(
    mut event_rx: std::async::channel::Receiver<OrderEvent>,
    db_pool: DbPool,
) {
    while let Ok(event) = event_rx.recv().await {
        match event {
            OrderEvent::OrderCreated(order) => {
                tracing::info!("Order created: {}", order.id);
                // 发送确认邮件
                // send_order_confirmation_email(&order).await;
            }
            OrderEvent::PaymentSuccess(order) => {
                tracing::info!("Payment successful for order: {}", order.id);
                // 通知仓库发货
                // notify_warehouse(&order).await;
            }
            OrderEvent::OrderShipped(order) => {
                tracing::info!("Order shipped: {}", order.id);
                // 发送发货通知
                // send_shipping_notification(&order).await;
            }
            OrderEvent::OrderCancelled(order_id, reason) => {
                tracing::warn!("Order cancelled: {} - {}", order_id, reason);
                // 发送取消通知
                // send_cancellation_notification(order_id, &reason).await;
            }
        }
    }
}
```

## 测试示例

```ae
#[cfg(test)]
mod tests {
    use super::*;
    use aether_test::{test, assert_eq, assert_ok, assert_err};
    
    #[test]
    async fn test_user_registration() {
        let db_pool = setup_test_db().await;
        let user_repo = Arc::new(UserRepository::new(db_pool));
        let auth_service = AuthService::new(user_repo, "test-secret".to_string());
        
        let request = CreateUserRequest {
            email: "test@example.com".to_string(),
            password: "secure_password_123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
        };
        
        let user = assert_ok!(auth_service.register(request).await);
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.role, UserRole::Customer);
    }
    
    #[test]
    async fn test_duplicate_email_registration() {
        // ... 设置
        let request1 = create_test_user_request();
        let request2 = create_test_user_request();
        
        assert_ok!(auth_service.register(request1).await);
        let result = auth_service.register(request2).await;
        
        assert_err!(result, AppError::Conflict(_));
    }
    
    #[test]
    async fn test_create_order_with_insufficient_stock() {
        // ... 设置
        let request = CreateOrderRequest {
            items: vec![OrderItem {
                product_id: product_id,
                quantity: 100,  // 超过库存
            }],
            shipping_address: create_test_address(),
        };
        
        let result = order_service.create_order(request, user_id).await;
        assert_err!(result, AppError::BadRequest(_));
    }
}
```

## 部署配置

### Dockerfile

```dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/ecommerce-platform /usr/local/bin/

EXPOSE 8080
CMD ["ecommerce-platform"]
```

### docker-compose.yml

```yaml
version: '3.8'

services:
  app:
    build: .
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgresql://postgres:password@db:5432/ecommerce
      - REDIS_URL=redis://redis:6379
      - JWT_SECRET=your-secret-key
      - RUST_LOG=info
    depends_on:
      - db
      - redis
  
  db:
    image: postgres:15
    environment:
      - POSTGRES_DB=ecommerce
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres_data:/var/lib/postgresql/data
  
  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data
  
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./ssl:/etc/nginx/ssl

volumes:
  postgres_data:
  redis_data:
```

## 性能基准

```bash
# 运行基准测试
apm run --release bench

# 结果示例
test bench_create_order ... bench:     125,000 ns/op
test bench_list_products ... bench:      45,000 ns/op
test bench_auth_login ... bench:      85,000 ns/op

# 负载测试
wrk -t12 -c400 -d30s http://localhost:8080/api/v1/products
```

这个示例展示了 Aether 语言在构建复杂企业级应用时的完整能力，包括类型安全、异步编程、错误处理、依赖注入、测试框架等核心特性。
