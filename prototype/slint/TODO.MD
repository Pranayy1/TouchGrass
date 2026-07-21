# TODO.md

## TouchGrass Prototype - Migration Task Checklist

### Phase 0: Bootstrap ✅ COMPLETE
- [x] Initialize Cargo project in `prototype/slint/`
- [x] Add `slint` and `slint-build` dependencies (v1.17.0)
- [x] Create `build.rs` for Slint compilation
- [x] Create `ui/app_window.slint` with centered "TouchGrass Prototype" text
- [x] Create `src/main.rs` with `slint::include_modules!()` and app entry point
- [x] Verify `cargo run` compiles without warnings and launches window

---

### Phase 1: Core UI ⏳ PENDING
#### Milestone 1.1: Application Shell
- [ ] Create main window component with header, sidebar, content area
- [ ] Implement responsive layout (desktop/mobile breakpoints)
- [ ] Add navigation sidebar with icons and labels
- [ ] Create header with app title, window controls (min/max/close)

#### Milestone 1.2: Theme System
- [ ] Define color tokens (light/dark mode)
- [ ] Implement theme switching (system/light/dark)
- [ ] Add CSS-like global styles for consistency

#### Milestone 1.3: Reusable Components
- [ ] Button variants (primary, secondary, ghost, destructive)
- [ ] Input fields, textareas, selects
- [ ] Card, modal, dropdown, tooltip components
- [ ] Loading states, skeletons, empty states

---

### Phase 2: State Management ⏳ PENDING
#### Milestone 2.1: Settings & Persistence
- [ ] Port Tauri settings store to Slint (file-based or native)
- [ ] Window state persistence (position, size, maximized)
- [ ] User preferences (theme, language, notifications)

#### Milestone 2.2: Application State
- [ ] Global state management pattern (signals, stores)
- [ ] Async task handling with proper loading/error states
- [ ] Inter-component communication

---

### Phase 3: Feature Parity ⏳ PENDING
#### Milestone 3.1: Core Features
- [ ] Port all Tauri commands to pure Rust/Slint equivalents
- [ ] File system operations (native dialogs, drag-drop)
- [ ] System tray integration
- [ ] Global keyboard shortcuts
- [ ] Notifications (native OS notifications)

#### Milestone 3.2: Domain Features
- [ ] [List specific features from Tauri app here]

---

### Phase 4: Polish & Optimize ⏳ PENDING
- [ ] Animations & transitions (page transitions, micro-interactions)
- [ ] Performance profiling & optimization
- [ ] Accessibility (keyboard nav, screen readers, contrast)
- [ ] Cross-platform testing (Windows, macOS, Linux)
- [ ] Automated tests (unit, integration, visual regression)

---

### Phase 5: Distribution ⏳ PENDING
- [ ] Build configuration (release profiles, optimization)
- [ ] Installer creation (MSI, DMG, AppImage, deb/rpm)
- [ ] Code signing & notarization
- [ ] Auto-update mechanism
- [ ] CI/CD pipeline for releases