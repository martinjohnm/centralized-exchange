# High-Performance Rust Exchange (LOB) Core

A distributed, low-latency Limit Order Book (LOB) and matching engine architected for high-concurrency financial transactions. This system bridges a **synchronous Axum API gateway** with an **asynchronous Rust execution kernel** using a Redis-backed event-sourced architecture.

---

## 🏗 Key Engineering Challenges Solved

### 1. The "Sync-over-Async" Bridge
* **Problem:** HTTP is synchronous, but a matching engine is asynchronous. 
* **Solution:** Engineered a non-blocking ingress gateway that parks HTTP handlers using `oneshot` channels. The API waits for a specific `ExecutionReport` keyed to a unique `client_id` before responding, ensuring the user receives the immediate result of their trade without blocking system threads.

### 2. Event-Driven Microservices Architecture
* **State Decoupling:** Separated the **Matching Engine** (Hot Path) from the **API Gateway** (IO Path) using Redis as a high-speed message bus.
* **Serialization:** Utilized **Protocol Buffers (Protobuf)** via `prost` for cross-service communication, minimizing payload size and CPU cycles during serialization compared to JSON.

### 3. Reactive Real-Time Updates
* **Signaling Manager:** Developed a WebSocket broadcast layer that streams incremental orderbook updates (L2) and trade executions.
* **Hybrid Reporting:** Immediate fills return via the HTTP response path for user feedback, while subsequent "resting" order fills are broadcasted asynchronously via WebSockets to maintain sub-millisecond UI updates.

### 4. Memory Safety & High Concurrency
* **Ownership & Pinning:** Leveraged Rust’s strict ownership model and `tokio::pin!` to manage high-speed asynchronous Redis Pub/Sub streams safely.
* **Thread Safety:** Utilized concurrent data structures (`DashMap`) to route engine responses to the correct pending HTTP requests with $O(1)$ complexity.

---

## 🛠 Tech Stack

| Layer | Technology |
| :--- | :--- |
| **Language** | Rust (Stable) |
| **API Gateway** | Axum / Tokio |
| **Messaging/Bus** | Redis (Pub/Sub & List-based WAL) |
| **Serialization** | Protobuf (Prost) |
| **Frontend** | React / TypeScript / Tailwind CSS / Vite |
| **Infrastructure** | Docker & Docker Compose |

---

## 🚦 System Workflow

1. **Ingress:** Axum receives a `CreateOrder` request and generates a unique `client_id`.
2. **Persistence:** The order is serialized into Protobuf and pushed to a Redis-based Write-Ahead Log (WAL).
3. **Execution:** The **Matching Engine** pops the order, matches it against the internal BTreeMap-based orderbook, and generates an `ExecutionReport`.
4. **Egress:** The report is fanned out:
   - Sent back to the specific Axum handler to resolve the HTTP request.
   - Broadcasted to the **Signaling Manager** for global WebSocket updates.

---

## 🚀 Future Roadmap (Scalability Targets)

- [ ] **Durable Persistence:** Implementation of a dedicated DB-Writer service to move SQL I/O out of the execution hot-path.
- [ ] **Redis Streams:** Transitioning from Pub/Sub to Redis Streams (XADD) for "at-least-once" delivery



## One-Command Setup

Follow these steps to "hydrate" the environment and launch the entire stack.

### 1. Bootstrap Environment
The project uses template files to manage service coordinates. Copy the examples to create your local environment manifests:

#### From the project root:
```bash
cp .env.example .env && cp apps/frontend/.env.example apps/frontend/.env
```
### 2. Build and Start (The "One-Command" Setup)
This command triggers the Cargo workspace compilation, the Vite production build, and initializes the Redis backplane in a single pass:

```bash 
docker compose up --build
```
### 3. Run 
```bash
docker compose up
```