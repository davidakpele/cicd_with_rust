use std::fs;
use std::path::PathBuf;
use std::process::Command;
use anyhow::{Context, Result};

pub struct NginxSiteManager {
    base_dir: String,       // Where generated conf files live (e.g. ./nginx/sites)
    nginx_container: String // Name of your nginx container
}

impl NginxSiteManager {
    pub fn new(base_dir: &str, nginx_container: &str) -> Self {
        Self {
            base_dir: base_dir.into(),
            nginx_container: nginx_container.into(),
        }
    }

    /// Create nginx.conf for static apps (React, Next, etc.)
    pub fn create_static_site(&self, app_name: &str, root_path: &str) -> Result<()> {
        let conf_path = PathBuf::from(&self.base_dir).join(format!("{}.conf", app_name));

        let conf_content = format!(
            r#"
server {{
    listen 80;
    server_name {app_name}.localhost;

    root {root_path};
    index index.html;

    location / {{
        try_files $uri /index.html;
    }}
}}
"#,
            app_name = app_name,
            root_path = root_path
        );

        fs::create_dir_all(&self.base_dir)?;
        fs::write(&conf_path, conf_content)
            .with_context(|| format!("Failed to write nginx conf file: {:?}", conf_path))?;

        self.reload_nginx()
    }

    /// Create nginx.conf for backend APIs (Rust, Python, Go, Java).
    pub fn create_backend_site(&self, app_name: &str, port: i32) -> Result<()> {
        let conf_path = PathBuf::from(&self.base_dir).join(format!("{}.conf", app_name));

        let conf_content = format!(
            r#"
server {{
    listen 80;
    server_name {app_name}.localhost;

    location / {{
        proxy_pass http://127.0.0.1:{port};
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }}
}}
"#,
            app_name = app_name,
            port = port
        );

        fs::create_dir_all(&self.base_dir)?;
        fs::write(&conf_path, conf_content)
            .with_context(|| format!("Failed to write nginx conf file: {:?}", conf_path))?;

        self.reload_nginx()
    }

    fn reload_nginx(&self) -> Result<()> {
        let output = Command::new("docker")
            .args(&["exec", &self.nginx_container, "nginx", "-s", "reload"])
            .output()
            .with_context(|| "Failed to reload nginx inside Docker")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Nginx reload failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        println!("🔄 Nginx reloaded successfully.");
        Ok(())
    }
}
