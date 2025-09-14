use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};
use std::process::Command;

pub struct NginxSiteManager {
    sites_dir: PathBuf,
}

impl NginxSiteManager {
    pub fn new(sites_dir: &str) -> Self {
        Self {
            sites_dir: PathBuf::from(sites_dir),
        }
    }

    /// Create static site for React apps
    pub fn create_static_site(&self, site_name: &str, root_path: &str) -> Result<()> {
        self.create_site_conf(site_name, root_path)?;
        self.reload_nginx()
    }

    fn create_site_conf(&self, site_name: &str, root_path: &str) -> Result<()> {
        fs::create_dir_all(&self.sites_dir)?;

        let conf_path = self.sites_dir.join(format!("{}.conf", site_name));

        let content = format!(
            r#"
server {{
    listen 80;
    server_name {site_name}.localhost;

    root {root_path};
    index index.html;

    # Serve static files
    location / {{
        try_files $uri $uri/ /index.html;
        add_header Cache-Control "no-cache, no-store, must-revalidate";
        add_header Pragma "no-cache";
        add_header Expires "0";
    }}

    # API proxy for future backend integration
    location /api/ {{
        proxy_pass http://localhost:3001;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }}

    # Health check endpoint
    location /health {{
        access_log off;
        return 200 "healthy";
        add_header Content-Type text/plain;
    }}
}}
"#,
            site_name = site_name,
            root_path = root_path
        );

        fs::write(&conf_path, content)
            .with_context(|| format!("Failed to write nginx conf file: {:?}", conf_path))?;

        println!("📝 Nginx conf created: {:?}", conf_path);
        Ok(())
    }

    fn reload_nginx(&self) -> Result<()> {
        // Test nginx configuration first
        let test_output = Command::new("nginx")
            .args(&["-t"])
            .output()
            .with_context(|| "Failed to test nginx configuration")?;

        if !test_output.status.success() {
            return Err(anyhow::anyhow!(
                "Nginx configuration test failed: {}",
                String::from_utf8_lossy(&test_output.stderr)
            ));
        }

        // Reload nginx
        let reload_output = Command::new("nginx")
            .args(&["-s", "reload"])
            .output()
            .with_context(|| "Failed to reload nginx")?;

        if !reload_output.status.success() {
            return Err(anyhow::anyhow!(
                "Nginx reload failed: {}",
                String::from_utf8_lossy(&reload_output.stderr)
            ));
        }

        println!("🔄 Nginx reloaded successfully.");
        Ok(())
    }

    /// Optional: Clean up site configuration
    pub fn remove_site(&self, site_name: &str) -> Result<()> {
        let conf_path = self.sites_dir.join(format!("{}.conf", site_name));
        
        if conf_path.exists() {
            fs::remove_file(&conf_path)
                .with_context(|| format!("Failed to remove nginx conf file: {:?}", conf_path))?;
            println!("🗑️ Removed nginx conf: {:?}", conf_path);
            self.reload_nginx()?;
        }
        
        Ok(())
    }
}