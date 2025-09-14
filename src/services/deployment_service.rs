use anyhow::{Context, Result};
use std::process::{Command, Stdio};
use std::fs;
use std::path::PathBuf;
use std::env;

use crate::repository::deployment_repository::DeploymentRepository;
use crate::models::deployment::Deployment;
use crate::utils::nginx_site_manager::NginxSiteManager;

pub struct DeploymentService {
    pub repository: DeploymentRepository,
}

impl DeploymentService {
    pub fn new(repository: DeploymentRepository) -> Self {
        Self { repository }
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

        // Clone
        println!("📡 Cloning repository from {}", repo_url);
        let clone_logs = self.repository.clone_repo(repo_url, branch, target_dir.to_str().unwrap())?;
        println!("✅ Clone finished");
        self.repository.update_status(deployment.id, "cloned", Some(&clone_logs)).await?;

        // Detect
        let project_type = self.detect_project_type(target_dir.to_str().unwrap())?;
        println!("🔍 Detected project type: {}", project_type);
        self.repository.update_status(deployment.id, &format!("detected: {}", project_type), None).await?;

        // Build
        println!("🔨 Running build step...");
        let build_logs = self.run_build(target_dir.to_str().unwrap(), &project_type)?;
        println!("✅ Build finished");
        self.repository.update_status(deployment.id, "built", Some(&build_logs)).await?;

        // Assign port
        println!("🔢 Assigning port...");
        let port = self.repository.assign_port(deployment.id).await?;
        println!("🔌 Port assigned: {}", port);
        self.repository.update_status(deployment.id, &format!("port-assigned: {}", port), None).await?;

        // Start (only for backends, not static)
        println!("▶️ Starting project on port {}", port);
        let start_logs = self.run_start(target_dir.to_str().unwrap(), &project_type, port)?;
        println!("✅ Project running on port {}", port);
        self.repository.update_status(deployment.id, "running", Some(&start_logs)).await?;

        // Configure Nginx (use shared sites dir, not inside project)
        let nginx_manager = NginxSiteManager::new("./nginx/sites", "cicd-nginx");
        let site_name = format!("{}-{}", repo_name, branch);

        if project_type == "react" || project_type == "next" || project_type == "static" {
            // Important: Nginx inside container must see the same path via volume
            let build_dir = format!("/deployments/{}/build", site_name);
            nginx_manager.create_static_site(&site_name, &build_dir)?;
            println!("🌍 Static site served via http://{}.localhost", site_name);
        } else {
            nginx_manager.create_backend_site(&site_name, port)?;
            println!("🌍 Backend service available via http://{}.localhost", site_name);
        }

        deployment.port = Some(port);
        Ok(deployment)
    }

    fn detect_project_type(&self, dir: &str) -> Result<String> {
        let node_file = format!("{}/package.json", dir);
        let rust_file = format!("{}/Cargo.toml", dir);
        let python_file = format!("{}/requirements.txt", dir);
        let index_html = format!("{}/index.html", dir);

        println!("🔎 Detecting project type in {}", dir);

        if fs::metadata(&node_file).is_ok() {
            let pkg = fs::read_to_string(&node_file)?;
            if pkg.contains("\"react\"") {
                return Ok("react".into());
            } else if pkg.contains("\"next\"") {
                return Ok("next".into());
            } else {
                return Ok("node".into());
            }
        } else if fs::metadata(&rust_file).is_ok() {
            return Ok("rust".into());
        } else if fs::metadata(&python_file).is_ok() {
            return Ok("python".into());
        } else if fs::metadata(&index_html).is_ok() {
            return Ok("static".into());
        }

        Ok("unknown".into())
    }

    fn run_build(&self, dir: &str, project_type: &str) -> Result<String> {
        #[cfg(target_os = "windows")]
        let npm_cmd = "npm.cmd";
        #[cfg(not(target_os = "windows"))]
        let npm_cmd = "npm";

        match project_type {
            "react" | "next" | "node" => {
                // Step 1: npm install
                let install_output = Command::new(npm_cmd)
                    .args(&["install"])
                    .current_dir(dir)
                    .output()
                    .with_context(|| "Failed to run npm install")?;

                let install_logs = format!(
                    "npm install stdout:\n{}\nstderr:\n{}",
                    String::from_utf8_lossy(&install_output.stdout),
                    String::from_utf8_lossy(&install_output.stderr)
                );

                // Step 2: npm run build (for react/next)
                if project_type == "react" || project_type == "next" {
                    let build_output = Command::new(npm_cmd)
                        .args(&["run", "build"])
                        .current_dir(dir)
                        .output()
                        .with_context(|| "Failed to run npm run build")?;

                    let build_logs = format!(
                        "npm run build stdout:\n{}\nstderr:\n{}",
                        String::from_utf8_lossy(&build_output.stdout),
                        String::from_utf8_lossy(&build_output.stderr)
                    );

                    Ok(format!("{}\n{}", install_logs, build_logs))
                } else {
                    Ok(install_logs)
                }
            }
            "python" => {
                let output = Command::new("pip")
                    .args(&["install", "-r", "requirements.txt"])
                    .current_dir(dir)
                    .output()
                    .with_context(|| "Failed to run pip install")?;

                Ok(format!(
                    "stdout:\n{}\nstderr:\n{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ))
            }
            "rust" => {
                let output = Command::new("cargo")
                    .args(&["build", "--release"])
                    .current_dir(dir)
                    .output()
                    .with_context(|| "Failed to run cargo build")?;

                Ok(format!(
                    "stdout:\n{}\nstderr:\n{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ))
            }
            "static" => Ok("Static site, no build required".into()),
            _ => Ok("No build command found".into()),
        }
    }

    fn run_start(&self, dir: &str, project_type: &str, port: i32) -> Result<String> {
        #[cfg(target_os = "windows")]
        let npm_cmd = "npm.cmd";
        #[cfg(not(target_os = "windows"))]
        let npm_cmd = "npm";

        let (cmd, args): (&str, Vec<String>) = match project_type {
            // For React and Next.js, the build output is static, so no "start" command is needed.
            "react" | "next" => return Ok("Static project, no start command required. Will be served by Nginx.".into()),
            "node" => ("node", vec!["server.js".into()]),
            "python" => ("python", vec!["app.py".into()]),
            "rust" => ("cargo", vec!["run".into()]),
            "static" => return Ok(format!("Static site served on port {}", port)),
            _ => return Ok("No start command found".into()),
        };

        println!("▶️ Running start command: {} {:?} with PORT={}", cmd, args, port);

        let mut child = Command::new(cmd)
            .args(&args)
            .env("PORT", port.to_string())
            .current_dir(dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to run start command {:?} {:?}", cmd, args))?;

        println!("▶️ Process spawned with PID {}", child.id());

        Ok(format!("Process started with PID {}", child.id()))
    }
}
