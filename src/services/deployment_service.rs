use anyhow::{Context, Result};
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use tokio::time::{sleep, Duration};
use std::fs;
use std::path::PathBuf;
use std::env;
use std::sync::Arc;

use crate::repository::deployment_repository::DeploymentRepository;
use crate::models::deployment::Deployment;
use crate::utils::nginx_site_manager::NginxSiteManager;
use crate::utils::react_server::ReactServer;


pub struct DeploymentService {
    pub repository: DeploymentRepository,
    react_servers: Arc<Mutex<HashMap<String, ReactServer>>>,
}

impl DeploymentService {
    pub fn new(repository: DeploymentRepository) -> Self {
        Self { 
            repository,
            react_servers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn get_deployment_dir(&self, repo_name: &str, branch: &str) -> Result<PathBuf> {
        let base_dir = env::current_dir()
            .with_context(|| "Failed to get current working directory")?;

        let mut deployment_dir = base_dir.join("deployments");
        if !deployment_dir.exists() {
            println!("📂 Creating deployments folder at {:?}", deployment_dir);
            fs::create_dir_all(&deployment_dir)?;
        }

        deployment_dir.push(format!("{}-{}", repo_name, branch));
        Ok(deployment_dir)
    }

    pub async fn deploy_project(
        &self,
        user_id: i64,
        repo_url: &str,
        repo_name: &str,
        branch: &str,
    ) -> Result<Deployment> {
        println!("🚀 Starting deployment for repo={} branch={}", repo_name, branch);

        let mut deployment = self.repository.create_deployment(user_id, repo_name, branch).await?;
        println!("📝 Deployment record created in DB with id={}", deployment.id);

        let target_dir = self.get_deployment_dir(repo_name, branch)?;
        if target_dir.exists() {
            println!("🧹 Removing old target dir: {:?}", target_dir);
            let _ = fs::remove_dir_all(&target_dir);
        }

        fs::create_dir_all(&target_dir)?;
        println!("📂 Target dir ready: {:?}", target_dir);

        // Clone repository
        println!("📡 Cloning repository from {}", repo_url);
        let clone_logs = self.repository.clone_repo(repo_url, branch, target_dir.to_str().unwrap())?;
        println!("✅ Clone finished");
        self.repository.update_status(deployment.id, "cloned", Some(&clone_logs)).await?;

        // Detect project type
        let project_type = self.detect_project_type(target_dir.to_str().unwrap())?;
        println!("🔍 Detected project type: {}", project_type);
        self.repository.update_status(deployment.id, &format!("detected: {}", project_type), None).await?;

        // For React projects: install dependencies and build
        if project_type == "react" {
            println!("📦 Installing npm dependencies...");
            let install_logs = self.run_npm_install(target_dir.to_str().unwrap())?;
            println!("✅ Dependencies installed");
            self.repository.update_status(deployment.id, "dependencies_installed", Some(&install_logs)).await?;

            println!("🔨 Building React project...");
            let build_logs = self.run_npm_build(target_dir.to_str().unwrap())?;
            println!("✅ Build completed");
            self.repository.update_status(deployment.id, "built", Some(&build_logs)).await?;
        } else {
            println!("⚠️  Project is not React. Currently only React projects are supported.");
            self.repository.update_status(deployment.id, "unsupported_project_type", None).await?;
            return Err(anyhow::anyhow!("Only React projects are supported at this time"));
        }

        // Start React server instead of using Nginx
        let site_name = format!("{}-{}", repo_name, branch);
        
        // Determine build directory for React apps
        let dist = target_dir.join("dist");
        let cra_build = target_dir.join("build");

        let build_dir = if cra_build.exists() {
            cra_build
        } else if dist.exists() {
            dist
        } else {
            return Err(anyhow::anyhow!("No build directory found. Expected 'build' or 'dist' folder after npm run build"));
        };

        // Get an available port
        let port = self.get_available_port().await?;
        
        // Start the React server
        println!("🌐 Starting React server on port {}...", port);
        let react_server = ReactServer::serve_static_files(build_dir, port).await?;
        
        // Store the server for later management
        let mut servers = self.react_servers.lock().await;
        servers.insert(site_name.clone(), react_server);

        println!("🌍 React app running on port: {}", port);
        println!("📋 Available at: http://localhost:{}", port);
        println!("🏷️  Site name: {}", site_name);

        // Update deployment with port information
        deployment.port = Some(port as i32);
        // self.repository.update_deployment_port(deployment.id, port as i32).await?;
        self.repository.update_status(deployment.id, "deployed", None).await?;

        Ok(deployment)
    }

    async fn get_available_port(&self) -> Result<u16> {
        // Simple port assignment starting from 3000
        for port in 3000..4000 {
            if self.is_port_available(port).await {
                return Ok(port);
            }
        }
        Err(anyhow::anyhow!("No available ports found"))
    }

    async fn is_port_available(&self, port: u16) -> bool {
        use std::net::TcpListener;
        TcpListener::bind(("0.0.0.0", port)).is_ok()
    }

    // Add method to stop servers when needed
    pub async fn stop_server(&self, site_name: &str) -> Result<()> {
        let mut servers = self.react_servers.lock().await;
        if let Some(server) = servers.remove(site_name) {
            server.shutdown().await;
            println!("🛑 Stopped server for: {}", site_name);
        }
        Ok(())
    }

    fn detect_project_type(&self, dir: &str) -> Result<String> {
        let package_json_path = format!("{}/package.json", dir);
        
        if let Ok(metadata) = fs::metadata(&package_json_path) {
            if metadata.is_file() {
                let pkg_content = fs::read_to_string(&package_json_path)?;
                
                // Check for React-specific dependencies or scripts
                if pkg_content.contains("\"react\"") || 
                   pkg_content.contains("\"react-scripts\"") ||
                   pkg_content.contains("\"vite\"") && pkg_content.contains("\"@vitejs/plugin-react\"") {
                    return Ok("react".into());
                }
                
                if pkg_content.contains("\"next\"") {
                    return Ok("next".into());
                }
                
                if pkg_content.contains("\"vue\"") {
                    return Ok("vue".into());
                }
                
                return Ok("node".into());
            }
        }

        // Check for other project types but focus on React
        let rust_file = format!("{}/Cargo.toml", dir);
        let python_file = format!("{}/requirements.txt", dir);
        let index_html = format!("{}/index.html", dir);

        if fs::metadata(&rust_file).is_ok() { return Ok("rust".into()); }
        if fs::metadata(&python_file).is_ok() { return Ok("python".into()); }
        if fs::metadata(&index_html).is_ok() { return Ok("static".into()); }

        Ok("unknown".into())
    }

    fn run_npm_install(&self, dir: &str) -> Result<String> {
        #[cfg(target_os = "windows")]
        let npm_cmd = "npm.cmd";
        #[cfg(not(target_os = "windows"))]
        let npm_cmd = "npm";

        // Try npm install with retries
        let max_retries = 3;
        let mut attempt = 0;
        
        while attempt < max_retries {
            let output = Command::new(npm_cmd)
                .arg("install")
                .arg("--no-audit") // Skip audit to speed things up
                .arg("--no-fund")  // Skip fund messages
                .current_dir(dir)
                .output()
                .with_context(|| format!("Failed to run npm install in {}", dir))?;

            if output.status.success() {
                return Ok(format!(
                    "npm install output:\n{}\n\nErrors:\n{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ));
            }

            // Check if it's a network error that we should retry
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("timeout") || stderr.contains("network") || stderr.contains("ECONNREFUSED") {
                attempt += 1;
                println!("⚠️  Network issue detected, retrying npm install (attempt {}/{})...", attempt, max_retries);
                
                // Wait before retrying (exponential backoff)
                let wait_time = Duration::from_secs(2u64.pow(attempt as u32));
                std::thread::sleep(wait_time);
                continue;
            }

            // If it's not a network error, fail immediately
            return Err(anyhow::anyhow!(
                "npm install failed: {}",
                stderr
            ));
        }

        Err(anyhow::anyhow!(
            "npm install failed after {} attempts due to network issues",
            max_retries
        ))
    }

    fn run_npm_build(&self, dir: &str) -> Result<String> {
        #[cfg(target_os = "windows")]
        let npm_cmd = "npm.cmd";
        #[cfg(not(target_os = "windows"))]
        let npm_cmd = "npm";

        let output = Command::new(npm_cmd)
            .args(&["run", "build"])
            .current_dir(dir)
            .output()
            .with_context(|| format!("Failed to run npm run build in {}", dir))?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "npm run build failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(format!(
            "npm run build output:\n{}\n\nErrors:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
    

    
}