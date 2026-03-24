use crate::core::event_bus::EventBus;
use crate::models::events::Event;
use crate::models::commands::Command;
use crate::models::virtenv::*;
use crate::utils::command_executor;
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;


pub struct VirtualEnvService {
    bus: EventBus,
    cmd_rx: mpsc::Receiver<Command>,
    poll_interval: std::time::Duration,
    environments: HashMap<String, VirtualEnvironment>,
    templates: Vec<EnvironmentTemplate>,
    active_env: Option<String>,
}

impl VirtualEnvService {
    pub async fn new(bus: EventBus, cmd_rx: mpsc::Receiver<Command>) -> Result<Self> {
        info!("[VIRTENV_SERVICE] Initializing VirtualEnvService...");
        
        let templates = Self::load_default_templates();
        info!("[VIRTENV_SERVICE] Loaded {} templates", templates.len());
        
        // Load existing environments from JSON file
        info!("[VIRTENV_SERVICE] About to load environments from file...");
        let environments = Self::load_environments_from_file().await.unwrap_or_else(|e| {
            error!("[VIRTENV_SERVICE] Failed to load environments from file: {}", e);
            HashMap::new()
        });
        
        info!("[VIRTENV_SERVICE] Successfully loaded {} environments from file", environments.len());
        info!("[VIRTENV_SERVICE] Environment IDs at startup: {:?}", environments.keys().collect::<Vec<_>>());
        
        Ok(Self {
            bus,
            cmd_rx,
            poll_interval: std::time::Duration::from_secs(10),
            environments,
            templates,
            active_env: None,
        })
    }
    
    async fn load_environments_from_file() -> Result<HashMap<String, VirtualEnvironment>> {
        // Use the same path structure as frontend: ~/.config/devenv/manager/environments.json
        info!("[LOAD_ENVS] Getting config directory...");
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not determine config directory"))?
            .join("manager");
        let file_path = config_dir.join("environments.json");
        
        info!("[LOAD_ENVS] Loading environments from: {:?}", file_path);
        
        if !file_path.exists() {
            info!("[LOAD_ENVS] No environments.json file found at {:?}", file_path);
            return Ok(HashMap::new());
        }
        
        let json = fs::read_to_string(&file_path).await?;
        info!("[LOAD_ENVS] Raw JSON content:\n{}", json);
        if json.trim().is_empty() {
            info!("[LOAD_ENVS] environments.json is empty; starting with no environments");
            return Ok(HashMap::new());
        }
        
        // Parse directly as VirtualEnvironment (backend format)
        let envs: Vec<VirtualEnvironment> = serde_json::from_str(&json)
            .context("Failed to parse environments from JSON")?;
        
        info!("[LOAD_ENVS] Successfully parsed {} environments from JSON", envs.len());
        
        let mut map = HashMap::new();
        for env in envs {
            info!("[LOAD_ENVS] Registering environment: id={}, name={}, path={}", env.id, env.name, env.path.display());
            map.insert(env.id.clone(), env);
        }
        
        info!("Total environments in map: {}", map.len());
        info!("Environment IDs: {:?}", map.keys().collect::<Vec<_>>());
        
        Ok(map)
    }
    
    async fn save_environments_to_file(&self) -> Result<()> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not determine config directory"))?
            .join("manager");
        let file_path = config_dir.join("environments.json");
        
        info!("Saving {} environments to: {:?}", self.environments.len(), file_path);
        
        // Ensure directory exists
        fs::create_dir_all(&config_dir).await?;
        
        // Convert HashMap to Vec for serialization
        let envs: Vec<&VirtualEnvironment> = self.environments.values().collect();
        let json = serde_json::to_string_pretty(&envs)?;
        
        fs::write(&file_path, json).await?;
        info!("Successfully saved environments to file");
        
        Ok(())
    }

    pub async fn create_environment(&mut self, request: CreateEnvironmentRequest) -> Result<VirtualEnvironment> {
        info!(name = %request.name, language = ?request.language, "Creating virtual environment");
        
        let env_id = Uuid::new_v4().to_string();
        let env_path = self.determine_env_path(&request)?;
        
        info!("[CREATE_ENV] Generated env_id: {}", env_id);
        info!("[CREATE_ENV] Determined env_path: {}", env_path.display());
        
        let environment = match request.language {
            Language::Python => self.create_python_env(&request, &env_id, &env_path).await?,
            Language::Node => self.create_node_env(&request, &env_id, &env_path).await?,
            Language::Rust => self.create_rust_env(&request, &env_id, &env_path).await?,
            Language::Java => self.create_java_env(&request, &env_id, &env_path).await?,
            Language::Ruby => self.create_ruby_env(&request, &env_id, &env_path).await?,
            Language::Php => self.create_php_env(&request, &env_id, &env_path).await?,
            Language::Other(ref lang) => return Err(anyhow!("Language not supported: {}", lang)),
        };
        
        info!("[CREATE_ENV] Inserting environment into map: id={}, name={}, path={}", 
            env_id, environment.name, environment.path.display());
        
        self.environments.insert(env_id.clone(), environment.clone());
        
        // Refresh packages for this new env to ensure we have the installed packages
        if let Err(e) = self.refresh_environment_packages(&env_id).await {
             warn!("Failed to refresh packages for new env {}: {}", env_id, e);
        }

        // Get updated env
        let updated_env = self.environments.get(&env_id).unwrap().clone();
        
        info!("[CREATE_ENV] Total environments after insert: {}", self.environments.len());
        info!("[CREATE_ENV] Environment IDs: {:?}", self.environments.keys().collect::<Vec<_>>());
        
        // Save environments to file after creating
        if let Err(e) = self.save_environments_to_file().await {
            error!("[CREATE_ENV] Failed to save environments to file: {}", e);
        }
        
        self.bus.publish(Event::VirtualEnvCreated { environment: updated_env.clone() });
        
        Ok(updated_env)
    }

    pub async fn delete_environment(&mut self, env_id: &str) -> Result<()> {
        let environment = self.environments.get(env_id)
            .ok_or_else(|| anyhow!("Environment not found: {}", env_id))?;
        
        info!(env_id = %env_id, name = %environment.name, path = %environment.path.display(), "Deleting virtual environment");
        
        // Delete the actual virtual environment directory
        if environment.path.exists() {
            info!("Removing environment directory: {}", environment.path.display());
            fs::remove_dir_all(&environment.path).await
                .context("Failed to remove environment directory")?;
            info!("Successfully deleted environment directory");
        } else {
            warn!("Environment directory does not exist: {}", environment.path.display());
        }
        
        self.environments.remove(env_id);
        
        // Save updated environments to file
        if let Err(e) = self.save_environments_to_file().await {
            error!("Failed to save environments after deletion: {}", e);
        }
        
        self.bus.publish(Event::VirtualEnvDeleted { env_id: env_id.to_string() });
        
        Ok(())
    }

    pub async fn activate_environment(&mut self, env_id: &str) -> Result<()> {
        if !self.environments.contains_key(env_id) {
            return Err(anyhow!("Environment not found: {}", env_id));
        }
        
        if let Some(current_id) = &self.active_env {
            if let Some(current_env) = self.environments.get_mut(current_id) {
                current_env.is_active = false;
            }
        }
        
        if let Some(env) = self.environments.get_mut(env_id) {
            env.is_active = true;
            env.last_used = Some(chrono::Utc::now());
            self.active_env = Some(env_id.to_string());
            
            info!(env_id = %env_id, name = %env.name, "Activated virtual environment");
            self.bus.publish(Event::VirtualEnvActivated { env_id: env_id.to_string() });
        }
        
        Ok(())
    }

    pub async fn install_packages(&mut self, operation: PackageOperation) -> Result<()> {
        info!("[INSTALL_PACKAGES] Attempting to find environment with ID: {}", operation.env_id);
        info!("[INSTALL_PACKAGES] Currently registered environment IDs: {:?}", self.environments.keys().collect::<Vec<_>>());
        info!("[INSTALL_PACKAGES] Total environments in memory: {}", self.environments.len());
        
        let env = self.environments.get(&operation.env_id)
            .ok_or_else(|| {
                error!("[INSTALL_PACKAGES] Environment not found! Requested ID: {}", operation.env_id);
                error!("[INSTALL_PACKAGES] Available IDs: {:?}", self.environments.keys().collect::<Vec<_>>());
                anyhow!("Environment not found: {}", operation.env_id)
            })?;
        
        info!(
            "[INSTALL_PACKAGES] Found environment: id={}, name={}, path={}",
            operation.env_id,
            env.name,
            env.path.display()
        );
        info!(
            env_id = %operation.env_id, 
            env_path = %env.path.display(),
            packages = ?operation.packages, 
            "Installing packages - using path from loaded environment"
        );
        
        self.bus.publish(Event::PackageOperationStarted { operation: operation.clone() });
        
        let result = match env.language {
            Language::Python => self.install_python_packages(env, &operation).await,
            Language::Node => self.install_node_packages(env, &operation).await,
            Language::Rust => self.install_rust_packages(env, &operation).await,
            Language::Java => self.install_java_packages(env, &operation).await,
            Language::Ruby => self.install_ruby_packages(env, &operation).await,
            Language::Php => self.install_php_packages(env, &operation).await,
            Language::Other(ref lang) => Err(anyhow!("Package installation not supported for {}", lang)),
        };
        
        match result {
            Ok(_) => {
                self.refresh_environment_packages(&operation.env_id).await?;
                self.bus.publish(Event::PackageOperationCompleted { 
                    env_id: operation.env_id.clone(), 
                    success: true, 
                    message: Some("Packages installed successfully".to_string()) 
                });
            }
            Err(e) => {
                error!(error = %e, "Failed to install packages");
                self.bus.publish(Event::PackageOperationCompleted { 
                    env_id: operation.env_id.clone(), 
                    success: false, 
                    message: Some(e.to_string()) 
                });
            }
        }
        
        Ok(())
    }

    pub async fn get_environments(&mut self) -> Vec<VirtualEnvironment> {
        self.update_env_sizes().await;
        self.environments.values().cloned().collect()
    }

    pub fn get_templates(&self) -> &[EnvironmentTemplate] {
        &self.templates
    }

    fn determine_env_path(&self, request: &CreateEnvironmentRequest) -> Result<PathBuf> {
        if let Some(location) = &request.location {
            Ok(location.join(&request.name))
        } else {
            match request.language {
                Language::Python => {
                    let home = env::var("HOME").context("HOME environment variable not set")?;
                    Ok(PathBuf::from(home).join(".virtualenvs").join(&request.name))
                }
                Language::Node => {
                    let home = env::var("HOME").context("HOME environment variable not set")?;
                    Ok(PathBuf::from(home).join(".nvm").join("environments").join(&request.name))
                }
                _ => {
                    let home = env::var("HOME").context("HOME environment variable not set")?;
                    Ok(PathBuf::from(home).join(".devenv").join(request.language.as_str()).join(&request.name))
                }
            }
        }
    }

    async fn create_python_env(&self, request: &CreateEnvironmentRequest, env_id: &str, env_path: &Path) -> Result<VirtualEnvironment> {
        fs::create_dir_all(env_path).await?;
        
        let python_cmd = if let Some(version) = &request.version {
            format!("python{}", version)
        } else {
            "python3".to_string()
        };
        
        command_executor::run(&python_cmd, &["-m", "venv", env_path.to_str().unwrap()]).await?;
        
        if let Some(template_id) = &request.template {
            self.apply_template(env_path, template_id, &Language::Python).await?;
        }
        
        if !request.packages.is_empty() {
            self.install_python_packages_direct(env_path, &request.packages).await?;
        }
        
        Ok(VirtualEnvironment {
            id: env_id.to_string(),
            name: request.name.clone(),
            path: env_path.to_path_buf(),
            language: Language::Python,
            version: request.version.clone().unwrap_or_else(|| "3.11".to_string()),
            is_active: false,
            packages: Vec::new(),
            created_at: chrono::Utc::now(),
            last_used: None,
            template: request.template.clone(),
            project_path: request.project_path.clone(),
            health: EnvironmentHealth {
                status: HealthStatus::Healthy,
                issues: Vec::new(),
                last_check: chrono::Utc::now(),
                cpu_usage: None,
                memory_usage: None,
            },
            size_mb: None,
        })
    }

    async fn create_node_env(&self, request: &CreateEnvironmentRequest, env_id: &str, env_path: &Path) -> Result<VirtualEnvironment> {
        fs::create_dir_all(env_path).await?;
        
        // Initialize npm project
        let init_args = vec!["init", "-y"];
        command_executor::run_in_dir("npm", &init_args, env_path).await
            .context("Failed to initialize npm project")?;
        
        // Install packages if specified
        if !request.packages.is_empty() {
            let mut install_args = vec!["install"];
            let package_refs: Vec<&str> = request.packages.iter().map(|s| s.as_str()).collect();
            install_args.extend(package_refs);
            command_executor::run_in_dir("npm", &install_args, env_path).await
                .context("Failed to install npm packages")?;
        }
        
        Ok(VirtualEnvironment {
            id: env_id.to_string(),
            name: request.name.clone(),
            path: env_path.to_path_buf(),
            language: Language::Node,
            version: request.version.clone().unwrap_or_else(|| "latest".to_string()),
            is_active: false,
            packages: Vec::new(),
            created_at: chrono::Utc::now(),
            last_used: None,
            template: request.template.clone(),
            project_path: request.project_path.clone().map(PathBuf::from),
            health: EnvironmentHealth::default(),
            size_mb: None,
        })
    }
    
    async fn create_rust_env(&self, request: &CreateEnvironmentRequest, env_id: &str, env_path: &Path) -> Result<VirtualEnvironment> {
        fs::create_dir_all(env_path).await?;
        
        info!("[RUST_ENV] Initializing Cargo project in directory: {:?}", env_path);
        
        // Initialize cargo project in the existing directory
        info!("[RUST_ENV] Running: cargo init --vcs none");
        let init_args = vec!["init", "--vcs", "none"];
        
        match command_executor::run_in_dir("cargo", &init_args, env_path).await {
            Ok(output) => {
                info!("[RUST_ENV] Cargo init successful: {}", output);
            }
            Err(e) => {
                error!("[RUST_ENV] Cargo init failed: {:?}", e);
                return Err(e).context("Failed to initialize Cargo project");
            }
        }
        
        // Add dependencies if specified
        if !request.packages.is_empty() {
            info!("[RUST_ENV] Adding {} dependencies", request.packages.len());
            for package in &request.packages {
                info!("[RUST_ENV] Adding dependency: {}", package);
                let add_args = vec!["add", package];
                match command_executor::run_in_dir("cargo", &add_args, env_path).await {
                    Ok(output) => {
                        info!("[RUST_ENV] Successfully added {}: {}", package, output);
                    }
                    Err(e) => {
                        error!("[RUST_ENV] Failed to add {}: {:?}", package, e);
                        return Err(e).context(format!("Failed to add dependency: {}", package));
                    }
                }
            }
        }
        
        Ok(VirtualEnvironment {
            id: env_id.to_string(),
            name: request.name.clone(),
            path: env_path.to_path_buf(),
            language: Language::Rust,
            version: request.version.clone().unwrap_or_else(|| "latest".to_string()),
            is_active: false,
            packages: Vec::new(),
            created_at: chrono::Utc::now(),
            last_used: None,
            template: request.template.clone(),
            project_path: request.project_path.clone().map(PathBuf::from),
            health: EnvironmentHealth::default(),
            size_mb: None,
        })
    }
    
    async fn create_java_env(&self, request: &CreateEnvironmentRequest, env_id: &str, env_path: &Path) -> Result<VirtualEnvironment> {
        fs::create_dir_all(env_path).await?;
        
        // Create basic Maven project structure
        let src_main_java = env_path.join("src/main/java");
        fs::create_dir_all(&src_main_java).await?;
        
        // Create a basic pom.xml
        let pom_content = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0
         http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>{}</artifactId>
    <version>1.0-SNAPSHOT</version>
    <properties>
        <maven.compiler.source>17</maven.compiler.source>
        <maven.compiler.target>17</maven.compiler.target>
    </properties>
</project>"#, request.name);
        
        fs::write(env_path.join("pom.xml"), pom_content).await?;
        
        Ok(VirtualEnvironment {
            id: env_id.to_string(),
            name: request.name.clone(),
            path: env_path.to_path_buf(),
            language: Language::Java,
            version: request.version.clone().unwrap_or_else(|| "17".to_string()),
            is_active: false,
            packages: Vec::new(),
            created_at: chrono::Utc::now(),
            last_used: None,
            template: request.template.clone(),
            project_path: request.project_path.clone().map(PathBuf::from),
            health: EnvironmentHealth::default(),
            size_mb: None,
        })
    }
    
    async fn create_ruby_env(&self, request: &CreateEnvironmentRequest, env_id: &str, env_path: &Path) -> Result<VirtualEnvironment> {
        fs::create_dir_all(env_path).await?;
        
        // Create a Gemfile
        let gemfile_content = "source 'https://rubygems.org'\n\n# Add your gems here\n";
        fs::write(env_path.join("Gemfile"), gemfile_content).await?;
        
        // Install gems if specified
        if !request.packages.is_empty() {
            for package in &request.packages {
                let install_args = vec!["install", package];
                command_executor::run_in_dir("gem", &install_args, env_path).await
                    .context(format!("Failed to install gem: {}", package))?;
            }
        }
        
        Ok(VirtualEnvironment {
            id: env_id.to_string(),
            name: request.name.clone(),
            path: env_path.to_path_buf(),
            language: Language::Ruby,
            version: request.version.clone().unwrap_or_else(|| "latest".to_string()),
            is_active: false,
            packages: Vec::new(),
            created_at: chrono::Utc::now(),
            last_used: None,
            template: request.template.clone(),
            project_path: request.project_path.clone().map(PathBuf::from),
            health: EnvironmentHealth::default(),
            size_mb: None,
        })
    }
    
    async fn create_php_env(&self, request: &CreateEnvironmentRequest, env_id: &str, env_path: &Path) -> Result<VirtualEnvironment> {
        fs::create_dir_all(env_path).await?;
        
        // Initialize composer project
        let package_name = format!("vendor/{}", request.name);
        let init_args = vec!["init", "--no-interaction", "--name", &package_name];
        command_executor::run_in_dir("composer", &init_args, env_path).await
            .context("Failed to initialize Composer project")?;
        
        // Require packages if specified
        if !request.packages.is_empty() {
            let mut require_args = vec!["require"];
            let package_refs: Vec<&str> = request.packages.iter().map(|s| s.as_str()).collect();
            require_args.extend(package_refs);
            command_executor::run_in_dir("composer", &require_args, env_path).await
                .context("Failed to install Composer packages")?;
        }
        
        Ok(VirtualEnvironment {
            id: env_id.to_string(),
            name: request.name.clone(),
            path: env_path.to_path_buf(),
            language: Language::Php,
            version: request.version.clone().unwrap_or_else(|| "latest".to_string()),
            is_active: false,
            packages: Vec::new(),
            created_at: chrono::Utc::now(),
            last_used: None,
            template: request.template.clone(),
            project_path: request.project_path.clone().map(PathBuf::from),
            health: EnvironmentHealth::default(),
            size_mb: None,
        })
    }

    async fn apply_template(&self, env_path: &Path, template_id: &str, language: &Language) -> Result<()> {
        if let Some(template) = self.templates.iter().find(|t| t.id == template_id && t.language == *language) {
            debug!(template_id, "Applying environment template");
            
            match language {
                Language::Python => {
                    self.install_python_packages_direct(env_path, &template.packages).await?;
                }
                _ => {
                    warn!(language = ?language, "Template application not implemented for language");
                }
            }
        }
        
        Ok(())
    }

    async fn install_python_packages_direct(&self, env_path: &Path, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }
        
        if !env_path.exists() {
            return Err(anyhow!("Virtual environment directory does not exist: {}", env_path.display()));
        }
        
        // Build pip install command
        let mut args = vec!["install"];
        for package in packages {
            args.push(package.as_str());
        }
        
        // Build pip paths using the environment directory
        let bin_pip = env_path.join("bin").join("pip");
        let scripts_pip = env_path.join("Scripts").join("pip.exe");
        
        // Try pip commands with absolute paths
        let pip_commands = [
            bin_pip.to_string_lossy().to_string(),
            scripts_pip.to_string_lossy().to_string(),
            "pip".to_string(),
            "pip3".to_string(),
        ];
        
        for pip_cmd in pip_commands {
            match command_executor::run(&pip_cmd, &args).await {
                Ok(output) => {
                    info!("Successfully installed packages using {}: {}", pip_cmd, output);
                    return Ok(());
                }
                Err(e) => {
                    debug!("Failed to install with {} in {}: {}", pip_cmd, env_path.display(), e);
                    continue;
                }
            }
        }
        
        Err(anyhow!("Failed to install packages in environment: {}", env_path.display()))
    }

    async fn install_python_packages(&self, env: &VirtualEnvironment, operation: &PackageOperation) -> Result<()> {
        // Build pip command arguments based on operation type
        let mut args = match operation.operation {
            PackageOperationType::Install => vec!["install"],
            PackageOperationType::Uninstall => vec!["uninstall", "-y"],
            PackageOperationType::Update | PackageOperationType::Upgrade => vec!["install", "--upgrade"],
        };
        
        // Add package names
        let package_refs: Vec<&str> = operation.packages.iter().map(|s| s.as_str()).collect();
        args.extend(package_refs);
        
        // IMPORTANT: Use env.path from the environment object loaded from JSON
        // This ensures we use the exact path stored in the JSON file
        info!(
            env_path = %env.path.display(),
            "Using environment path from loaded environment (from JSON file)"
        );
        
        // Build pip paths using the environment directory from JSON
        let bin_pip = env.path.join("bin").join("pip");
        let scripts_pip = env.path.join("Scripts").join("pip.exe");
        
        // Try pip commands with absolute paths
        let pip_commands = [
            bin_pip.to_string_lossy().to_string(),
            scripts_pip.to_string_lossy().to_string(),
            "pip".to_string(),
            "pip3".to_string(),
        ];
        
        for pip_cmd in pip_commands {
            match command_executor::run(&pip_cmd, &args).await {
                Ok(output) => {
                    info!("Successfully executed pip operation using {}: {}", pip_cmd, output);
                    return Ok(());
                }
                Err(e) => {
                    debug!("Failed with {} in {}: {}", pip_cmd, env.path.display(), e);
                    continue;
                }
            }
        }
        
        Err(anyhow!("Failed to execute pip operation in environment: {}", env.path.display()))
    }

    async fn install_node_packages(&self, env: &VirtualEnvironment, operation: &PackageOperation) -> Result<()> {
        info!("Installing Node packages in: {}", env.path.display());
        
        // Determine package manager command
        let mut args = match operation.operation {
            PackageOperationType::Install => vec!["install"],
            PackageOperationType::Uninstall => vec!["uninstall"],
            PackageOperationType::Update | PackageOperationType::Upgrade => vec!["update"],
        };
        
        // Add package names
        let package_refs: Vec<&str> = operation.packages.iter().map(|s| s.as_str()).collect();
        args.extend(package_refs);
        
        // Try npm, then yarn, then pnpm
        let package_managers = ["npm", "yarn", "pnpm"];
        
        for pm in package_managers {
            match command_executor::run_in_dir(pm, &args, &env.path).await {
                Ok(output) => {
                    info!("Successfully executed {:?} operation using {}: {}", operation.operation, pm, output);
                    return Ok(());
                }
                Err(e) => {
                    debug!("Failed with {} in {}: {}", pm, env.path.display(), e);
                    continue;
                }
            }
        }
        
        Err(anyhow!("Failed to execute npm/yarn/pnpm operation in environment: {}", env.path.display()))
    }
    
    async fn install_rust_packages(&self, env: &VirtualEnvironment, operation: &PackageOperation) -> Result<()> {
        info!("Installing Rust packages in: {}", env.path.display());
        
        match operation.operation {
            PackageOperationType::Install => {
                // For Rust, we add dependencies to Cargo.toml
                for package in &operation.packages {
                    let args = vec!["add", package];
                    command_executor::run_in_dir("cargo", &args, &env.path).await
                        .context(format!("Failed to add package: {}", package))?;
                }
                Ok(())
            }
            PackageOperationType::Uninstall => {
                for package in &operation.packages {
                    let args = vec!["remove", package];
                    command_executor::run_in_dir("cargo", &args, &env.path).await
                        .context(format!("Failed to remove package: {}", package))?;
                }
                Ok(())
            }
            PackageOperationType::Update | PackageOperationType::Upgrade => {
                let args = vec!["update"];
                command_executor::run_in_dir("cargo", &args, &env.path).await
                    .context("Failed to update packages")?;
                Ok(())
            }
        }
    }
    
    async fn install_java_packages(&self, env: &VirtualEnvironment, _operation: &PackageOperation) -> Result<()> {
        info!("Installing Java packages in: {}", env.path.display());
        
        // Check if using Maven or Gradle
        let is_maven = env.path.join("pom.xml").exists();
        let is_gradle = env.path.join("build.gradle").exists() || env.path.join("build.gradle.kts").exists();
        
        if is_maven {
            // Maven dependencies need to be added to pom.xml manually
            Err(anyhow!("Maven package installation requires manual editing of pom.xml"))
        } else if is_gradle {
            // Gradle dependencies need to be added to build.gradle manually
            Err(anyhow!("Gradle package installation requires manual editing of build.gradle"))
        } else {
            Err(anyhow!("No Maven or Gradle project detected"))
        }
    }
    
    async fn install_ruby_packages(&self, env: &VirtualEnvironment, operation: &PackageOperation) -> Result<()> {
        info!("Installing Ruby packages in: {}", env.path.display());
        
        let mut args = match operation.operation {
            PackageOperationType::Install => vec!["install"],
            PackageOperationType::Uninstall => vec!["uninstall"],
            PackageOperationType::Update | PackageOperationType::Upgrade => vec!["update"],
        };
        
        let package_refs: Vec<&str> = operation.packages.iter().map(|s| s.as_str()).collect();
        args.extend(package_refs);
        
        command_executor::run_in_dir("gem", &args, &env.path).await
            .context("Failed to execute gem operation")?;
        
        Ok(())
    }
    
    async fn install_php_packages(&self, env: &VirtualEnvironment, operation: &PackageOperation) -> Result<()> {
        info!("Installing PHP packages in: {}", env.path.display());
        
        let command = match operation.operation {
            PackageOperationType::Install => "require",
            PackageOperationType::Uninstall => "remove",
            PackageOperationType::Update | PackageOperationType::Upgrade => "update",
        };
        
        let mut args = vec![command];
        let package_refs: Vec<&str> = operation.packages.iter().map(|s| s.as_str()).collect();
        args.extend(package_refs);
        
        command_executor::run_in_dir("composer", &args, &env.path).await
            .context("Failed to execute composer operation")?;
        
        Ok(())
    }

    async fn refresh_environment_packages(&mut self, env_id: &str) -> Result<()> {
        if let Some(env) = self.environments.get(env_id) {
            let env_path = env.path.clone();
            let language = env.language.clone();
            let packages = match language {
                Language::Python => self.list_python_packages(&env_path).await?,
                Language::Node => self.list_node_packages(&env_path).await?,
                Language::Rust => self.list_rust_packages(&env_path).await?,
                Language::Java => self.list_java_packages(&env_path).await?,
                Language::Ruby => self.list_ruby_packages(&env_path).await?,
                Language::Php => self.list_php_packages(&env_path).await?,
                _ => Vec::new(),
            };
            
            if let Some(env) = self.environments.get_mut(env_id) {
                env.packages = packages;
                self.bus.publish(Event::VirtualEnvUpdated { environment: env.clone() });
            }
        }
        
        Ok(())
    }

    async fn refresh_all_packages(&mut self) {
        info!("Refreshing packages for all environments...");
        let env_ids: Vec<String> = self.environments.keys().cloned().collect();
        for id in env_ids {
            if let Err(e) = self.refresh_environment_packages(&id).await {
                // Log error but continue
                tracing::warn!("Failed to refresh packages for {}: {}", id, e);
            }
        }
        info!("Finished refreshing packages for all environments");
    }

    async fn list_python_packages(&self, env_path: &Path) -> Result<Vec<Package>> {
        let pip_path = env_path.join("bin").join("pip");
        
        let output = command_executor::run(pip_path.to_str().unwrap(), &["list", "--format=json"]).await?;
        
        let packages: Vec<serde_json::Value> = serde_json::from_str(&output)?;
        let mut result = Vec::new();
        
        for pkg in packages {
            if let (Some(name), Some(version)) = (pkg["name"].as_str(), pkg["version"].as_str()) {
                result.push(Package {
                    name: name.to_string(),
                    version: version.to_string(),
                    description: None,
                    size: None,
                    dependencies: Vec::new(),
                    is_dev_dependency: false,
                });
            }
        }
        
        Ok(result)
    }

    async fn list_node_packages(&self, env_path: &Path) -> Result<Vec<Package>> {
        let output = command_executor::run_in_dir("npm", &["list", "--json", "--depth=0"], env_path).await.unwrap_or_else(|_| "{}".to_string());
        let json: serde_json::Value = serde_json::from_str(&output).unwrap_or(serde_json::json!({}));
        let mut result = Vec::new();
        if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
             for (name, info) in deps {
                 let version = info.get("version").and_then(|v| v.as_str()).unwrap_or("unknown");
                 result.push(Package {
                    name: name.clone(),
                    version: version.to_string(),
                    description: None,
                    size: None,
                    dependencies: Vec::new(),
                    is_dev_dependency: false,
                 });
             }
        }
        Ok(result)
    }

    async fn list_rust_packages(&self, env_path: &Path) -> Result<Vec<Package>> {
        info!("[RUST_PACKAGES] Listing packages for: {:?}", env_path);
        
        // Check if Cargo.toml exists
        let cargo_toml = env_path.join("Cargo.toml");
        if !cargo_toml.exists() {
            warn!("[RUST_PACKAGES] No Cargo.toml found at {:?}", cargo_toml);
            return Ok(Vec::new());
        }
        
        // Read Cargo.toml to get dependencies
        let toml_content = match tokio::fs::read_to_string(&cargo_toml).await {
            Ok(content) => content,
            Err(e) => {
                error!("[RUST_PACKAGES] Failed to read Cargo.toml: {}", e);
                return Ok(Vec::new());
            }
        };
        
        info!("[RUST_PACKAGES] Cargo.toml content:\n{}", toml_content);
        
        // Parse TOML to extract dependencies
        let toml: toml::Value = match toml::from_str(&toml_content) {
            Ok(v) => v,
            Err(e) => {
                error!("[RUST_PACKAGES] Failed to parse Cargo.toml: {}", e);
                return Ok(Vec::new());
            }
        };
        
        let mut result = Vec::new();
        
        // Extract regular dependencies
        if let Some(deps) = toml.get("dependencies").and_then(|v| v.as_table()) {
            for (name, value) in deps {
                let version = match value {
                    toml::Value::String(v) => v.clone(),
                    toml::Value::Table(t) => {
                        t.get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("*")
                            .to_string()
                    }
                    _ => "*".to_string(),
                };
                
                info!("[RUST_PACKAGES] Found dependency: {} = {}", name, version);
                
                result.push(Package {
                    name: name.clone(),
                    version,
                    description: None,
                    size: None,
                    dependencies: Vec::new(),
                    is_dev_dependency: false,
                });
            }
        }
        
        // Extract dev dependencies
        if let Some(dev_deps) = toml.get("dev-dependencies").and_then(|v| v.as_table()) {
            for (name, value) in dev_deps {
                let version = match value {
                    toml::Value::String(v) => v.clone(),
                    toml::Value::Table(t) => {
                        t.get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("*")
                            .to_string()
                    }
                    _ => "*".to_string(),
                };
                
                info!("[RUST_PACKAGES] Found dev-dependency: {} = {}", name, version);
                
                result.push(Package {
                    name: name.clone(),
                    version,
                    description: None,
                    size: None,
                    dependencies: Vec::new(),
                    is_dev_dependency: true,
                });
            }
        }
        
        info!("[RUST_PACKAGES] Total packages found: {}", result.len());
        Ok(result)
    }

    async fn list_java_packages(&self, env_path: &Path) -> Result<Vec<Package>> {
        let output = command_executor::run_in_dir("mvn", &["dependency:list", "-DoutputType=text", "-DincludeScope=runtime", "--batch-mode"], env_path).await.unwrap_or_default();
        let mut result = Vec::new();
        for line in output.lines() {
            let line = line.trim();
            if line.starts_with("[INFO]    ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                     // The part usually is groupId:artifactId:type:version:scope
                     let spec = parts[1];
                     let segments: Vec<&str> = spec.split(':').collect();
                     if segments.len() >= 4 {
                         result.push(Package {
                             name: format!("{}:{}", segments[0], segments[1]),
                             version: segments[3].to_string(),
                             description: None,
                             size: None,
                             dependencies: Vec::new(),
                             is_dev_dependency: false,
                         });
                     }
                }
            }
        }
        Ok(result)
    }

    async fn list_ruby_packages(&self, env_path: &Path) -> Result<Vec<Package>> {
        let output = command_executor::run_in_dir("bundle", &["list"], env_path).await.unwrap_or_default();
        let mut result = Vec::new();
        for line in output.lines() {
            let line = line.trim();
            if line.starts_with('*') {
                if let Some(start_paren) = line.rfind('(') {
                    if let Some(end_paren) = line.rfind(')') {
                        let name = line[1..start_paren].trim();
                        let version = &line[start_paren+1..end_paren];
                        result.push(Package {
                            name: name.to_string(),
                            version: version.to_string(),
                            description: None,
                            size: None,
                            dependencies: Vec::new(),
                            is_dev_dependency: false,
                        });
                    }
                }
            }
        }
        Ok(result)
    }

    async fn list_php_packages(&self, env_path: &Path) -> Result<Vec<Package>> {
         let output = command_executor::run_in_dir("composer", &["show", "--format=json"], env_path).await.unwrap_or_else(|_| "{}".to_string());
         let json: serde_json::Value = serde_json::from_str(&output).unwrap_or(serde_json::json!({}));
         let mut result = Vec::new();
         if let Some(installed) = json["installed"].as_array() {
             for pkg in installed {
                 if let (Some(name), Some(version)) = (pkg["name"].as_str(), pkg["version"].as_str()) {
                      result.push(Package {
                          name: name.to_string(),
                          version: version.to_string(),
                          description: pkg["description"].as_str().map(|s| s.to_string()),
                          size: None,
                          dependencies: Vec::new(),
                          is_dev_dependency: false,
                      });
                 }
             }
         }
         Ok(result)
    }

    fn load_default_templates() -> Vec<EnvironmentTemplate> {
        vec![
            EnvironmentTemplate {
                id: "python-basic".to_string(),
                name: "Python Basic".to_string(),
                description: "Basic Python environment with common utilities".to_string(),
                language: Language::Python,
                packages: vec!["pip".to_string(), "setuptools".to_string(), "wheel".to_string()],
                scripts: HashMap::new(),
                files: HashMap::new(),
                settings: HashMap::new(),
            },
            EnvironmentTemplate {
                id: "python-data-science".to_string(),
                name: "Python Data Science".to_string(),
                description: "Python environment with data science packages".to_string(),
                language: Language::Python,
                packages: vec![
                    "numpy".to_string(),
                    "pandas".to_string(),
                    "scikit-learn".to_string(),
                    "matplotlib".to_string(),
                    "seaborn".to_string(),
                    "jupyter".to_string(),
                ],
                scripts: HashMap::new(),
                files: HashMap::new(),
                settings: HashMap::new(),
            },
        ]
    }
}

#[async_trait::async_trait]
impl crate::services::Service for VirtualEnvService {
    async fn start(&mut self) -> Result<()> {
        info!("[VIRTENV_SERVICE_START] VirtualEnv service starting");
        info!("[VIRTENV_SERVICE_START] Currently have {} environments in memory", self.environments.len());
        info!("[VIRTENV_SERVICE_START] Environment IDs: {:?}", self.environments.keys().collect::<Vec<_>>());
        Ok(())
    }

    async fn run(mut self) -> Result<()> {
        let poll_interval = self.poll_interval;
        
        loop {
            tokio::select! {
                biased;
                maybe_cmd = self.cmd_rx.recv() => {
                    if let Some(cmd) = maybe_cmd {
                        self.handle_command(cmd).await;
                    } else {
                        break Ok(());
                    }
                }
                _ = tokio::time::sleep(poll_interval) => {
                    match count_python_envs().await {
                        Ok(total) => {
                            let summary = VirtualEnvSummary { total, active: 0 };
                            self.bus.publish(Event::VirtualEnvSummary(summary));
                        }
                        Err(err) => {
                            info!(error = ?err, "virtenv scan failed");
                        }
                    }
                }
            }
        }
    }
}

impl VirtualEnvService {    
    async fn handle_command(&mut self, cmd: Command) {
        match cmd {
            Command::VirtEnvCreate { request } => {
                info!("Received VirtEnvCreate command");
                match self.create_environment(request).await {
                    Ok(env) => {
                        info!("Successfully created environment: {}", env.name);
                        // Only publish the created environment event - frontend will add it to its list
                        self.bus.publish(Event::VirtualEnvCreated { environment: env.clone() });
                    }
                    Err(e) => {
                        error!("Failed to create environment: {}", e);
                        self.bus.publish(Event::VirtualEnvError { 
                            message: format!("Failed to create environment: {}", e) 
                        });
                    }
                }
            }
            Command::VirtEnvDelete { env_id } => {
                info!("Received VirtEnvDelete command: {}", env_id);
                match self.delete_environment(&env_id).await {
                    Ok(_) => {
                        info!("Successfully deleted environment: {}", env_id);
                        let environments = self.get_environments().await;
                        self.bus.publish(Event::VirtualEnvList(environments));
                    }
                    Err(e) => {
                        error!("Failed to delete environment: {}", e);
                        self.bus.publish(Event::VirtualEnvError { 
                            message: format!("Failed to delete environment: {}", e) 
                        });
                    }
                }
            }
            Command::VirtEnvActivate { env_id } => {
                info!("Received VirtEnvActivate command: {}", env_id);
                match self.activate_environment(&env_id).await {
                    Ok(_) => {
                        info!("Successfully activated environment: {}", env_id);
                        let environments = self.get_environments().await;
                        self.bus.publish(Event::VirtualEnvList(environments));
                    }
                    Err(e) => {
                        error!("Failed to activate environment: {}", e);
                        self.bus.publish(Event::VirtualEnvError { 
                            message: format!("Failed to activate environment: {}", e) 
                        });
                    }
                }
            }
            Command::VirtEnvDeactivate { env_id } => {
                info!("Received VirtEnvDeactivate command: {}", env_id);
                if let Some(env) = self.environments.get_mut(&env_id) {
                    env.is_active = false;
                    self.active_env = None;
                    info!("Successfully deactivated environment: {}", env_id);
                    let environments = self.get_environments().await;
                    self.bus.publish(Event::VirtualEnvList(environments));
                }
            }
            Command::VirtEnvInstallPackages { operation } => {
                info!("[CMD_HANDLER] Received VirtEnvInstallPackages command");
                info!("[CMD_HANDLER] Operation details: env_id={}, packages={:?}", operation.env_id, operation.packages);
                info!("[CMD_HANDLER] Current environments in memory: {}", self.environments.len());
                match self.install_packages(operation).await {
                    Ok(_) => {
                        info!("[CMD_HANDLER] Package installation completed successfully");
                    }
                    Err(e) => {
                        error!("[CMD_HANDLER] Failed to install packages: {}", e);
                    }
                }
            }
            Command::VirtEnvList => {
                info!("Received VirtEnvList command");
                // Refresh packages to ensure list is up to date
                self.refresh_all_packages().await;
                let environments = self.get_environments().await;
                self.bus.publish(Event::VirtualEnvList(environments));
            }
            Command::VirtEnvGetTemplates => {
                info!("Received VirtEnvGetTemplates command");
                let templates = self.get_templates().to_vec();
                self.bus.publish(Event::VirtualEnvTemplates(templates));
            }
            _ => {} // Ignore non-virtenv commands
        }
    }
}

async fn count_python_envs() -> Result<usize> {
    let mut total = 0usize;
    for root in python_env_roots() {
        if !root.exists() {
            continue;
        }
        total += count_envs_in_dir(&root).await.unwrap_or(0);
    }
    Ok(total)
}

async fn count_envs_in_dir(root: &Path) -> Result<usize> {
    let mut count = 0usize;
    let mut entries = fs::read_dir(root).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if entry.file_type().await?.is_dir() {
            if is_python_venv(&path).await {
                count += 1;
            }
        }
    }
    Ok(count)
}

async fn is_python_venv(path: &Path) -> bool {
    let cfg = path.join("pyvenv.cfg");
    fs::metadata(cfg).await.is_ok()
}

fn python_env_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(home) = env::var("HOME") {
        roots.push(PathBuf::from(format!("{home}/.virtualenvs")));
        roots.push(PathBuf::from(format!("{home}/.local/share/virtualenvs")));
        roots.push(PathBuf::from(format!("{home}/.venvs")));
    }
    if let Ok(workon_home) = env::var("WORKON_HOME") {
        roots.push(PathBuf::from(workon_home));
    }
    roots
}

impl VirtualEnvService {
    async fn calculate_env_size(path: &Path) -> Result<u64> {
        let mut size = 0;
        let mut entries = fs::read_dir(path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;
            if metadata.is_dir() {
                size += Box::pin(Self::calculate_env_size(&entry.path())).await?;
            } else {
                size += metadata.len();
            }
        }
        Ok(size)
    }

    async fn update_env_sizes(&mut self) {
        for env in self.environments.values_mut() {
            if let Ok(size) = Self::calculate_env_size(&env.path).await {
                env.size_mb = Some(size / 1024 / 1024);
            }
        }
    }
}
