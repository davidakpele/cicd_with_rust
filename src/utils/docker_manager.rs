// src/utils/docker_manager.rs
use shiplift::{
    builder::{ContainerOptions, RmContainerOptions},
    Docker,
};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::{Result, Context};

#[derive(Clone)]
pub struct DockerManager {
    pub(crate) docker: Docker,
    port_pool: Arc<Mutex<PortPool>>,
}

#[derive(Debug)]
struct PortPool {
    start_port: u16,
    end_port: u16,
    used_ports: HashSet<u16>,
}

impl PortPool {
    fn new(start_port: u16, end_port: u16) -> Self {
        PortPool {
            start_port,
            end_port,
            used_ports: HashSet::new(),
        }
    }

    fn get_available_port(&mut self) -> Result<u16, anyhow::Error> {
        for port in self.start_port..=self.end_port {
            if !self.used_ports.contains(&port) {
                self.used_ports.insert(port);
                return Ok(port);
            }
        }
        Err(anyhow::anyhow!("No available ports in the pool"))
    }

    fn release_port(&mut self, port: u16) {
        self.used_ports.remove(&port);
    }
}

impl DockerManager {
    pub fn new(start_port: u16, end_port: u16) -> Self {
        let port_pool = PortPool::new(start_port, end_port);
        DockerManager {
            docker: Docker::new(),
            port_pool: Arc::new(Mutex::new(port_pool)),
        }
    }

    pub async fn build_image(&self, context_path: &str, image_name: &str) -> Result<()> {
        println!("Building Docker image from path: {}", context_path);
        
        let output = std::process::Command::new("docker")
            .arg("build")
            .arg("-t")
            .arg(image_name)
            .arg(context_path)
            .output()
            .context("Failed to execute docker build")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Docker build failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        println!("✅ Docker image built successfully: {}", image_name);
        Ok(())
    }

    pub async fn run_container(&self, image_name: &str, container_name: &str, host_port: Option<u16>) -> Result<(String, String, u16)> {
        let mut port_pool = self.port_pool.lock().await;
        let selected_port = host_port.unwrap_or_else(|| {
            port_pool.get_available_port().unwrap_or(8080)
        });

        // Clean up any existing container with the same name FIRST
        self.force_remove_container(container_name).await?;

        let options = ContainerOptions::builder(image_name)
            .name(container_name)
            .expose(selected_port as u32, "tcp", 80)
            // Add network configuration to ensure connectivity with nginx
            .build();

        let container_info = self.docker
            .containers()
            .create(&options)
            .await
            .context("Failed to create container")?;

        self.docker
            .containers()
            .get(&container_info.id)
            .start()
            .await
            .context("Failed to start container")?;

        println!("Container {} started on port {}", container_info.id, selected_port);
        Ok((container_info.id, container_name.to_string(), selected_port))
    }


    pub async fn stop_container(&self, container_id: &str) -> Result<()> {
        self.docker
            .containers()
            .get(container_id)
            .stop(None)
            .await
            .context("Failed to stop container")?;
        Ok(())
    }

    pub async fn remove_container(&self, container_id: &str) -> Result<()> {
        let options = RmContainerOptions::builder().force(true).build();
        self.docker
            .containers()
            .get(container_id)
            .remove(options)
            .await
            .context("Failed to remove container")?;
        Ok(())
    }

    pub async fn stop_and_remove_container(&self, container_name: &str) -> Result<()> {
        let container = self.docker.containers().get(container_name);
        
        if let Ok(_) = container.inspect().await {
            println!("🛑 Stopping and removing old container: {}", container_name);
            let options = RmContainerOptions::builder().force(true).build();
            
            if let Err(e) = container.remove(options).await {
                eprintln!("Failed to remove container {}: {}", container_name, e);
            } else {
                println!("✅ Container removed: {}", container_name);
            }
        } else {
            println!("No container named {} found. Skipping removal.", container_name);
        }
        Ok(())
    }

    pub async fn force_remove_container(&self, container_name: &str) -> Result<()> {
        match self.docker.containers().get(container_name).inspect().await {
            Ok(_) => {
                println!("🛑 Removing existing container: {}", container_name);
                let options = RmContainerOptions::builder().force(true).build();
                if let Err(e) = self.docker.containers().get(container_name).remove(options).await {
                    eprintln!("Warning: Failed to remove container {}: {}", container_name, e);
                }
            }
            Err(_) => {
                println!("No existing container named: {}", container_name);
            }
        }
        Ok(())
    }


}