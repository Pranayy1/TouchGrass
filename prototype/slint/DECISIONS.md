# DECISIONS.md

## TouchGrass Prototype - Architectural Decisions

*This document records every significant architectural decision made during the Tauri → Slint migration. Update this file whenever a milestone is completed or a major decision is made.*

---

### Decision 001: Project Location
**Date:** 2025-07-06
**Phase:** Phase 0 - Bootstrap

**Decision:** Place the new Slint project in `prototype/slint/` (sibling to existing Tauri `src-tauri/`src-tauri/`).

**Why:** Keeps the migration isolated from the working Tauri application. Allows side-by-side comparison and gradual migration without breaking the production app.

**Alternatives Considered:**
- Replace Tauri `src-tauri/` in place — rejected (too risky, no rollback)
- Separate repository — rejected (harder to reference shared assets, context switching)

---

### Decision 002: Slint Version
**Date:** 2025-07-06
**Phase:** Phase 0 - Bootstrap

**Decision:** Use Slint **1.17.0** (latest stable as of 2025-06-24).

**Why:** Latest stable provides best performance, bug fixes, and modern API. Uses 2021 edition Rust.

**Alternatives Considered:**
- Slint 1.16.x — rejected (older, no compelling reason to pin)
- Slint main branch — rejected (unstable, breaks frequently)

---

### Decision 003: UI Definition Approach
**Date:** 2025-07-06
**Phase:** Phase 0 - Bootstrap

**Decision:** Use `.slint` files with `slint_build::compile()` in `build.rs` (not inline `slint!` macro).

**Why:**
- Separation of concerns: UI markup separate from Rust logic
- Better IDE support (Slint LSP, syntax highlighting)
- Easier designer/developer collaboration
- Cleaner compilation model (generated code in `OUT_DIR`)
- Standard Slint best practice

**Alternatives Considered:**
- Inline `slint! { }` macro — rejected (poor IDE support, mixes concerns)
- Dynamic `slint::compile!()` at runtime — rejected (slower startup, no compile-time checks)

---

### Decision 004: Project Structure
**Date:** 2025-07-06
**Phase:** Phase 0 - Bootstrap (Enhanced 2025-07-07 for Production Readiness)

**Decision:** Organize code into layered architecture with clear separation of concerns:

```
prototype/slint/
├── Cargo.toml
├── build.rs
├── src/
│   ├── lib.rs          # Library re-exports all modules
│   ├── main.rs         # Application entry point
│   ├── core/           # Business logic and domain models
│   ├── platform/       # Platform-specific abstractions
│   ├── models/         # Data structures and DTOs
│   ├── services/       # Application services and use cases
│   ├── utils/          # Cross-cutting utilities
│   └── assets/         # Static assets (if any)
│
└── ui/
    ├── components/     # Reusable UI components
    └── windows/        # Window and dialog definitions
```

**Why:** Professional, scalable structure following clean architecture principles. Separates:
- **UI Layer** (`ui/`) — Declarative UI components (.slint files)
- **Application Layer** (`src/main.rs`) — Entry point and window creation
- **Domain Layer** (`src/core/`) — Business logic independent of UI/platform
- **Interface Adapters** (`srcers/`) — DTOs and service
- **Frameworks & Drivers** (`src/platform/`, `src/assets/`) — Platform specifics and external interfaces

This follows hexagonal/clean architecture principles, making the core business logic testable and independent of UI frameworks.

**Alternatives Considered:**
- Flat structure — rejected (doesn't scale)
- Feature-based grouping initially — rejected (layered architecture better separates concerns for maintainability)
- Mixing UI and business logic — rejected (violates separation of concerns, hard to test)

---

### Decision 005: Window Title & Content
**Date:** 2025-07-06
**Phase:** Phase 0 - Bootstrap

**Decision:** Window title = "TouchGrass Prototype", centered text = "TouchGrass Prototype". No additional UI.

**Why:** Matches requirement exactly. Minimal viable window to verify the toolchain works.

**Alternatives Considered:** None — requirement was explicit.

---

### Decision 006: Rust Edition
**Date:** 2025-07-06
**Phase:** Phase 0 - Bootstrap

**Decision:** Use **Rust 2021 edition**.

**Why:** Required by Slint 1.17.0. Modern edition with better ergonomics.

**Alternatives Considered:**
- Rust 2018 — rejected (Slint requires 2021+)

---

### Decision 007: Build System
**Date:** 2025-07-06
**Phase:** Phase 0 - Bootstrap

**Decision:** Standard Cargo + `slint-build` in `build.rs`.

**Why:** Native Rust build integration. No external build tools (npm, make, etc.) needed.

**Alternatives Considered:**
- Just `cargo build` with inline slint! — rejected (see Decision 003)
- Custom build script without slint-build — rejected (reinventing the wheel)

---

### Decision 008: No Tauri Dependencies
**Date:** 2025-07-06
**Phase:** Phase 0 - Bootstrap

**Decision:** Zero Tauri dependencies in the Slint prototype. Pure Slint + Rust.

**Why:** Explicit requirement: "Do NOT modify any existing Tauri code." Clean break for migration.

**Alternatives Considered:**
- Reuse Tauri Rust backend — rejected (couples prototype to Tauri architecture)
- Shared workspace — rejected (complicates isolation)

---

### Decision 009: Documentation as Source of Truth
**Date:** 2025-07-06
**Phase:** Phase 0 - Bootstrap

**Decision:** Maintain `ROADMAP.md`, `TODO.md`, `DECISIONS.md` as living documents. Update at every milestone.

**Why:** Migration is multi-phase. These docs prevent knowledge loss, enable handoff, and track rationale.

**Alternatives Considered:**
- Code comments only — rejected (hard to get holistic view)
- External wiki — rejected (drift from code, extra tool)
- Git commits only — rejected (hard to query, no rationale)

---

### Decision 010: Warning-Free Compilation
**Date:** 2025-07-06
**Phase:** Phase 0 - Bootstrap

**Decision:** Project must compile with zero warnings (`cargo run` clean).

**Why:** Enforces code quality from day one. Catches API misuse early.

**Alternatives Considered:**
- Allow warnings initially — rejected (technical debt accumulates)
- `#![allow(unused)]` — rejected (hides real issues)

---

### Decision 011: Layered Architecture
**Date:** 2025-07-07
**Phase:** Phase 0 - Bootstrap (Enhanced for Production)

**Decision:** Implemented layered architecture with clearly defined responsibilities:
- **core/** - Business logic, domain models, application services (UI/platform independent)
- **platform/** - Platform abstractions and implementations (file system, hardware, OS integration)
- **models/** - Data structures, DTOs, API models, view models
- **services/** - Application services orchestrating use cases
- **utils/** - Cross-cutting utilities, helpers, extension traits
- **assets/** - Static assets (icons, images, fonts, etc.)
- **ui/components/** - Reusable UI components (buttons, inputs, cards, etc.)
- **ui/windows/** - Window and dialog definitions (main window, settings, dialogs)

**Why:** Creates maintainable, testable codebase that separates concerns effectively. Follows industry best practices for desktop applications. Enables:
- Independent testing of business logic
- Platform isolation (easier to port to different OSes)
- Clear dependency flow (outer layers depend on inner layers, never vice versa)
- Team scalability (different teams can work on different layers)
- Code reusability (components and services can be reused)

**Alternatives Considered:**
- Monolithic structure — rejected (becomes unmaintainable as app grows)
- Feature-first structure — rejected (better for small apps, creates coupling in larger apps)
- MVC/MVVM patterns — rejected (too prescriptive; layered architecture is more flexible and follows Clean/Hexagonal principles better)

---

### Decision 012: Library Organization
**Date:** 2025-07-07
**Phase:** Phase 0 - Bootstrap (Enhanced for Production)

**Decision:** Created `src/lib.rs` to re-export all internal modules, providing a clean API for the binary crate and potential future library usage.

**Why:** 
- Provides organized public API surface
- Enables potential future extraction of core logic as a library
- Improves code organization and discoverability
- Follows Rust best practices for multi-module crates

**Alternatives Considered:**
- No lib.rs, direct module paths in main.rs — rejected (less clean, harder to refactor)
- Feature-gated modules — rejected (over-engineering for current scope)

---

*End of decisions for Phase 0. Next decision will be recorded at Phase 1 completion.*