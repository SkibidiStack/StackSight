use dioxus::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct DependencyNode {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<String>,
    pub dependents: Vec<String>,
    pub is_direct: bool,
    pub size: Option<u64>,
}

#[component]
pub fn DependencyViewer(env_id: String) -> Element {
    let mut view_mode = use_signal(|| "tree".to_string());
    let mut selected_package = use_signal(|| None::<String>);
    let mut search_query = use_signal(|| String::new());
    
    // Mock dependency data
    let dependencies = get_mock_dependencies();
    
    let filtered_deps: Vec<_> = dependencies.iter()
        .filter(|(name, _)| {
            search_query().is_empty() || name.to_lowercase().contains(&search_query().to_lowercase())
        })
        .map(|(name, dep)| (name.clone(), dep.clone()))
        .collect();
    
    rsx! {
        div { class: "dependency-viewer",
            div { class: "viewer-header",
                h2 { "Dependency Viewer" }
                div { class: "viewer-controls",
                    div { class: "view-modes",
                        button { 
                            class: format!("btn {}", if view_mode() == "tree" { "active" } else { "" }),
                            onclick: move |_| view_mode.set("tree".to_string()),
                            "🌳 Tree"
                        }
                        button { 
                            class: format!("btn {}", if view_mode() == "graph" { "active" } else { "" }),
                            onclick: move |_| view_mode.set("graph".to_string()),
                            "🔗 Graph"
                        }
                        button { 
                            class: format!("btn {}", if view_mode() == "list" { "active" } else { "" }),
                            onclick: move |_| view_mode.set("list".to_string()),
                            "📄 List"
                        }
                    }
                    input {
                        r#type: "text",
                        class: "search-input",
                        placeholder: "Search dependencies...",
                        value: "{search_query()}",
                        oninput: move |evt| search_query.set(evt.value())
                    }
                }
            }
            
            div { class: "viewer-content",
                match view_mode().as_str() {
                    "tree" => rsx! { TreeView { dependencies: dependencies.clone(), search_query: search_query() } },
                    "graph" => rsx! { GraphView { dependencies: dependencies.clone() } },
                    "list" => rsx! { ListView { dependencies: filtered_deps } },
                    _ => rsx! { div { "Unknown view mode" } }
                }
            }
            
            if let Some(package) = selected_package() {
                PackageDetails { 
                    package_name: package,
                    dependencies: dependencies.clone(),
                    on_close: move |_| selected_package.set(None)
                }
            }
        }
    }
}

#[component]
fn TreeView(dependencies: HashMap<String, DependencyNode>, search_query: String) -> Element {
    let root_packages: Vec<_> = dependencies.values()
        .filter(|dep| dep.is_direct)
        .filter(|dep| search_query.is_empty() || dep.name.to_lowercase().contains(&search_query.to_lowercase()))
        .collect();
    
    rsx! {
        div { class: "tree-view",
            if root_packages.is_empty() {
                div { class: "empty-state",
                    "No dependencies found"
                }
            } else {
                for package in root_packages {
                    TreeNode { 
                        node: package.clone(),
                        dependencies: dependencies.clone(),
                        level: 0
                    }
                }
            }
        }
    }
}

#[component]
fn TreeNode(node: DependencyNode, dependencies: HashMap<String, DependencyNode>, level: usize) -> Element {
    let mut expanded = use_signal(|| level == 0);
    
    let indent_style = format!("margin-left: {}px", level * 20);
    let has_children = !node.dependencies.is_empty();
    
    rsx! {
        div { class: "tree-node", style: "{indent_style}",
            div { class: "node-header",
                if has_children {
                    button { 
                        class: "expand-btn",
                        onclick: move |_| expanded.set(!expanded()),
                        if expanded() { "▼" } else { "▶" }
                    }
                }
                span { class: "package-name", "{node.name}" }
                span { class: "package-version", "v{node.version}" }
                if node.is_direct {
                    span { class: "direct-badge", "Direct" }
                }
                if let Some(size) = node.size {
                    span { class: "package-size", "{size} KB" }
                }
            }
            
            if expanded() && has_children {
                div { class: "node-children",
                    for dep_name in &node.dependencies {
                        if let Some(dep_node) = dependencies.get(dep_name) {
                            TreeNode { 
                                node: dep_node.clone(),
                                dependencies: dependencies.clone(),
                                level: level + 1
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn GraphView(dependencies: HashMap<String, DependencyNode>) -> Element {
    rsx! {
        div { class: "graph-view",
            div { class: "graph-placeholder",
                div { class: "placeholder-icon", "📈" }
                div { class: "placeholder-title", "Dependency Graph" }
                div { class: "placeholder-description", 
                    "Interactive dependency graph visualization would be rendered here using a graph library like D3.js or vis.js"
                }
            }
        }
    }
}

#[component]
fn ListView(dependencies: Vec<(String, DependencyNode)>) -> Element {
    rsx! {
        div { class: "list-view",
            div { class: "list-header",
                div { class: "header-item", "Package" }
                div { class: "header-item", "Version" }
                div { class: "header-item", "Type" }
                div { class: "header-item", "Size" }
                div { class: "header-item", "Dependencies" }
            }
            div { class: "list-items",
                for (_, dep) in dependencies {
                    div { class: "list-item",
                        div { class: "item-cell", "{dep.name}" }
                        div { class: "item-cell", "v{dep.version}" }
                        div { class: "item-cell",
                            if dep.is_direct { "Direct" } else { "Transitive" }
                        }
                        div { class: "item-cell",
                            if let Some(size) = dep.size {
                                "{size} KB"
                            } else {
                                "Unknown"
                            }
                        }
                        div { class: "item-cell", "{dep.dependencies.len()}" }
                    }
                }
            }
        }
    }
}

#[component]
fn PackageDetails(package_name: String, dependencies: HashMap<String, DependencyNode>, on_close: EventHandler<()>) -> Element {
    let package = dependencies.get(&package_name);
    
    match package {
        Some(pkg) => rsx! {
            div { class: "modal-overlay",
                div { class: "package-details-modal",
                    div { class: "modal-header",
                        h3 { "Package Details: {pkg.name}" }
                        button { 
                            class: "close-btn",
                            onclick: move |_| on_close.call(()),
                            "×"
                        }
                    }
                    
                    div { class: "modal-content",
                        div { class: "package-info",
                            div { class: "info-item",
                                strong { "Version: " }
                                span { "{pkg.version}" }
                            }
                            div { class: "info-item",
                                strong { "Type: " }
                                span { if pkg.is_direct { "Direct dependency" } else { "Transitive dependency" } }
                            }
                            if let Some(size) = pkg.size {
                                div { class: "info-item",
                                    strong { "Size: " }
                                    span { "{size} KB" }
                                }
                            }
                        }
                        
                        if !pkg.dependencies.is_empty() {
                            div { class: "dependencies-section",
                                h4 { "Dependencies ({pkg.dependencies.len()})" }
                                div { class: "dependency-list",
                                    for dep_name in &pkg.dependencies {
                                        div { class: "dependency-item", "{dep_name}" }
                                    }
                                }
                            }
                        }
                        
                        if !pkg.dependents.is_empty() {
                            div { class: "dependents-section",
                                h4 { "Used by ({pkg.dependents.len()})" }
                                div { class: "dependents-list",
                                    for dependent in &pkg.dependents {
                                        div { class: "dependent-item", "{dependent}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        None => rsx! {
            div { "Package not found" }
        }
    }
}

fn get_mock_dependencies() -> HashMap<String, DependencyNode> {
    let mut deps = HashMap::new();
    
    deps.insert("numpy".to_string(), DependencyNode {
        name: "numpy".to_string(),
        version: "1.24.3".to_string(),
        dependencies: vec![],
        dependents: vec!["pandas".to_string(), "matplotlib".to_string()],
        is_direct: true,
        size: Some(4200),
    });
    
    deps.insert("pandas".to_string(), DependencyNode {
        name: "pandas".to_string(),
        version: "2.0.1".to_string(),
        dependencies: vec!["numpy".to_string(), "python-dateutil".to_string()],
        dependents: vec![],
        is_direct: true,
        size: Some(8500),
    });
    
    deps.insert("matplotlib".to_string(), DependencyNode {
        name: "matplotlib".to_string(),
        version: "3.7.1".to_string(),
        dependencies: vec!["numpy".to_string(), "pillow".to_string()],
        dependents: vec![],
        is_direct: true,
        size: Some(6200),
    });
    
    deps.insert("python-dateutil".to_string(), DependencyNode {
        name: "python-dateutil".to_string(),
        version: "2.8.2".to_string(),
        dependencies: vec![],
        dependents: vec!["pandas".to_string()],
        is_direct: false,
        size: Some(240),
    });
    
    deps.insert("pillow".to_string(), DependencyNode {
        name: "pillow".to_string(),
        version: "10.0.0".to_string(),
        dependencies: vec![],
        dependents: vec!["matplotlib".to_string()],
        is_direct: false,
        size: Some(1800),
    });
    
    deps
}
