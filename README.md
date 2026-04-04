# Rust Centralized Exchange (CEX) Core

A high-performance, containerized matching engine and orderbook built with **Rust**, **Redis**, and **TypeScript**. This project simulates a real-world exchange architecture using an event-driven microservices approach.

---

## The Architecture

The system is split into five core services, all managed via **Docker Compose**:

* **Matching Engine (Rust):** The high-speed core that processes orders and executes trades.
* **Order Firer (Rust):** A strategy sidecar that simulates market liquidity by injecting orders.
* **WebSocket Gateway (Axum/Rust):** A broadcast layer that streams real-time updates to clients.
* **Redis:** The high-speed message bus and persistence layer.
* **Frontend (Vite/TS):** A React-based visualization of the L2 Orderbook and Trade History.

---

## One-Command Setup

Follow these steps to "hydrate" the environment and launch the entire stack.

### 1. Bootstrap Environment
The project uses template files to manage service coordinates. Copy the examples to create your local environment manifests:

```bash
# From the project root:
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