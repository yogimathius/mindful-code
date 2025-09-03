# Mindful Code Backend

Ultra-low latency Rust backend for developer productivity tracking with real-time flow state detection, ML-powered insights, and privacy-first data handling.

## 🚀 Performance Targets

- **Flow State Detection**: <1ms latency
- **API Response Time**: <5ms 
- **Throughput**: 50,000+ requests/second
- **Memory Usage**: <20MB per 1,000 users
- **WebSocket Latency**: <10ms real-time updates

## 🏗️ Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   VS Code       │    │   React Dashboard│    │   Mobile Apps   │
│   Extension     │    │                  │    │                 │
└─────────┬───────┘    └─────────┬────────┘    └─────────┬───────┘
          │                      │                       │
          └──────────────────────┼───────────────────────┘
                                 │
    ┌─────────────────────────────▼─────────────────────────────┐
    │                  Axum Web Server                          │
    │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐│
    │  │  REST API       │  │   WebSockets    │  │  GraphQL    ││
    │  │  Endpoints      │  │   Real-time     │  │  (Future)   ││
    │  └─────────────────┘  └─────────────────┘  └─────────────┘│
    └───────────────────────────┬───────────────────────────────┘
                                │
    ┌───────────────────────────▼───────────────────────────────┐
    │                Core Services Layer                        │
    │  ┌───────────────┐ ┌─────────────┐ ┌──────────────────────┐│
    │  │ Flow Detection│ │ ML Engine   │ │ WebAssembly Plugins  ││
    │  │   <1ms        │ │ Inference   │ │    Extensions        ││
    │  └───────────────┘ └─────────────┘ └──────────────────────┘│
    │  ┌───────────────┐ ┌─────────────┐ ┌──────────────────────┐│
    │  │ Encryption    │ │ Privacy     │ │ Team Analytics       ││
    │  │ AES-256-GCM   │ │ Manager     │ │   (Premium)          ││
    │  └───────────────┘ └─────────────┘ └──────────────────────┘│
    └───────────────────────────┬───────────────────────────────┘
                                │
    ┌───────────────────────────▼───────────────────────────────┐
    │                  PostgreSQL Database                      │
    │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐│
    │  │  Sessions   │ │ Flow States │ │    Encrypted Data       ││
    │  │   Metrics   │ │  Analysis   │ │   GDPR Compliant        ││
    │  └─────────────┘ └─────────────┘ └─────────────────────────┘│
    └─────────────────────────────────────────────────────────────┘
```

## 🛠️ Technology Stack

- **Runtime**: Rust 1.70+ with Tokio async runtime
- **Web Framework**: Axum with ultra-low latency optimizations
- **Database**: PostgreSQL with SQLx for compile-time verified queries
- **ML Framework**: Candle for on-device inference
- **WebAssembly**: Wasmtime for extensible plugins
- **Encryption**: AES-256-GCM for privacy-first data handling
- **Testing**: Comprehensive test suite with property-based testing

## 🚦 Quick Start

### Prerequisites

- Rust 1.70+
- PostgreSQL 14+
- Node.js 18+ (for VS Code extension)

### Development Setup

1. **Clone and setup environment**:
```bash
cd mindful-code
cp .env.example .env
# Edit .env with your database credentials
```

2. **Setup PostgreSQL**:
```bash
# Create database
createdb mindful_code

# Or using Docker
docker run --name mindful-postgres \
  -e POSTGRES_DB=mindful_code \
  -e POSTGRES_USER=mindful_code \
  -e POSTGRES_PASSWORD=password \
  -p 5432:5432 -d postgres:14
```

3. **Run database migrations**:
```bash
# Migrations run automatically on startup
# Or manually: sqlx migrate run --database-url="your-db-url"
```

4. **Start the backend**:
```bash
cargo run --bin mindful-code-backend
```

5. **Run tests and benchmarks**:
```bash
# Unit and integration tests
cargo test

# Performance benchmarks
cargo bench

# Load testing
cargo test test_high_load_stability --release
```

## 📊 API Endpoints

### Core Endpoints

```rust
// Authentication
POST   /api/auth/register    // User registration
POST   /api/auth/login       // User login
POST   /api/auth/refresh     // Refresh JWT token

// Real-time Flow State Detection
POST   /api/flow/detect      // <1ms flow state analysis
GET    /api/flow/patterns    // Personal flow patterns
GET    /api/flow/insights    // AI-generated insights

// Session Management
POST   /api/sessions/start   // Start coding session
PUT    /api/sessions/:id/update // Real-time updates
POST   /api/sessions/:id/end // End session
GET    /api/sessions/history // Session history

// Team Features (Premium)
GET    /api/teams/:id/analytics // Team metrics
GET    /api/teams/:id/insights  // Team optimization
POST   /api/teams/:id/alerts    // Burnout detection

// Privacy & Data Control (GDPR)
GET    /api/privacy/export   // Export all user data
DELETE /api/privacy/purge    // Delete all user data
PUT    /api/privacy/settings // Privacy preferences

// System
GET    /health               // Health check
GET    /metrics             // Prometheus metrics
```

### WebSocket Real-time Updates

```javascript
// Connect with JWT token
const ws = new WebSocket('ws://localhost:3001/ws?token=your-jwt-token');

// Message types
{
  "type": "flow_state_update",
  "session_id": "uuid",
  "flow_state": {
    "is_in_flow": true,
    "flow_intensity": 0.85,
    "recommendations": ["Great focus!", "Keep it up!"]
  }
}
```

## ⚡ Performance Optimization

### Flow State Detection Engine

```rust
// Ultra-low latency analysis
let flow_result = engine.analyze_flow_state(flow_data, preferences).await?;
assert!(flow_result.analysis_time_ms < 1.0); // <1ms guarantee
```

Key optimizations:
- **Ring buffer** for keystroke analysis (O(1) operations)
- **Zero-copy** data processing where possible
- **Compile-time optimizations** with aggressive inlining
- **Memory pool** for frequent allocations
- **SIMD instructions** for mathematical computations

### Database Performance

```sql
-- Optimized indexes for <2ms queries
CREATE INDEX idx_sessions_user_start_time ON coding_sessions(user_id, start_time);
CREATE INDEX idx_flow_states_session_intensity ON flow_states(session_id, intensity_score);
```

### Memory Efficiency

- **Zero-cost abstractions** throughout the codebase
- **Compact data structures** with careful memory layout
- **Connection pooling** with optimal settings
- **Garbage-free hot paths** in critical sections

## 🧠 Machine Learning Pipeline

### On-Device Flow Prediction

```rust
// Lightweight neural network for real-time inference
let features = [rhythm_score, focus_score, consistency_score, error_penalty, velocity_score];
let flow_probability = ml_engine.predict_flow_state(features).await?;
```

**Model Architecture**:
- Input: 5 engineered features
- Hidden: 16 → 8 → 4 neurons (ReLU activation)
- Output: 1 sigmoid (flow probability)
- Inference time: <0.1ms

### Privacy-Preserving Learning

- **On-device processing** - no keystroke data leaves the device
- **Federated learning** for team insights (optional)
- **Differential privacy** for team analytics
- **Model updates** without exposing individual data

## 🔒 Privacy & Security

### Encryption

```rust
// AES-256-GCM for sensitive data
let encrypted = encryption_service.encrypt_sensitive_data(&user_data)?;
```

### GDPR Compliance

- **Right to Access**: Complete data export in JSON/CSV
- **Right to Deletion**: Secure multi-pass deletion
- **Right to Rectification**: Update any personal data
- **Data Minimization**: Only collect necessary metrics
- **Purpose Limitation**: Clear data usage policies

### Authentication & Authorization

- **JWT tokens** with configurable expiration
- **Argon2** password hashing
- **Rate limiting** per user/IP
- **Role-based access control** for team features

## 🧩 WebAssembly Plugin System

### Plugin Development

```rust
// Create plugin template
let plugin_builder = PluginBuilder::new("flow-analyzer".to_string())
    .add_capability("flow_analysis".to_string())
    .add_function("process_keystroke_data".to_string());

plugin_builder.build_template(&plugin_path).await?;
```

### Plugin Execution

```rust
// Safe sandboxed execution
let mut runtime = wasm_manager.create_runtime("my-plugin").await?;
let result = runtime.process_flow_data(execution_context).await?;
```

**Security Features**:
- **Sandboxed execution** with resource limits
- **Capability-based permissions** system
- **Fuel-based execution limits** to prevent infinite loops
- **Memory limits** per plugin instance

## 📈 Monitoring & Observability

### Metrics Collection

```bash
# Prometheus metrics endpoint
curl http://localhost:3001/metrics
```

**Key Metrics**:
- Request latency (p50, p95, p99)
- Throughput (requests/second)
- Error rates by endpoint
- Database connection pool usage
- WebSocket connection count
- Memory usage per service

### Health Checks

```json
{
  "status": "ok",
  "timestamp": 1699123456789,
  "database": {
    "status": "healthy",
    "pool_size": 10,
    "idle_connections": 3
  },
  "services": {
    "active_sessions": 1247,
    "flow_engines": 892,
    "websocket_connections": 456
  }
}
```

## 🧪 Testing Strategy

### Performance Testing

```bash
# Benchmark flow detection latency
cargo bench flow_detection

# API load testing
cargo bench api_performance

# Memory efficiency tests
cargo test test_memory_efficiency --release
```

### Property-Based Testing

```rust
// Ensure flow intensity always in [0,1] range
quickcheck(prop_flow_intensity_bounds as fn(Vec<u16>, u8, u8) -> TestResult);
```

### Integration Testing

- **Database integration** with test containers
- **WebSocket communication** with mock clients
- **End-to-end request pipelines**
- **High-load stability testing** (10k+ concurrent requests)

## 🚀 Deployment

### Production Configuration

```bash
# Environment variables
DATABASE_URL=postgresql://user:pass@db.example.com/mindful_code
JWT_SECRET=your-256-bit-secret-key
ENCRYPTION_KEY=your-32-byte-encryption-key
RUST_LOG=warn
MAX_CONNECTIONS=200
PORT=3001
```

### Docker Deployment

```dockerfile
FROM rust:1.70 as builder
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /target/release/mindful-code-backend /app/
EXPOSE 3001
CMD ["/app/mindful-code-backend"]
```

### Infrastructure Requirements

**Minimum Production Setup**:
- **CPU**: 2 cores (4 recommended)
- **RAM**: 2GB (4GB recommended)
- **Storage**: 20GB SSD
- **Network**: 1Gbps

**Scaling Configuration**:
- **Load Balancer**: NGINX with upstream backends
- **Database**: PostgreSQL with read replicas
- **Caching**: Redis for session state
- **Monitoring**: Prometheus + Grafana

## 📚 Architecture Decisions

### Why Rust + Axum?

1. **Memory Safety**: Zero-cost abstractions without runtime overhead
2. **Performance**: Native compilation with LLVM optimizations
3. **Concurrency**: Tokio's async runtime handles thousands of connections
4. **Type Safety**: Compile-time guarantees prevent entire classes of bugs
5. **Ecosystem**: Rich crate ecosystem for specialized needs

### Why PostgreSQL + SQLx?

1. **Compile-time Verification**: SQLx validates queries at compile time
2. **Performance**: Connection pooling and prepared statements
3. **ACID Compliance**: Strong consistency for critical user data
4. **JSON Support**: Native JSONB for flexible schema evolution
5. **Extensions**: Full-text search, analytics functions

### Why On-Device ML?

1. **Privacy**: Keystroke data never leaves the user's device
2. **Latency**: <0.1ms inference vs 100ms+ network requests
3. **Reliability**: Works offline, no dependency on external services
4. **Cost**: No inference API costs at scale
5. **Customization**: Models adapt to individual typing patterns

## 🤝 Contributing

1. **Fork** the repository
2. **Create** a feature branch
3. **Add tests** for new functionality
4. **Run benchmarks** to ensure performance targets
5. **Submit** a pull request with detailed description

### Code Style

```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Check for security issues
cargo audit
```

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

---

**Built with ⚡ performance and 🔒 privacy in mind**