# Yew PWA Development Rules

Follow these rules when writing or modifying this project.

## Project Context

- This repository is a reusable template for small personal progressive web apps.
- The application is a client-side Yew app compiled to WebAssembly with `wasm-pack`.
- GitHub Pages serves the generated `docs` directory.
- Development happens in root source files; `docs` contains generated deployment output.

## General Priorities

- Prefer clear, explicit Rust over clever, overly concise, or highly abstract code.
- Optimize for code that can be understood locally without tracing through unnecessary layers.
- Make only changes required by the current task and preserve unrelated user changes.
- Follow existing project conventions unless they conflict with correctness or these rules.
- Keep implementations as simple as current requirements allow; avoid speculative flexibility.

## Source and Generated Files

- Treat root files as the source of truth and `docs` as generated GitHub Pages output.
- Do not edit files in `pkg` or `docs` manually; regenerate them with `build_and_sync.bat`.
- Keep local certificates, private keys, editor state, and machine-specific configuration out of Git.
- Never embed secrets, private API keys, passwords, or privileged credentials in client code or generated files.
- Keep source code separate from generated WebAssembly, JavaScript, CSS, and deployment artifacts.
- Commit `Cargo.lock` for applications, but update it through Cargo rather than editing it manually.
- Resolve batch-script paths relative to `%~dp0` and quote paths that may contain spaces.
- Make build scripts fail fast and preserve exit codes when build, copy, or synchronization commands fail.
- Keep a non-watching build command that runs reproducibly without launching servers or persistent windows.

## Functions and Control Flow

- Each function should perform one coherent task at one consistent level of abstraction.
- Review functions over roughly 40 lines for possible decomposition, but do not treat length as a hard limit.
- Extract a function only when it names a meaningful concept, hides complexity, separates a side effect, enables testing, reduces nesting, or provides genuine reuse.
- Do not create trivial pass-through wrappers; high-level workflows should read from top to bottom as meaningful steps.
- Prefer guard clauses, early returns, and `?` over deeply nested conditionals.
- Use `match` when alternatives matter to understanding behavior.
- Keep iterator chains simple; use named intermediate values or ordinary loops when they improve readability.

## Naming and Organization

- Use descriptive domain names for functions, variables, types, enum variants, modules, and fields.
- Avoid vague names such as `data`, `manager`, `helper`, or `process`; short local names are acceptable when context makes them obvious.
- Function names must reveal important side effects.
- Keep modules focused on one domain concept or closely related responsibility.
- Avoid allowing a module to accumulate UI rendering, business rules, browser storage, networking, and unrelated utilities; separate responsibilities when the module becomes difficult to understand locally.
- Do not introduce layers that merely forward calls without adding meaning, policy, transformation, or isolation.

## Domain Modeling

- Use structs and enums when they make domain concepts and states clearer.
- Use enums instead of combinations of booleans when representing states, modes, or alternatives.
- Make invalid states difficult to construct when doing so remains reasonably simple.
- Keep fields private and add validated construction when a type has invariants.
- Do not create wrapper types unless they add meaning, validation, safety, or a useful interface.

## Ownership, State, and Mutation

- Prefer straightforward ownership over complicated borrowing or explicit lifetimes.
- Borrow values when the ownership relationship is simple and local.
- Inexpensive clones are acceptable when they materially simplify Yew callbacks or asynchronous ownership.
- Do not clone large values merely to avoid designing ownership clearly.
- Keep mutation local and obvious; avoid shared mutable state unless the UI genuinely requires it.
- Do not introduce `Rc<RefCell<_>>`, `Arc<Mutex<_>>`, or similar tools without a genuine shared-ownership need.

## Interfaces and Errors

- Prefer no more than four function parameters when practical; group values only when they form one coherent concept.
- Avoid positional boolean parameters and vague `Context`, `Options`, or `Config` structures.
- Use `Option<T>` for meaningful absence and `Result<T, E>` for failures callers should handle.
- Do not use `unwrap()` or `expect()` in production paths unless the invariant is guaranteed.
- An `expect()` message must explain the violated invariant, not merely repeat that an operation failed.
- Add useful context at parsing, storage, networking, initialization, and other external boundaries.
- Use typed errors when callers need to distinguish failure cases; do not silently discard errors.
- Logging an error is not handling it unless continuing is intentional and safe.
- Document retry, fallback, partial-success, and failure behavior when it affects users or callers.

## Abstractions and Unsafe Code

- Prefer concrete types until multiple real implementations or a genuine testing boundary justifies abstraction.
- Keep traits small and do not introduce one for a single implementation without a clear benefit.
- Avoid deeply nested generic types and complicated trait bounds unless the problem requires them.
- Do not use macros, factories, registries, or dependency-injection frameworks when simpler Rust is sufficient.
- Do not introduce `unsafe` unless the task cannot reasonably be completed with safe Rust.
- If `unsafe` is unavoidable, keep it small, document its invariants, and test the boundary.

## Yew and Browser Code

- Keep components focused on one screen, visible region, or interaction.
- Keep business rules in ordinary Rust functions that can be tested without rendering the UI.
- Use Yew state only for values that affect rendering or interaction.
- Call hooks only at the top level of function components or custom hooks, and never conditionally.
- Do not store values in component state when they can be derived clearly from existing state or properties.
- Use stable domain identifiers as keys when rendering dynamic lists.
- Represent asynchronous UI state with an enum such as `Idle`, `Loading`, `Loaded`, and `Failed` rather than overlapping booleans.
- Keep callbacks focused on one user action and avoid capturing more state than they need.
- Keep browser APIs, storage, networking, clocks, and JavaScript interop at clear boundaries.
- Treat URL parameters, imported files, browser storage, network responses, and user-entered content as untrusted input.
- Avoid rendering unchecked HTML; sanitize externally supplied HTML before displaying it.
- Do not log sensitive or personally identifying user data.
- Prefer semantic HTML and accessible controls with labels, keyboard behavior, and visible focus states.
- Ensure important interactions do not depend solely on pointer or touch input.
- Treat browser storage formats as persistent schemas and migrate them when their shape changes.
- Handle unavailable, malformed, or outdated stored values without panicking.
- Make asynchronous task ownership, cancellation, and error handling clear when tasks can outlive a callback.

## Sass and Styling

- Write source styles in `static/style.scss`; do not edit generated CSS directly.
- Organize Sass by component or visible page region.
- Use nesting only when selectors share a real structural relationship.
- Keep selector nesting shallow, normally no more than three levels, and do not reproduce the entire DOM tree.
- Keep state selectors such as `:hover`, `:focus-visible`, and modifier classes beside their component.
- Extract repeated colors, spacing, and sizing into clearly named variables after repetition is established.
- Prefer reusable classes over element IDs and overly specific selectors.
- Keep mobile layouts, larger text, focus, disabled, loading, error, and empty states understandable.

## PWA and GitHub Pages

- Keep app paths relative because GitHub Pages may host the app below a repository path.
- If client-side routing is introduced, test direct navigation and refreshes under the GitHub Pages repository path.
- Keep app title, manifest identity, theme colors, icons, and Cargo metadata accurate.
- Do not add a service worker without an explicit cache-update strategy.
- When offline support is enabled, test first load, refresh, upgrade, cache invalidation, and offline fallback behavior.
- Do not cache non-GET requests or development-tool requests.
- Ensure failed network requests cannot overwrite valid cached or locally stored user data.

## Dependencies and Documentation

- Prefer the standard library and existing dependencies when they solve the problem clearly.
- Add crates only for current functionality and explain why each new dependency is needed.
- Remove stale template dependencies and keep `Cargo.toml` metadata accurate.
- Write comments that explain why a decision exists rather than restating the code.
- Document public APIs, important domain types, invariants, errors, and non-obvious side effects.
- Record browser workarounds, storage formats, and important business rules near the affected code.

## Testing

- Keep pure business logic independently testable.
- Test observable behavior rather than private implementation details.
- Cover normal cases, boundaries, expected failures, important state transitions, and previously discovered bugs.
- Prefer direct tests, small fakes, or in-memory implementations over excessive mocking.
- Test storage parsing and migrations when persistent formats change.
- Do not add tests that only duplicate compiler or type-system guarantees.

## Validation and Completion

- Run `cargo fmt` after changing Rust code, then run `cargo fmt --check`, `cargo check`, and `cargo test` as relevant.
- Run `cargo clippy --all-targets --all-features` before completing substantial Rust changes.
- Address relevant Clippy warnings rather than automatically silencing them.
- Run `wasm-pack build --target web` when changes can affect the browser build.
- Run `build_and_sync.bat` before committing deployment output and confirm generated files match their sources.
- Review the result for unnecessary abstractions, wrappers, clones, shared state, and indirection.
- Confirm ownership, error behavior, side effects, and the main execution path are easy to understand.
- Remove stale template references exposed by the change.
- Summarize changed files, important decisions, validation results, and remaining risks.
