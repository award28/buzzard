# 🦅 buzzard

> A lightweight, DDD-first message bus framework for Rust — orchestrate commands, events, and projections with confidence.

---

**buzzard** is a strongly typed, domain-driven orchestration framework for message-based applications in Rust. Inspired by hexagonal and DDD patterns, it lets you model business flows with explicit **Command**, **Event**, and **Projection** types while remaining infrastructure-agnostic.

---

## ✨ Features

- 🧠 Domain-driven: Clean separation of Commands, Events, and Projections
- 🔄 Fully asynchronous and `Send + Sync + Clone` safe
- 📦 Broker-agnostic (`Redis`, `Postgres`, `NATS`, etc.)
- 🧱 Extensible via `MessageBusDriver`, `CommandHandler`, `Policy`, and `Projector`
- 🚀 Easy to integrate into web servers, CLI apps, and background workers

---

## 📦 Quickstart Example

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let driver = MyDriver::init().await?;
    let bus = MessageBus::from(&driver);

    // Run bus in the background
    let background = tokio::spawn(bus.clone().start());

    // Dispatch a command from a web handler, CLI, etc.
    let cmd = MyCommand { sku: "ABC-123".into() };
    let response = bus.dispatch(cmd).await?;

    background.await??;
    Ok(())
}
```

---

## 🧠 Message Flow


                                ┌────────────────────────┐
                                │      Message Bus       │
                                └────────────┬───────────┘
                                             │
            ┌────────────────────────────────┼────────────────────────────────┐
            │                                │                                │
            ▼                                ▼                                ▼
    ┌───────────────┐              ┌─────────────────────┐             ┌────────────────────┐
    │ Command Msg   │              │    Event Msg        │             │ Projection Msg     │
    └──────┬────────┘              └─────────┬───────────┘             └──────────┬─────────┘
           │                                 │                                    │
           ▼                                 ▼                                    ▼
    ┌───────────────┐              ┌────────────────────┐             ┌────────────────────────┐
    │ CommandHandler│              │      Policy        │             │       Projector        │
    └──────┬────────┘              └────────┬───────────┘             └───────────┬────────────┘
           │ domain mutation                │ maps event to side effects          │
           ▼                                ▼                                     │
    ┌────────────────────┐      ┌────────────────────────────┐                    │
    │     UnitOfWork     │      │ Vec<SideEffect<Cmd, Proj>> │                    │
    │ - apply mutations  │      └─────────────┬──────────────┘                    │
    │ - capture events   │                    │                                   │
    │ - commit/rollback  │                    │                                   │
    └─────────┬──────────┘                    │                                   │
              │ commits                       │ publishes                         │
              ▼                               ▼                                   ▼
      ┌──────────────┐        ┌────────────────────────────┐         ┌─────────────────────────────┐
      │ Event Msg(s) │───────►│ Message Bus (loopback)     │         │ Projection Msgs from Policy │
      └──────────────┘        └────────────────────────────┘         └─────────────────────────────┘


---

## 📚 Traits You Implement

| Trait              | Description                                |
|-------------------|--------------------------------------------|
| `MessageBusDriver`| Declares domain types + infrastructure     |
| `CommandHandler`  | Executes domain logic with mutation        |
| `Policy`          | Reacts to events with follow-up messages   |
| `Projector`       | Handles read model and infra updates       |

---

## 🔄 Lifecycle Summary

1. `dispatch(command)` → handled in `UnitOfWork`
2. Domain events are emitted and published
3. Events trigger `Policy` logic → follow-up messages
4. Messages include new commands or projections
5. Projections handled by external-side `Projector`

---

## ✅ When to Use

- You want a clean, extensible message-processing runtime
- You use DDD / CQRS and want to model each message explicitly
- You want to test your business logic independently of infrastructure

---

## 🔗 Built With

- `futures`, `anyhow`
- Your backend: PostgreSQL, Redis, NATS, MeiliSearch...
