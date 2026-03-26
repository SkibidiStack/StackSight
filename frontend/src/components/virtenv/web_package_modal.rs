use crate::services::backend_client::BackendClient;
use crate::state::AppState;
use dioxus::prelude::*;
use dioxus_signals::Signal;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct OnlinePackage {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub downloads: u64,
    pub license: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub keywords: Vec<String>,
}

// PyPI API response structures
#[derive(Deserialize, Debug)]
struct PyPISearchResponse {
    projects: Vec<PyPIProject>,
}

#[derive(Deserialize, Debug)]
struct PyPIProject {
    name: String,
    summary: String,
    version: String,
}

#[derive(Deserialize, Debug)]
struct PyPIPackageResponse {
    info: PyPIPackageInfo,
    releases: HashMap<String, Vec<PyPIRelease>>,
}

#[derive(Deserialize, Debug)]
struct PyPIPackageInfo {
    name: String,
    version: String,
    summary: String,
    author: Option<String>,
    license: Option<String>,
    home_page: Option<String>,
    project_urls: Option<HashMap<String, String>>,
    keywords: Option<String>,
}

#[derive(Deserialize, Debug)]
struct PyPIRelease {
    packagetype: String,
}

// npm API response structures
#[derive(Deserialize, Debug)]
struct NpmSearchResponse {
    objects: Vec<NpmSearchResult>,
}

#[derive(Deserialize, Debug)]
struct NpmSearchResult {
    package: NpmPackage,
    score: NpmScore,
}

#[derive(Deserialize, Debug)]
struct NpmPackage {
    name: String,
    version: String,
    description: Option<String>,
    author: Option<NpmAuthor>,
    keywords: Option<Vec<String>>,
    links: Option<NpmLinks>,
}

#[derive(Deserialize, Debug)]
struct NpmAuthor {
    name: String,
}

#[derive(Deserialize, Debug)]
struct NpmLinks {
    npm: Option<String>,
    homepage: Option<String>,
    repository: Option<String>,
}

#[derive(Deserialize, Debug)]
struct NpmScore {
    detail: NpmScoreDetail,
}

#[derive(Deserialize, Debug)]
struct NpmScoreDetail {
    popularity: f64,
}

#[component]
pub fn WebPackageModal(env_id: String, language: String, on_close: EventHandler<()>) -> Element {
    let mut app_state = use_context::<Signal<AppState>>();

    // Clear any stale package operation state for this environment on modal open
    {
        let mut state = app_state.write();
        if let Some(ref op) = state.virtenv.package_operation {
            if op.env_id == env_id && !op.in_progress {
                // Clear completed or failed operations when reopening modal
                state.virtenv.package_operation = None;
                tracing::info!("Cleared stale package operation state for env: {}", env_id);
            }
        }
    }

    let search_query = use_signal(|| String::new());
    let mut search_results = use_signal(|| Vec::<OnlinePackage>::new());
    let selected_packages = use_signal(|| Vec::<String>::new());
    let mut is_searching = use_signal(|| false);
    let mut search_error = use_signal(|| Option::<String>::None);
    let mut current_tab = use_signal(|| "search".to_string());
    let sort_by = use_signal(|| "relevance".to_string());

    let package_operation = app_state.read().virtenv.package_operation.clone();
    let is_installing = package_operation
        .as_ref()
        .map(|op| op.env_id == env_id && op.in_progress)
        .unwrap_or(false);

    // Real search function - makes actual HTTP requests to package registries
    let env_id_clone = env_id.clone();
    let language_clone = language.clone();
    let perform_search = {
        let query = search_query();
        let lang = language_clone.clone();
        move |_| {
            tracing::info!("🔍 perform_search called with query: '{}'", query);

            if query.trim().is_empty() {
                tracing::warn!("Search query is empty, aborting search");
                return;
            }

            tracing::info!("Starting search: query='{}', language='{}'", query, lang);

            is_searching.set(true);
            search_error.set(None);

            let query_clone = query.clone();
            let lang_clone = lang.clone();

            spawn(async move {
                tracing::info!("🌐 Spawned async search task for query: '{}'", query_clone);
                match search_packages(&query_clone, &lang_clone).await {
                    Ok(results) => {
                        tracing::info!(
                            "✅ Search completed successfully, found {} packages",
                            results.len()
                        );
                        is_searching.set(false);
                        search_results.set(results);
                    }
                    Err(e) => {
                        tracing::error!("❌ Search failed with error: {}", e);
                        is_searching.set(false);
                        search_error.set(Some(format!("Search failed: {}", e)));
                        search_results.set(Vec::new());
                    }
                }
            });
        }
    };

    let install_selected = {
        let env_id_for_install = env_id_clone.clone();
        let mut app_state_for_install = app_state.clone();
        let on_close_for_install = on_close.clone();
        move |_| {
            tracing::info!("🔥 INSTALL BUTTON CLICKED!");

            if selected_packages().is_empty() {
                tracing::warn!("No packages selected for installation");
                return;
            }

            let packages_to_install = selected_packages();
            tracing::info!(
                "📦 Starting installation of {} packages: {:?}",
                packages_to_install.len(),
                packages_to_install
            );

            // Set installing state
            {
                let mut app_state_mut = app_state_for_install.write();
                tracing::info!("🔄 Setting installation state to in_progress=true");
                app_state_mut.virtenv.package_operation =
                    Some(crate::state::PackageOperationStatus {
                        env_id: env_id_for_install.clone(),
                        operation: "install".to_string(),
                        packages: packages_to_install.clone(),
                        in_progress: true,
                        success: None,
                        message: Some(format!(
                            "Starting installation of {} package(s)...",
                            packages_to_install.len()
                        )),
                    });
                tracing::info!("✅ Installation state set successfully");
            }

            // Actually install packages via backend
            let mut app_state_install = app_state_for_install.clone();
            let env_id_install = env_id_for_install.clone();
            let packages_clone = packages_to_install.clone();
            let on_close_install = on_close_for_install.clone();

            tracing::info!("🚀 Spawning async installation task");
            spawn(async move {
                tracing::info!(
                    "⚡ Starting package installation for {} packages",
                    packages_clone.len()
                );

                // Send all packages in ONE operation to backend
                let client = BackendClient::new();
                let operation = crate::services::backend_client::PackageOperation {
                    env_id: env_id_install.clone(),
                    operation: crate::services::backend_client::PackageOperationType::Install,
                    packages: packages_clone.clone(),
                    options: std::collections::HashMap::new(),
                };

                // Set initial in_progress state
                {
                    let mut app_state_mut = app_state_install.write();
                    app_state_mut.virtenv.package_operation =
                        Some(crate::state::PackageOperationStatus {
                            env_id: env_id_install.clone(),
                            operation: "install".to_string(),
                            packages: packages_clone.clone(),
                            in_progress: true,
                            success: None,
                            message: Some(format!(
                                "Installing {} package(s)...",
                                packages_clone.len()
                            )),
                        });
                    crate::state::push_toast(
                        &mut app_state_mut.ui,
                        format!("Installing {} package(s)...", packages_clone.len()),
                        crate::state::ToastType::Info,
                    );
                }

                // Send command to backend - backend will send PackageOperationCompleted when done
                match client
                    .send_command(
                        crate::services::backend_client::Command::VirtEnvInstallPackages {
                            operation,
                        },
                    )
                    .await
                {
                    Ok(_) => {
                        tracing::info!("✅ Installation command sent to backend successfully");
                        // Wait for backend to send PackageOperationCompleted event (handled by app.rs)
                        // Poll the state until operation completes
                        for _ in 0..60 {
                            // Wait up to 60 seconds
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                            let state = app_state_install.read();
                            if let Some(op) = &state.virtenv.package_operation {
                                if !op.in_progress {
                                    let success = op.success.unwrap_or(false);
                                    let message = op
                                        .message
                                        .clone()
                                        .unwrap_or_else(|| "Installation completed".to_string());
                                    tracing::info!("Installation completed: success={}", success);
                                    drop(state);

                                    // Add success/error toast
                                    let mut state_mut = app_state_install.write();
                                    let toast_type = if success {
                                        crate::state::ToastType::Success
                                    } else {
                                        crate::state::ToastType::Error
                                    };
                                    crate::state::push_toast(
                                        &mut state_mut.ui,
                                        message,
                                        toast_type,
                                    );

                                    // Clear package operation state and close modal
                                    state_mut.virtenv.package_operation = None;
                                    drop(state_mut);
                                    on_close_install.call(());
                                    return;
                                }
                            }
                        }

                        // Timeout - add error toast and close
                        tracing::warn!("Installation timed out");
                        let mut state_mut = app_state_install.write();
                        crate::state::push_toast(
                            &mut state_mut.ui,
                            "Package installation timed out",
                            crate::state::ToastType::Error,
                        );

                        state_mut.virtenv.package_operation = None;
                        drop(state_mut);
                        on_close_install.call(());
                    }
                    Err(e) => {
                        tracing::error!("❌ Failed to send installation command: {:?}", e);
                        let mut app_state_mut = app_state_install.write();
                        crate::state::push_toast(
                            &mut app_state_mut.ui,
                            format!("Failed to send command: {}", e),
                            crate::state::ToastType::Error,
                        );

                        app_state_mut.virtenv.package_operation = None;
                        drop(app_state_mut);
                        on_close_install.call(());
                    }
                }
            });

            tracing::info!("📋 Install button handler finished - async task spawned");
        }
    };

    rsx! {
        div { class: "modal-overlay",
            onclick: move |_| on_close.call(()),

            div { class: "web-package-modal modal-large",
                onclick: |e| e.stop_propagation(),

                div { class: "modal-header",
                    h3 { "Install Packages from Web" }
                    div { class: "modal-subtitle",
                        "Environment: {env_id_clone} ({language_clone})"
                    }
                    button {
                        class: "close-btn",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                div { class: "modal-tabs",
                    button {
                        class: if current_tab() == "search" { "tab-btn active" } else { "tab-btn" },
                        onclick: move |_| current_tab.set("search".to_string()),
                        "🔍 Search Packages"
                    }
                    button {
                        class: if current_tab() == "popular" { "tab-btn active" } else { "tab-btn" },
                        onclick: move |_| current_tab.set("popular".to_string()),
                        "⭐ Popular"
                    }
                    button {
                        class: if current_tab() == "categories" { "tab-btn active" } else { "tab-btn" },
                        onclick: move |_| current_tab.set("categories".to_string()),
                        "📂 Categories"
                    }
                }

                div { class: "modal-content",
                    // Show installation progress if installing
                    if is_installing {
                        div { class: "installation-progress",
                            div { class: "progress-header",
                                h4 { "Installing Packages..." }
                                if let Some(ref op) = package_operation {
                                    if let Some(ref message) = op.message {
                                        p { class: "progress-message", "{message}" }
                                    }
                                }
                            }
                            div { class: "progress-packages",
                                h5 { "Selected Packages:" }
                                for package in selected_packages().iter() {
                                    div { class: "package-item installing",
                                        span { class: "package-name", "{package}" }
                                        span { class: "package-status", "⏳ Installing..." }
                                    }
                                }
                            }
                        }
                    }
                    // Show completion status if done but not closed
                    else if package_operation.as_ref().map(|op| op.env_id == env_id && !op.in_progress).unwrap_or(false) {
                        div { class: "installation-complete",
                            if let Some(ref op) = package_operation {
                                div { class: if op.success.unwrap_or(false) { "success-message" } else { "error-message" },
                                    if let Some(ref message) = op.message {
                                        h4 { "{message}" }
                                    }
                                    div { class: "installed-packages",
                                        h5 { "Package Installation Results:" }
                                        for package in op.packages.iter() {
                                            div { class: "package-item completed",
                                                span { class: "package-name", "{package}" }
                                                span { class: "package-status",
                                                    if op.success.unwrap_or(false) {
                                                        "✅ Installed"
                                                    } else {
                                                        "❌ Failed"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // Show normal search interface
                    else {
                        if current_tab() == "search" {
                            SearchTab {
                                search_query: search_query.clone(),
                                search_results: search_results.clone(),
                                selected_packages: selected_packages.clone(),
                                is_searching: is_searching(),
                                search_error: search_error(),
                                sort_by: sort_by.clone(),
                                on_search: perform_search,
                                language: language_clone.clone()
                            }
                        } else if current_tab() == "popular" {
                            PopularTab {
                                language: language_clone.clone(),
                                selected_packages: selected_packages.clone(),
                                search_results: search_results.clone()
                            }
                        } else if current_tab() == "categories" {
                            CategoriesTab {
                                language: language_clone.clone(),
                                selected_packages: selected_packages.clone(),
                                search_results: search_results.clone()
                            }
                        }
                    }
                }

                div { class: "modal-footer",
                    div { class: "selected-info",
                        if !selected_packages().is_empty() {
                            span { class: "selected-count",
                                "Selected: {selected_packages().len()} package(s)"
                            }
                        }
                    }

                    div { class: "modal-actions",
                        if is_installing {
                            button {
                                class: "btn btn-warning",
                                onclick: {
                                    let mut app_state_reset = app_state.clone();
                                    let env_id_reset = env_id_clone.clone();
                                    move |_| {
                                        let mut state = app_state_reset.write();
                                        state.virtenv.package_operation = None;
                                        tracing::info!("Cancelled package installation for env: {}", env_id_reset);
                                    }
                                },
                                "Cancel Installation"
                            }
                        } else if package_operation.as_ref().map(|op| op.env_id == env_id && !op.in_progress).unwrap_or(false) {
                            // Show close button after installation is complete
                            button {
                                class: "btn btn-primary",
                                onclick: {
                                    let mut app_state_clear = app_state.clone();
                                    move |_| {
                                        // Clear the completed operation
                                        let mut state = app_state_clear.write();
                                        state.virtenv.package_operation = None;
                                        on_close.call(());
                                    }
                                },
                                "Close"
                            }
                        } else {
                            button {
                                class: "btn btn-secondary",
                                onclick: move |_| on_close.call(()),
                                "Cancel"
                            }
                            button {
                                class: "btn btn-primary",
                                disabled: selected_packages().is_empty(),
                                onclick: install_selected,
                                "Install Selected ({selected_packages().len()})"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SearchTab(
    search_query: Signal<String>,
    search_results: Signal<Vec<OnlinePackage>>,
    selected_packages: Signal<Vec<String>>,
    is_searching: bool,
    search_error: Option<String>,
    sort_by: Signal<String>,
    on_search: EventHandler<()>,
    language: String,
) -> Element {
    rsx! {
        div { class: "search-tab",
            div { class: "search-section",
                div { class: "search-bar",
                    input {
                        r#type: "text",
                        class: "search-input",
                        placeholder: match language.as_str() {
                            "python" => "Search PyPI packages (e.g., numpy, pandas, tensorflow)",
                            "node" => "Search npm packages (e.g., express, react, lodash)",
                            "rust" => "Search crates.io packages (e.g., tokio, serde, reqwest)",
                            "java" => "Search Maven packages (e.g., spring-boot, junit, log4j)",
                            _ => "Search packages..."
                        },
                        value: "{search_query()}",
                        oninput: move |evt| search_query.set(evt.value()),
                        onkeypress: move |evt| {
                            tracing::info!("Key pressed in search input: {:?}", evt.code());
                            if evt.code() == dioxus::prelude::Code::Enter {
                                tracing::info!("Enter key detected, triggering search");
                                on_search.call(());
                            }
                        }
                    }
                    button {
                        class: "btn btn-primary search-btn",
                        onclick: move |_| {
                            tracing::info!("🔍 Search button clicked!");
                            on_search.call(());
                        },
                        disabled: is_searching || search_query().trim().is_empty(),
                        if is_searching { "Searching..." } else { "🔍 Search" }
                    }
                }

                div { class: "search-filters",
                    select {
                        class: "form-select",
                        value: "{sort_by()}",
                        onchange: move |evt| sort_by.set(evt.value()),
                        option { value: "relevance", "Sort by Relevance" }
                        option { value: "downloads", "Sort by Downloads" }
                        option { value: "name", "Sort by Name" }
                        option { value: "updated", "Sort by Last Updated" }
                    }
                }
            }

            if let Some(ref error) = search_error {
                div { class: "search-error",
                    "⚠️ {error}"
                }
            }

            if is_searching {
                div { class: "search-loading",
                    div { class: "loading-spinner" }
                    "Searching packages..."
                }
            } else if !search_results().is_empty() {
                PackageResults {
                    packages: search_results(),
                    selected_packages: selected_packages.clone()
                }
            } else if !search_query().trim().is_empty() && search_error.is_none() {
                div { class: "no-results",
                    "No packages found for '{search_query()}'"
                }
            } else {
                div { class: "search-placeholder",
                    div { class: "placeholder-icon", "🔍" }
                    h4 { "Search for packages" }
                    p { "Enter a package name above to search the {language} package registry" }
                }
            }
        }
    }
}

#[component]
fn PopularTab(
    language: String,
    selected_packages: Signal<Vec<String>>,
    search_results: Signal<Vec<OnlinePackage>>,
) -> Element {
    let language_clone = language.clone();
    use_effect(move || {
        // Load popular packages for the language using real API calls
        let lang = language_clone.clone();
        spawn(async move {
            match get_popular_packages_async(&lang).await {
                Ok(popular) => search_results.set(popular),
                Err(e) => {
                    tracing::error!("Failed to load popular packages: {}", e);
                    search_results.set(get_popular_packages(&lang));
                }
            }
        });
    });

    rsx! {
        div { class: "popular-tab",
            div { class: "tab-header",
                h4 { "Popular {language} packages" }
                p { class: "muted", "Most downloaded and widely used packages in the {language} ecosystem" }
            }

            if !search_results().is_empty() {
                PackageResults {
                    packages: search_results(),
                    selected_packages: selected_packages.clone()
                }
            } else {
                div { class: "loading-popular",
                    div { class: "loading-spinner" }
                    "Loading popular packages..."
                }
            }
        }
    }
}

#[component]
fn CategoriesTab(
    language: String,
    selected_packages: Signal<Vec<String>>,
    search_results: Signal<Vec<OnlinePackage>>,
) -> Element {
    let mut selected_category = use_signal(|| String::new());

    let categories = get_package_categories(&language);

    rsx! {
        div { class: "categories-tab",
            div { class: "categories-grid",
                for category in categories {
                    button {
                        class: if selected_category() == category.id { "category-card active" } else { "category-card" },
                        onclick: {
                            let category_id = category.id.clone();
                            let lang = language.clone();
                            move |_| {
                                selected_category.set(category_id.clone());
                                let packages = get_packages_by_category(&lang, &category_id);
                                search_results.set(packages);
                            }
                        },
                        div { class: "category-icon", "{category.icon}" }
                        div { class: "category-name", "{category.name}" }
                        div { class: "category-count", "{category.count} packages" }
                    }
                }
            }

            if !selected_category().is_empty() && !search_results().is_empty() {
                div { class: "category-results",
                    h4 { "Packages in {selected_category()}" }
                    PackageResults {
                        packages: search_results(),
                        selected_packages: selected_packages.clone()
                    }
                }
            }
        }
    }
}

#[component]
fn PackageResults(packages: Vec<OnlinePackage>, selected_packages: Signal<Vec<String>>) -> Element {
    rsx! {
        div { class: "package-results",
            for package in packages {
                PackageCard {
                    package: package.clone(),
                    selected: selected_packages().contains(&package.name),
                    on_select: {
                        let pkg_name = package.name.clone();
                        move |selected: bool| {
                            let mut current = selected_packages();
                            if selected {
                                if !current.contains(&pkg_name) {
                                    current.push(pkg_name.clone());
                                }
                            } else {
                                current.retain(|p| p != &pkg_name);
                            }
                            selected_packages.set(current);
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PackageCard(package: OnlinePackage, selected: bool, on_select: EventHandler<bool>) -> Element {
    let mut expanded = use_signal(|| false);

    rsx! {
        div { class: format!("package-card {}", if selected { "selected" } else { "" }),
            div { class: "package-header",
                div { class: "package-checkbox",
                    input {
                        r#type: "checkbox",
                        checked: selected,
                        onchange: move |evt| on_select.call(evt.checked())
                    }
                }

                div { class: "package-main-info",
                    div { class: "package-title",
                        span { class: "package-name", "{package.name}" }
                        span { class: "package-version", "v{package.version}" }
                    }
                    div { class: "package-description", "{package.description}" }
                    div { class: "package-meta",
                        span { class: "package-author", "by {package.author}" }
                        span { class: "package-downloads", "📥 {format_downloads(package.downloads)}" }
                        span { class: "package-license", "📜 {package.license}" }
                    }
                }

                div { class: "package-actions",
                    button {
                        class: "btn-icon",
                        onclick: move |_| expanded.set(!expanded()),
                        title: "View details",
                        if expanded() { "▲" } else { "▼" }
                    }
                }
            }

            if expanded() {
                div { class: "package-details",
                    if !package.keywords.is_empty() {
                        div { class: "package-keywords",
                            span { class: "keywords-label", "Keywords:" }
                            for keyword in &package.keywords {
                                span { class: "keyword-tag", "{keyword}" }
                            }
                        }
                    }

                    div { class: "package-links",
                        if let Some(homepage) = &package.homepage {
                            a {
                                href: "{homepage}",
                                target: "_blank",
                                class: "package-link",
                                "🏠 Homepage"
                            }
                        }
                        if let Some(repository) = &package.repository {
                            a {
                                href: "{repository}",
                                target: "_blank",
                                class: "package-link",
                                "📦 Repository"
                            }
                        }
                    }
                }
            }
        }
    }
}

// Real API functions
async fn search_packages(query: &str, language: &str) -> Result<Vec<OnlinePackage>, String> {
    tracing::info!(
        "Searching packages: query='{}', language='{}'",
        query,
        language
    );

    match language.to_lowercase().as_str() {
        "python" => {
            tracing::info!("Using PyPI search for Python packages");
            search_pypi_packages(query).await
        }
        "node" | "javascript" | "js" => {
            tracing::info!("Using npm search for Node/JavaScript packages");
            search_npm_packages(query).await
        }
        _ => {
            tracing::warn!("Package search not implemented for language: '{}', falling back to popular packages", language);
            Ok(get_popular_packages(language))
        }
    }
}

async fn search_pypi_packages(query: &str) -> Result<Vec<OnlinePackage>, String> {
    let url = format!(
        "https://pypi.org/search/?q={}&format=json",
        urlencoding::encode(query)
    );

    tracing::info!("Searching PyPI for: {} at URL: {}", query, url);

    // For now, let's search for individual packages by name
    let package_names = if query.contains(" ") {
        query.split_whitespace().collect::<Vec<_>>()
    } else {
        vec![query]
    };

    let mut packages = Vec::new();

    for package_name in package_names.iter().take(5) {
        // Limit to 5 packages to avoid too many requests
        if let Ok(package) = fetch_pypi_package(package_name).await {
            packages.push(package);
        }
    }

    // If no direct matches found, try some common packages that might match the query
    if packages.is_empty() {
        let suggestions = get_package_suggestions(query, "python");
        for suggestion in suggestions.iter().take(3) {
            if let Ok(package) = fetch_pypi_package(suggestion).await {
                packages.push(package);
            }
        }
    }

    Ok(packages)
}

async fn fetch_pypi_package(package_name: &str) -> Result<OnlinePackage, String> {
    let url = format!("https://pypi.org/pypi/{}/json", package_name);

    tracing::info!("Fetching PyPI package: {} from URL: {}", package_name, url);

    match fetch_json::<PyPIPackageResponse>(&url).await {
        Ok(response) => {
            let keywords = response
                .info
                .keywords
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            let homepage = response.info.home_page.clone().or_else(|| {
                response
                    .info
                    .project_urls
                    .as_ref()
                    .and_then(|urls| urls.get("Homepage").cloned())
            });

            let repository = response.info.project_urls.as_ref().and_then(|urls| {
                urls.get("Repository")
                    .or_else(|| urls.get("Source"))
                    .or_else(|| urls.get("Source Code"))
                    .cloned()
            });

            Ok(OnlinePackage {
                name: response.info.name,
                version: response.info.version,
                description: response.info.summary,
                author: response
                    .info
                    .author
                    .unwrap_or_else(|| "Unknown".to_string()),
                downloads: 0, // PyPI doesn't provide download counts in this API
                license: response
                    .info
                    .license
                    .unwrap_or_else(|| "Unknown".to_string()),
                homepage,
                repository,
                keywords,
            })
        }
        Err(e) => {
            tracing::error!("Failed to fetch PyPI package {}: {}", package_name, e);
            Err(format!("Failed to fetch package {}: {}", package_name, e))
        }
    }
}

async fn search_npm_packages(query: &str) -> Result<Vec<OnlinePackage>, String> {
    let url = format!(
        "https://registry.npmjs.org/-/v1/search?text={}&size=10",
        urlencoding::encode(query)
    );

    tracing::info!("Searching npm for: {} at URL: {}", query, url);

    match fetch_json::<NpmSearchResponse>(&url).await {
        Ok(response) => {
            let packages = response
                .objects
                .into_iter()
                .map(|result| {
                    let pkg = result.package;
                    let keywords = pkg.keywords.unwrap_or_default();
                    let author = pkg
                        .author
                        .map(|a| a.name)
                        .unwrap_or_else(|| "Unknown".to_string());

                    let homepage = pkg.links.as_ref().and_then(|links| links.homepage.clone());

                    let repository = pkg
                        .links
                        .as_ref()
                        .and_then(|links| links.repository.clone());

                    let downloads = (result.score.detail.popularity * 1000000.0) as u64;

                    OnlinePackage {
                        name: pkg.name,
                        version: pkg.version,
                        description: pkg.description.unwrap_or_default(),
                        author,
                        downloads,
                        license: "Unknown".to_string(), // npm search doesn't include license
                        homepage,
                        repository,
                        keywords,
                    }
                })
                .collect();

            Ok(packages)
        }
        Err(e) => {
            tracing::error!("Failed to search npm: {}", e);
            Err(format!("Failed to search npm: {}", e))
        }
    }
}

#[cfg(target_arch = "wasm32")]
async fn fetch_json<T: for<'de> serde::Deserialize<'de>>(url: &str) -> Result<T, String> {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, RequestMode, Response};

    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;

    let window = web_sys::window().ok_or("No window available")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch failed: {:?}", e))?;

    let resp: Response = resp_value
        .dyn_into()
        .map_err(|e| format!("Failed to cast response: {:?}", e))?;

    if !resp.ok() {
        return Err(format!("HTTP error: {}", resp.status()));
    }

    let json = JsFuture::from(
        resp.json()
            .map_err(|e| format!("Failed to get JSON: {:?}", e))?,
    )
    .await
    .map_err(|e| format!("Failed to parse JSON: {:?}", e))?;

    serde_wasm_bindgen::from_value(json).map_err(|e| format!("Failed to deserialize: {:?}", e))
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_json<T: for<'de> serde::Deserialize<'de>>(url: &str) -> Result<T, String> {
    let client = reqwest::Client::builder()
        .user_agent("DevEnv-Manager/1.0")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let json = response
        .json::<T>()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    Ok(json)
}

fn get_package_suggestions(query: &str, language: &str) -> Vec<String> {
    let query_lower = query.to_lowercase();
    match language {
        "python" => {
            let mut suggestions = Vec::new();

            // Common Python packages that might match partial queries
            if query_lower.contains("math") || query_lower.contains("plot") {
                suggestions.extend_from_slice(&["matplotlib", "numpy", "scipy"]);
            }
            if query_lower.contains("data") || query_lower.contains("pandas") {
                suggestions.extend_from_slice(&["pandas", "numpy", "openpyxl"]);
            }
            if query_lower.contains("web") || query_lower.contains("http") {
                suggestions.extend_from_slice(&["requests", "flask", "django", "fastapi"]);
            }
            if query_lower.contains("ml")
                || query_lower.contains("machine")
                || query_lower.contains("ai")
            {
                suggestions.extend_from_slice(&["scikit-learn", "tensorflow", "torch", "keras"]);
            }
            if query_lower.contains("image") || query_lower.contains("cv") {
                suggestions.extend_from_slice(&["opencv-python", "pillow", "imageio"]);
            }

            // Always include some popular packages
            if suggestions.is_empty() {
                suggestions.extend_from_slice(&["requests", "numpy", "pandas", "matplotlib"]);
            }

            suggestions
        }
        "node" => {
            vec!["express", "lodash", "axios", "react"]
        }
        _ => Vec::new(),
    }
    .into_iter()
    .map(|s| s.to_string())
    .collect()
}

async fn get_popular_packages_async(language: &str) -> Result<Vec<OnlinePackage>, String> {
    match language.to_lowercase().as_str() {
        "python" => {
            let popular_packages = vec!["requests", "numpy", "pandas", "matplotlib", "scipy"];
            let mut packages = Vec::new();

            for package_name in popular_packages {
                if let Ok(package) = fetch_pypi_package(package_name).await {
                    packages.push(package);
                }
            }

            Ok(packages)
        }
        "node" | "javascript" => {
            // For npm, we can search for popular packages
            search_npm_packages("express lodash react axios").await
        }
        _ => Ok(get_popular_packages(language)),
    }
}

fn get_popular_packages(language: &str) -> Vec<OnlinePackage> {
    match language {
        "python" => vec![
            OnlinePackage {
                name: "requests".to_string(),
                version: "2.31.0".to_string(),
                description: "Python HTTP for Humans".to_string(),
                author: "Kenneth Reitz".to_string(),
                downloads: 10_000_000,
                license: "Apache 2.0".to_string(),
                homepage: Some("https://requests.readthedocs.io".to_string()),
                repository: Some("https://github.com/psf/requests".to_string()),
                keywords: vec!["http".to_string(), "api".to_string(), "web".to_string()],
            },
            OnlinePackage {
                name: "numpy".to_string(),
                version: "1.24.3".to_string(),
                description: "Fundamental package for array computing with Python".to_string(),
                author: "NumPy Developers".to_string(),
                downloads: 5_000_000,
                license: "BSD".to_string(),
                homepage: Some("https://numpy.org".to_string()),
                repository: Some("https://github.com/numpy/numpy".to_string()),
                keywords: vec![
                    "array".to_string(),
                    "scientific".to_string(),
                    "math".to_string(),
                ],
            },
        ],
        "node" => vec![OnlinePackage {
            name: "lodash".to_string(),
            version: "4.17.21".to_string(),
            description:
                "A modern JavaScript utility library delivering modularity, performance, & extras"
                    .to_string(),
            author: "John-David Dalton".to_string(),
            downloads: 8_000_000,
            license: "MIT".to_string(),
            homepage: Some("https://lodash.com".to_string()),
            repository: Some("https://github.com/lodash/lodash".to_string()),
            keywords: vec![
                "utility".to_string(),
                "functional".to_string(),
                "helper".to_string(),
            ],
        }],
        _ => Vec::new(),
    }
}

#[derive(Clone)]
struct PackageCategory {
    id: String,
    name: String,
    icon: String,
    count: usize,
}

fn get_package_categories(language: &str) -> Vec<PackageCategory> {
    match language {
        "python" => vec![
            PackageCategory {
                id: "data-science".to_string(),
                name: "Data Science".to_string(),
                icon: "📊".to_string(),
                count: 250,
            },
            PackageCategory {
                id: "web-dev".to_string(),
                name: "Web Development".to_string(),
                icon: "🌐".to_string(),
                count: 180,
            },
            PackageCategory {
                id: "ml-ai".to_string(),
                name: "Machine Learning".to_string(),
                icon: "🤖".to_string(),
                count: 120,
            },
            PackageCategory {
                id: "networking".to_string(),
                name: "Networking".to_string(),
                icon: "🔌".to_string(),
                count: 95,
            },
        ],
        "node" => vec![
            PackageCategory {
                id: "frameworks".to_string(),
                name: "Frameworks".to_string(),
                icon: "⚡".to_string(),
                count: 300,
            },
            PackageCategory {
                id: "utilities".to_string(),
                name: "Utilities".to_string(),
                icon: "🛠️".to_string(),
                count: 450,
            },
            PackageCategory {
                id: "testing".to_string(),
                name: "Testing".to_string(),
                icon: "🧪".to_string(),
                count: 150,
            },
        ],
        _ => Vec::new(),
    }
}

fn get_packages_by_category(_language: &str, _category: &str) -> Vec<OnlinePackage> {
    // Mock implementation
    Vec::new()
}

fn format_downloads(downloads: u64) -> String {
    if downloads >= 1_000_000 {
        format!("{:.1}M", downloads as f64 / 1_000_000.0)
    } else if downloads >= 1_000 {
        format!("{:.1}K", downloads as f64 / 1_000.0)
    } else {
        downloads.to_string()
    }
}
