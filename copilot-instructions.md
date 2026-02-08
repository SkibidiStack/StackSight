# DevEnv Manager Prompt

Goal: Build a 100% Rust-based application that manages Docker containers, virtual environments, and system health. Use the attached backend/frontend architectures as the baseline. Prioritize strong separation of concerns, async operations (Tokio), safety, and cross-platform support. Deliverables must ship as a single self-contained executable per platform (no zip extractions or multi-file installers).

# DISCLAIMER: ALWAYS DOUBLE CHECK THE FILES YOU EDIT FOR ERRORS ALWAYS REBUILD THE APPS TO MAKE SURE THERE ARE NO ERRORS

# DISCLAIMER: WHENEVER ADDING ANY PLACEHOLDER CODE MAKE SURE TO LET THE USER KNOW THAT THERE IS PLACEHOLDER CODE AND PREFERABLY DONT USE PLACEHOLDER CODE WHATSOEVER ALWAYS TRY TO CREATE THE ACTUAL FUNCTIONALITY REQUESTED

## Product Scope
- Docker Dashboard: create/start/stop/restart/remove containers; pull/build/tag images; manage networks/volumes; view logs/stats; generate docker-compose files.
- Virtual Environments: create/activate/deactivate/list/clone/remove environments for Python (incl. scientific stacks like scikit-learn), Rust, Node, Go, .NET, Java, etc.; manage package installs/updates with pip/conda/poetry, cargo, npm/yarn/pnpm, go mod, nuget; support premade templates/folders and dependency resolution.
- System Health: resource metrics (CPU/mem/disk/net/temps), process management, alerts, performance collector, health checks.

## Backend Architecture (Rust)
READ BACKEND ARCHITECTURE FILE /home/shaun/StackSight/StackSight/README-DevEnv-Manager-Backend-Architecture.md
- Core: service manager, event bus, error handling (anyhow), config, structured logging.
- Services:
  - Docker: Bollard client, container/image/network/volume managers, compose generator, registry client, stats collector.
  - Virtual Environments: environment manager; language handlers (python/node/rust/go/dotnet/java, extensible); package managers (pip/npm/cargo/go/nuget); templates; dependency resolver.
  - System: resource monitor, process manager, performance collector, alert engine, health checker.
  - File System: operations, path resolver, watcher, permission manager, project detector.
  - Config: settings/profile/theme/plugin config, migration.
  - Communication: frontend bridge, event dispatcher, websocket server, IPC, optional HTTP API.
- Platform Abstraction: windows/linux/macos modules for process/package/service/shell specifics.
- Models/Utils/Tests: typed data models; command/file/network/crypto/compression/validation utilities; unit/integration tests.

## Frontend Architecture (Rust + Dioxus/Tauri)
READ FRONTEND ARCHITECTURE FILE /home/shaun/StackSight/StackSight/README-DevEnv-Manager-Frontend-Architecture.md
- Routing: dashboard, Docker management, virtual environments, monitoring, settings.
- Components: container list/detail, image manager, compose builder, network/volume managers, logs/stats; environment list/detail, language/version selector, package manager UI, project wizard, template manager, dependency viewer; monitoring charts/alerts/process monitor; common sidebar/header/modals/toasts/file browser/terminal.
- State: signals-based reactive state; global slices for user prefs, system status, Docker state, virtual env state, UI state.
- UX: realtime updates (2-3s for critical metrics), optimistic actions, validation, accessibility, responsive layouts.

## Key Flows to Support
1) Docker: list containers/images; pull/build; create containers with advanced config; start/stop/restart/remove; stream logs; exec into container; network/volume operations; generate/export compose; registry auth.
2) Virtual Envs: detect language; select version; create env with template (e.g., Python data-science preset with numpy/pandas/scikit-learn); install packages; activate/deactivate; clone/export; link to project; show health/resource usage.
3) Health: continuous metrics; alert rules; anomaly detection; process tree; kill/terminate; performance snapshots.

## Suggested Improvements
- Observability: add structured tracing (tracing + OpenTelemetry) and metric exporters; optional self-hosted/Prometheus endpoint.
- Security: secrets vault for registry creds and API keys; image signing/verification; policy-based container constraints; sandboxed script execution.
- Reliability: circuit breakers/retries on Docker/registry calls; backpressure on event streams; health probes for services; graceful degradation when Docker daemon is down.
- Offline/Cache: package manager mirror caching; offline install for curated stacks; content-addressed cache for templates.
- Templates: user-defined stack templates for containers and virtual envs (compose snippets + env presets + folder scaffolds); gallery with version pinning.
- Automation: task recipes (e.g., "build image + run migrations + start stack"); scheduled cleanups for old images/volumes/envs.
- Testing: contract tests for Docker/registry interactions via mock server; fixture-based env creation tests per language; cross-platform CI matrix.
- UX: bulk actions with progress; diff view for generated compose; preflight checks before creating containers/envs; onboarding tour and context help.

## Acceptance Criteria
- Backend crates compile on Windows/Linux/macOS; async-first; robust error contexts; unit/integration tests for core services.
- Frontend provides responsive Docker dashboard, env manager, and monitoring views with realtime updates and optimistic UI.
- Compose generator and env templates are round-trippable: UI -> model -> files -> reload.
- Security and stability: validated inputs, guarded shell execution, protected secrets, graceful handling when Docker or package managers are unavailable.
- Packaging: produce a single executable artifact per OS (e.g., statically linked where feasible, or Tauri single-binary bundles) that runs without auxiliary installers or zip extraction.

## Next Steps
- Scaffold workspace per backend/frontend folder layout; set up workspaces/crates for services and shared models.
- Implement event bus + logging + config first; then Docker service happy-path (list/inspect/start/stop); then env creation for Python/Rust + template support; then monitoring pipeline.
- Add tracing/metrics, tests, and UI wiring with Dioxus/Tauri.
- Decide packaging path early: target single-file outputs via `cargo bundle`/Tauri single-binary mode; prefer static linking on Linux and embed assets/resources to avoid external folders.
