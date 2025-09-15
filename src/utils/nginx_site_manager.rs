// src/utils/nginx_site_manager.rs

use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};
use shiplift::{Docker, ExecContainerOptions};
use futures_util::TryStreamExt;

pub struct NginxSiteManager {
    sites_dir: PathBuf,
    docker: Docker, 
}

impl NginxSiteManager {
    pub fn new(sites_dir: &str, docker: Docker) -> Self {
        Self {
            sites_dir: PathBuf::from(sites_dir),
            docker,
        }
    }

    // Creates and manages an Nginx reverse proxy configuration.
    pub async fn create_reverse_proxy(
        &self, 
        site_name: &str, 
        container_name: &str, 
        port: i32
    ) -> Result<()> {
        fs::create_dir_all(&self.sites_dir)?;
        let conf_path = self.sites_dir.join(format!("{}.conf", site_name));

        // Add this line to remove the existing config file if it exists.
        if conf_path.exists() {
            fs::remove_file(&conf_path)
                .with_context(|| format!("Failed to remove existing nginx conf file: {:?}", conf_path))?;
            println!("🗑️ Removed old nginx conf: {:?}", conf_path);
        }

        let content = format!(
            r#"
                server {{
                    listen 80;
                    server_name {site_name}.localhost;

                    location / {{
                        proxy_pass http://{container_name}:{port};
                        proxy_http_version 1.1;
                        proxy_set_header Upgrade $http_upgrade;
                        proxy_set_header Connection 'upgrade';
                        proxy_set_header Host $host;
                        proxy_set_header X-Real-IP $remote_addr;
                        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
                        proxy_set_header X-Forwarded-Proto $scheme;
                        proxy_cache_bypass $http_upgrade;
                    }}
                }}
                "#
        );
        
        fs::write(&conf_path, content)
            .with_context(|| format!("Failed to write nginx conf file: {:?}", conf_path))?;
        println!("📝 Nginx conf created: {:?}", conf_path);
        self.reload_nginx().await
    }
    
    /// Reloads the Nginx service configuration inside its container.
    pub async fn reload_nginx(&self) -> Result<()> {
        let container = self.docker.containers().get("cicd-nginx");
        let exec_options = ExecContainerOptions::builder()
            .cmd(vec!["nginx", "-s", "reload"])
            .build();
        
        let mut stream = container.exec(&exec_options);
        
        while let Some(result) = stream.try_next().await? {
            println!("Nginx reload output: {:?}", result);
        }
        
        println!("🔄 Nginx reloaded successfully via Docker exec.");
        Ok(())
    }

    /// Removes a site configuration file and reloads Nginx.
    pub async fn remove_site(&self, site_name: &str) -> Result<()> {
        let conf_path = self.sites_dir.join(format!("{}.conf", site_name));
        
        if conf_path.exists() {
            fs::remove_file(&conf_path)
                .with_context(|| format!("Failed to remove nginx conf file: {:?}", conf_path))?;
            println!("🗑️ Removed nginx conf: {:?}", conf_path);
            self.reload_nginx().await?;
        }
        
        Ok(())
    }
}