use crate::components::virtenv::LanguageSelector;
use crate::state::Command;
use crate::state::AppState;
use dioxus::prelude::*;
use dioxus_signals::Signal;

#[derive(Clone, PartialEq, Debug)]
pub struct CreateEnvironmentForm {
    pub name: String,
    pub language: Option<String>,
    pub version: Option<String>,
    pub template: Option<String>,
    pub location: Option<String>,
    pub packages: Vec<String>,
}

#[derive(Clone, PartialEq)]
enum PackageValidationState {
    Idle,
    Checking,
    Invalid(String),
}

#[component]
pub fn ProjectWizard(
    on_close: EventHandler<()>,
    on_create: EventHandler<CreateEnvironmentForm>,
) -> Element {
    let bridge = use_context::<crate::app::BackendBridge>();
    let form = use_signal(|| CreateEnvironmentForm {
        name: String::new(),
        language: None,
        version: None,
        template: None,
        location: None,
        packages: Vec::new(),
    });
    let mut current_step = use_signal(|| 0);
    let package_validation = use_signal(|| PackageValidationState::Idle);
    let _app_state = use_context::<Signal<AppState>>();

    use_effect(move || {
        bridge.send(Command::VirtEnvGetTemplates);
    });

    let step_titles = vec![
        "Basic Info",
        "Language & Version",
        "Template",
        "Packages",
        "Review",
    ];

    let can_proceed = match current_step() {
        0 => !form.read().name.is_empty(),
        1 => form.read().language.is_some(),
        2 => true, // Template is optional
        3 => matches!(package_validation(), PackageValidationState::Idle),
        4 => true,
        _ => false,
    };

    rsx! {
        div { class: "modal-overlay",
            div { class: "wizard-modal",
                div { class: "wizard-header",
                    h2 { "Create Virtual Environment" }
                    button {
                        class: "close-btn",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                div { class: "wizard-progress",
                    for (i, title) in step_titles.iter().enumerate() {
                        div {
                            class: format!("progress-step {} {}",
                                if i <= current_step() { "active" } else { "" },
                                if i < current_step() { "completed" } else { "" }
                            ),
                            div { class: "step-number", "{i + 1}" }
                            div { class: "step-title", "{title}" }
                        }
                    }
                }

                div { class: "wizard-content",
                    match current_step() {
                        0 => rsx! { BasicInfoStep { form: form.clone() } },
                        1 => rsx! { LanguageStep { form: form.clone() } },
                        2 => rsx! { TemplateStep { form: form.clone() } },
                        3 => rsx! { PackagesStep { form: form.clone(), validation: package_validation } },
                        4 => rsx! { ReviewStep { form: form.read().clone() } },
                        _ => rsx! { div { "Unknown step" } }
                    }
                }

                div { class: "wizard-actions",
                    if current_step() > 0 {
                        button {
                            class: "btn btn-secondary",
                            onclick: move |_| {
                                current_step.set(current_step() - 1);
                            },
                            "Back"
                        }
                    } else {
                        div {}
                    }
                    if current_step() < 4 {
                        button {
                            class: "btn btn-primary",
                            disabled: !can_proceed,
                            onclick: move |_| current_step.set(current_step() + 1),
                            "Next"
                        }
                    } else {
                        button {
                            class: "btn btn-primary",
                            onclick: move |_| {
                                on_create.call(form.read().clone());
                            },
                            "Create Environment"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn BasicInfoStep(form: Signal<CreateEnvironmentForm>) -> Element {
    rsx! {
        div { class: "wizard-step",
            h3 { "Environment Details" }
            div { class: "form-group",
                label { "Name" }
                input {
                    r#type: "text",
                    class: "form-input",
                    placeholder: "my-project",
                    value: "{form.read().name}",
                    oninput: move |evt| {
                        form.write().name = evt.value();
                    }
                }
                div { class: "form-help",
                    "Choose a unique name for your virtual environment"
                }
            }
            div { class: "form-group",
                label { "Location (optional)" }
                div { class: "file-picker",
                    input {
                        r#type: "text",
                        class: "form-input file-picker-input",
                        placeholder: "Leave empty for default location",
                        value: "{form.read().location.as_deref().unwrap_or(\"\")}",
                        oninput: move |evt| {
                            let value = evt.value();
                            form.write().location = if value.is_empty() { None } else { Some(value) };
                        }
                    }
                    button {
                        class: "btn btn-outline file-picker-button",
                        onclick: move |_| {
                            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                form.write().location = Some(folder.to_string_lossy().to_string());
                            }
                        },
                        "Browse"
                    }
                }
                div { class: "form-help", "Pick a folder or leave empty for the default location." }
            }
        }
    }
}

#[component]
fn LanguageStep(form: Signal<CreateEnvironmentForm>) -> Element {
    rsx! {
        div { class: "wizard-step",
            LanguageSelector {
                selected_language: form.read().language.clone(),
                on_language_select: move |lang: String| {
                    let default_version = match lang.as_str() {
                        "python" => Some("3.13".to_string()),
                        "node" => Some("22".to_string()),
                        "rust" => Some("stable".to_string()),
                        "java" => Some("21".to_string()),
                        _ => None,
                    };

                    let mut state = form.write();
                    state.language = Some(lang);
                    state.version = default_version;
                }
            }
        }
    }
}

#[component]
fn TemplateStep(form: Signal<CreateEnvironmentForm>) -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let selected_language = form.read().language.clone();
    let templates: Vec<_> = app_state
        .read()
        .virtenv
        .templates
        .iter()
        .filter(|template| {
            selected_language
                .as_deref()
                .map(|lang| template.language.eq_ignore_ascii_case(lang))
                .unwrap_or(true)
        })
        .cloned()
        .collect();

    rsx! {
        div { class: "wizard-step",
            h3 { "Choose Template (Optional)" }
            div { class: "template-options",
                div {
                    class: format!("template-card {}",
                        if form.read().template.is_none() { "selected" } else { "" }
                    ),
                    onclick: move |_| form.write().template = None,
                    div { class: "template-name", "Blank Environment" }
                    div { class: "template-description", "Start with a clean environment" }
                }
                for template in templates {
                    div {
                        class: format!("template-card {}",
                            if form.read().template.as_ref() == Some(&template.id) { "selected" } else { "" }
                        ),
                        onclick: {
                            let template_id = template.id.clone();
                            move |_| form.write().template = Some(template_id.clone())
                        },
                        div { class: "template-name", "{template.name}" }
                        div { class: "template-description", "{template.description}" }
                        div { class: "template-packages", "{template.package_count} packages" }
                    }
                }
            }
        }
    }
}

#[component]
fn PackagesStep(
    form: Signal<CreateEnvironmentForm>,
    validation: Signal<PackageValidationState>,
) -> Element {
    let app_state = use_context::<Signal<AppState>>();

    let is_java = form
        .read()
        .language
        .as_deref()
        .map(|l| l.eq_ignore_ascii_case("java"))
        .unwrap_or(false);

    let selected_template_packages: Vec<String> = form
        .read()
        .template
        .as_ref()
        .and_then(|template_id| {
            app_state
                .read()
                .virtenv
                .templates
                .iter()
                .find(|template| template.id == *template_id)
                .map(|template| template.packages.clone())
        })
        .unwrap_or_default();

    if is_java && !form.read().packages.is_empty() {
        form.write().packages.clear();
    }

    let mut package_input = use_signal(|| String::new());
    let mut suggestions = use_signal(|| Vec::<String>::new());
    let mut last_language = use_signal(|| String::new());

    let language = form.read().language.clone().unwrap_or_default();
    if !language.is_empty() && last_language() != language {
        last_language.set(language.clone());
        let list = popular_packages(&language);
        suggestions.set(list);
    }

    let validation_message = match validation() {
        PackageValidationState::Invalid(msg) => Some(msg),
        PackageValidationState::Checking => Some("Checking package...".to_string()),
        _ => None,
    };

    let add_package = std::rc::Rc::new(std::cell::RefCell::new(move |name: String| {
        let raw = name.trim();
        if raw.is_empty() {
            return;
        }

        validation.set(PackageValidationState::Checking);

        let mut validation = validation.clone();
        let mut form = form.clone();
        let mut package_input = package_input.clone();
        let language = language.clone();

        let candidates: Vec<String> = raw
            .split(|c: char| c == ',' || c.is_whitespace())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let invalid: Vec<String> = candidates
            .iter()
            .filter(|pkg| !validate_package_exists(&language, pkg))
            .cloned()
            .collect();

        if !invalid.is_empty() {
            validation.set(PackageValidationState::Invalid(format!(
                "Not found in popular list: {}",
                invalid.join(", ")
            )));
            return;
        }

        let mut state = form.write();
        for pkg in candidates {
            if !state.packages.iter().any(|p| p.eq_ignore_ascii_case(&pkg)) {
                state.packages.push(pkg);
            }
        }

        validation.set(PackageValidationState::Idle);
        package_input.set(String::new());
    }));

    let add_package_for_keydown = add_package.clone();
    let add_package_for_button = add_package.clone();

    rsx! {
        div { class: "wizard-step",
            h3 { "Additional Packages (Optional)" }
            if form.read().template.is_some() {
                div { class: "package-list",
                    h4 { "Packages from selected template:" }
                    if selected_template_packages.is_empty() {
                        div { class: "muted", "This template does not include predefined packages." }
                    } else {
                        for package in selected_template_packages.iter() {
                            div { class: "package-item",
                                span { "{package}" }
                            }
                        }
                    }
                }
            }
            if is_java {
                div { class: "muted",
                    "Adding packages is disabled for Java environments. You can manage dependencies directly in pom.xml/build.gradle."
                }
            } else {
                div { class: "package-input",
                    input {
                        r#type: "text",
                        class: "form-input",
                        list: "package-suggestions",
                        placeholder: "Package name (e.g., numpy, express, tokio)",
                        value: "{package_input()}",
                        oninput: move |evt| {
                            package_input.set(evt.value());
                            if !matches!(validation(), PackageValidationState::Idle) {
                                validation.set(PackageValidationState::Idle);
                            }
                        },
                        onkeydown: move |evt| {
                            if evt.key() == Key::Enter {
                                add_package_for_keydown.borrow_mut()(package_input());
                            }
                        }
                    }
                    datalist { id: "package-suggestions",
                        for pkg in suggestions.read().iter() {
                            option { value: "{pkg}" }
                        }
                    }
                    button {
                        class: "btn btn-secondary",
                        disabled: package_input().trim().is_empty() || matches!(validation(), PackageValidationState::Checking),
                        onclick: move |_| {
                            add_package_for_button.borrow_mut()(package_input());
                        },
                        if matches!(validation(), PackageValidationState::Checking) { "Checking..." } else { "Add" }
                    }
                }
                if let Some(msg) = validation_message {
                    div { class: "validation-message", "{msg}" }
                }
                div { class: "package-suggestions",
                    if suggestions.read().is_empty() {
                        div { class: "muted", "Popular packages unavailable for this language." }
                    } else {
                        div { class: "package-suggestions-title", "Popular packages" }
                        div { class: "package-chip-grid",
                            for pkg in suggestions.read().iter() {
                                button {
                                    class: "package-chip",
                                    onclick: {
                                        let pkg = pkg.clone();
                                        let add_package = add_package.clone();
                                        move |_| add_package.borrow_mut()(pkg.clone())
                                    },
                                    "{pkg}"
                                }
                            }
                        }
                    }
                }
                if !form.read().packages.is_empty() {
                    div { class: "package-list",
                        h4 { "Packages to install:" }
                        for (i, package) in form.read().packages.iter().enumerate() {
                            div { class: "package-item",
                                span { "{package}" }
                                button {
                                    class: "remove-btn",
                                    onclick: move |_| {
                                        form.write().packages.remove(i);
                                    },
                                    "×"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ReviewStep(form: CreateEnvironmentForm) -> Element {
    rsx! {
        div { class: "wizard-step",
            h3 { "Review Configuration" }
            div { class: "review-section",
                div { class: "review-item",
                    strong { "Name:" }
                    span { "{form.name}" }
                }
                if let Some(language) = &form.language {
                    div { class: "review-item",
                        strong { "Language:" }
                        span { "{language}" }
                        if let Some(version) = &form.version {
                            span { " ({version})" }
                        }
                    }
                }
                if let Some(location) = &form.location {
                    div { class: "review-item",
                        strong { "Location:" }
                        span { "{location}" }
                    }
                }
                if let Some(template) = &form.template {
                    div { class: "review-item",
                        strong { "Template: " }
                        span { "{template}" }
                    }
                }
                if !form.packages.is_empty() {
                    div { class: "review-item",
                        strong { "Additional Packages: " }
                        div { class: "package-tags",
                            for package in &form.packages {
                                span { class: "package-tag", "{package}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn popular_packages(language: &str) -> Vec<String> {
    match language {
        "python" => vec![
            "numpy",
            "pandas",
            "requests",
            "scipy",
            "matplotlib",
            "scikit-learn",
            "fastapi",
            "flask",
            "django",
            "pydantic",
            "pytest",
            "sqlalchemy",
        ],
        "node" => vec![
            "express",
            "react",
            "react-dom",
            "next",
            "lodash",
            "axios",
            "typescript",
            "jest",
            "vite",
            "eslint",
            "prettier",
            "pnpm",
        ],
        "rust" => vec![
            "tokio",
            "serde",
            "serde_json",
            "reqwest",
            "clap",
            "anyhow",
            "thiserror",
            "tracing",
            "uuid",
            "chrono",
            "sqlx",
            "axum",
        ],
        "go" => vec![
            "github.com/gin-gonic/gin",
            "github.com/gorilla/mux",
            "github.com/spf13/viper",
            "github.com/sirupsen/logrus",
            "github.com/stretchr/testify",
            "gorm.io/gorm",
            "github.com/go-chi/chi",
            "github.com/rs/zerolog",
            "github.com/google/uuid",
            "github.com/jmoiron/sqlx",
            "github.com/pkg/errors",
            "go.uber.org/zap",
        ],
        "dotnet" => vec![
            "Newtonsoft.Json",
            "Dapper",
            "Serilog",
            "AutoMapper",
            "Polly",
            "FluentValidation",
            "Microsoft.EntityFrameworkCore",
            "xunit",
            "NUnit",
            "MediatR",
            "Refit",
            "Quartz",
        ],
        "java" => vec![
            "org.springframework:spring-core",
            "org.springframework.boot:spring-boot-starter",
            "com.google.guava:guava",
            "com.fasterxml.jackson.core:jackson-databind",
            "org.projectlombok:lombok",
            "org.slf4j:slf4j-api",
            "ch.qos.logback:logback-classic",
            "org.hibernate:hibernate-core",
            "junit:junit",
            "org.mockito:mockito-core",
            "org.apache.commons:commons-lang3",
            "org.apache.httpcomponents:httpclient",
        ],
        _ => vec![
            "requests", "express", "tokio", "serde", "numpy", "pandas", "lodash", "axios",
            "fastapi", "flask", "clap", "tracing",
        ],
    }
    .iter()
    .map(|s| s.to_string())
    .collect()
}

fn validate_package_exists(language: &str, name: &str) -> bool {
    let normalized = name.trim();
    if normalized.is_empty() {
        return false;
    }
    popular_packages(language)
        .iter()
        .any(|pkg| pkg.eq_ignore_ascii_case(normalized))
}
