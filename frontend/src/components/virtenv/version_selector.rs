use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct VersionInfo {
    pub version: String,
    pub is_installed: bool,
    pub is_recommended: bool,
    pub release_date: Option<String>,
    pub status: VersionStatus,
}

#[derive(Clone, PartialEq)]
pub enum VersionStatus {
    Current,
    LTS,
    Beta,
    Deprecated,
    EOL,
}

#[component]
pub fn VersionSelector(
    language: String,
    selected_version: Option<String>,
    on_version_select: EventHandler<String>,
) -> Element {
    let versions = get_versions_for_language(&language);

    rsx! {
        div { class: "version-selector",
            h4 { "Select {language} Version" }
            div { class: "version-grid",
                for version_info in versions {
                    VersionCard {
                        version_info: version_info.clone(),
                        selected: selected_version.as_ref().map_or(false, |s| s == &version_info.version),
                        on_select: {
                            let version = version_info.version.clone();
                            move |_| on_version_select.call(version.clone())
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn VersionCard(version_info: VersionInfo, selected: bool, on_select: EventHandler<()>) -> Element {
    let status_class = match version_info.status {
        VersionStatus::Current => "current",
        VersionStatus::LTS => "lts",
        VersionStatus::Beta => "beta",
        VersionStatus::Deprecated => "deprecated",
        VersionStatus::EOL => "eol",
    };

    let status_label = match version_info.status {
        VersionStatus::Current => "Current",
        VersionStatus::LTS => "LTS",
        VersionStatus::Beta => "Beta",
        VersionStatus::Deprecated => "Deprecated",
        VersionStatus::EOL => "End of Life",
    };

    rsx! {
        div {
            class: format!("version-card {} {}",
                status_class,
                if selected { "selected" } else { "" }
            ),
            onclick: move |_| on_select.call(()),
            div { class: "version-header",
                span { class: "version-number", "{version_info.version}" }
                if version_info.is_recommended {
                    span { class: "recommended-badge", "Recommended" }
                }
                span { class: format!("status-badge {}", status_class), "{status_label}" }
            }
            div { class: "version-info",
                div { class: "install-status",
                    if version_info.is_installed {
                        span { class: "installed", "✓ Installed" }
                    } else {
                        span { class: "not-installed", "Not installed" }
                    }
                }
                if let Some(date) = &version_info.release_date {
                    div { class: "release-date", "Released: {date}" }
                }
            }
            if selected {
                div { class: "selected-indicator", "✓" }
            }
        }
    }
}

fn get_versions_for_language(language: &str) -> Vec<VersionInfo> {
    match language {
        "python" => vec![
            VersionInfo {
                version: "3.12".to_string(),
                is_installed: true,
                is_recommended: true,
                release_date: Some("Oct 2023".to_string()),
                status: VersionStatus::Current,
            },
            VersionInfo {
                version: "3.11".to_string(),
                is_installed: true,
                is_recommended: false,
                release_date: Some("Oct 2021".to_string()),
                status: VersionStatus::LTS,
            },
            VersionInfo {
                version: "3.10".to_string(),
                is_installed: false,
                is_recommended: false,
                release_date: Some("Oct 2020".to_string()),
                status: VersionStatus::LTS,
            },
            VersionInfo {
                version: "3.9".to_string(),
                is_installed: false,
                is_recommended: false,
                release_date: Some("Oct 2020".to_string()),
                status: VersionStatus::Deprecated,
            },
        ],
        "node" => vec![
            VersionInfo {
                version: "20".to_string(),
                is_installed: true,
                is_recommended: true,
                release_date: Some("Apr 2023".to_string()),
                status: VersionStatus::LTS,
            },
            VersionInfo {
                version: "18".to_string(),
                is_installed: false,
                is_recommended: false,
                release_date: Some("Apr 2022".to_string()),
                status: VersionStatus::LTS,
            },
            VersionInfo {
                version: "16".to_string(),
                is_installed: false,
                is_recommended: false,
                release_date: Some("Apr 2021".to_string()),
                status: VersionStatus::EOL,
            },
        ],
        _ => vec![],
    }
}
