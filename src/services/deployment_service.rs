// src/services/deployment_service.rs

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::env;
use crate::repository::deployment_repository::DeploymentRepository;
use crate::models::deployment::Deployment;
use crate::utils::docker_manager::DockerManager;
use crate::utils::nginx_site_manager::NginxSiteManager;

pub struct DeploymentService {
    pub repository: DeploymentRepository,
    docker_manager: DockerManager,
    nginx_manager: NginxSiteManager,
}

impl DeploymentService {
    pub fn new(repository: DeploymentRepository, docker_manager: DockerManager, nginx_manager: NginxSiteManager) -> Self {
        Self {
            repository,
            docker_manager,
            nginx_manager,
        }
    }

    fn get_deployment_dir(&self, repo_name: &str, branch: &str) -> Result<PathBuf> {
        let base_dir = env::current_dir()
            .with_context(|| "Failed to get current working directory")?;
        let deployment_dir = base_dir.join("deployments").join(format!("{}-{}", repo_name, branch));
        Ok(deployment_dir)
    }

    fn detect_project_type(&self, dir: &str) -> Result<String> {
        let package_json_path = format!("{}/package.json", dir);
        if let Ok(metadata) = fs::metadata(&package_json_path) {
            if metadata.is_file() {
                let pkg_content = fs::read_to_string(&package_json_path)?;
                if pkg_content.contains("\"react\"") || pkg_content.contains("\"react-scripts\"") || pkg_content.contains("\"@vitejs/plugin-react\"") {
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
        let cargo_toml_path = format!("{}/Cargo.toml", dir);
        if fs::metadata(&cargo_toml_path).is_ok() {
            return Ok("rust".into());
        }
        let requirements_txt_path = format!("{}/requirements.txt", dir);
        if fs::metadata(&requirements_txt_path).is_ok() {
            return Ok("python".into());
        }
        let index_html_path = format!("{}/index.html", dir);
        if fs::metadata(&index_html_path).is_ok() {
            return Ok("static".into());
        }
        Ok("unknown".into())
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
            fs::remove_dir_all(&target_dir)?;
        }
        fs::create_dir_all(&target_dir)?;

        println!("📡 Cloning repository from {}...", repo_url);
        let clone_logs = self.repository.clone_repo(repo_url, branch, target_dir.to_str().unwrap())?;
        self.repository.update_status(deployment.id, "cloned", Some(&clone_logs), None).await?;

        println!("🔍 Detecting project type...");
        let project_type = self.detect_project_type(target_dir.to_str().unwrap())?;
        self.repository.update_status(deployment.id, &format!("detected: {}", project_type), None, None).await?;
        
        if project_type == "unknown" {
            self.repository.update_status(deployment.id, "unsupported_project_type", None, None).await?;
            return Err(anyhow::anyhow!("Unsupported project type."));
        }

        println!("📋 Generating Dockerfile for {}...", project_type);
        let dockerfile_template = match project_type.as_str() {
            "react" | "next" | "vue" | "node" => include_str!("../../dockerfiles/node.Dockerfile"),
            "rust" => include_str!("../../dockerfiles/rust.Dockerfile"),
            "python" => include_str!("../../dockerfiles/python.Dockerfile"),
            "static" => include_str!("../../dockerfiles/static.Dockerfile"),
            _ => {
                self.repository.update_status(deployment.id, "unsupported_project_type", None, None).await?;
                return Err(anyhow::anyhow!("Unsupported project type for Dockerfile generation."));
            }
        };

        fs::write(target_dir.join("Dockerfile"), dockerfile_template)?;
        
        // 1. Build the Docker image
        println!("🔨 Building Docker image...");
        let image_name = format!("{}-{}", repo_name, branch);
        let container_name = format!("{}-{}", repo_name, branch);

        println!("✨ Cleaning up previous deployment (if any)...");
        self.docker_manager.stop_and_remove_container(&container_name).await?;

        println!("🔨 Building Docker image...");
        self.docker_manager.build_image(target_dir.to_str().unwrap(), &image_name).await?;
        self.repository.update_status(deployment.id, "image_built", None, None).await?;
        
        // 2. Run the Docker container
        println!("📦 Running Docker container...");
        let (container_id, _, host_port) = self.docker_manager.run_container(&image_name, &container_name, None).await?;
        self.repository.update_status(deployment.id, "container_running", None, None).await?;

        // 3. Update the deployment record with container details
        deployment.container_id = Some(container_id);
        deployment.port = Some(host_port as i32); 

        // 4. Update Nginx configuration
        println!("🔄 Updating Nginx configuration...");
        let site_name = format!("{}-{}", repo_name, branch);
        let live_url = format!("http://{}.localhost", site_name); // Construct the live URL
        self.nginx_manager.create_reverse_proxy(&site_name, &container_name, host_port as i32).await?; 

        // 5. Update the deployment record with the final URL
        deployment.live_url = Some(live_url);
        self.repository.update_status(deployment.id, "deployed", None, live_url).await?;

        println!("🎉 Deployment completed successfully!");
        println!("📊 Deployment ID: {}", deployment.id);
        println!("🔗 Live URL: {}", deployment.live_url.as_ref().unwrap_or(&"N/A".to_string()));

        Ok(deployment)
    }


}