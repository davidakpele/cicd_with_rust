# 🚀 A Comprehensive CI/CD & PaaS Solution

## 🚀 Overview
Deployer is a powerful and flexible Continuous Integration/Continuous Deployment (CI/CD) and Platform-as-a-Service (PaaS) solution. Built with a focus on speed, security, and simplicity, it automates the process of taking code from a Git repository and deploying it to a live, production-ready environment.<br/><br/>
This platform is language-agnostic, intelligently detecting and deploying applications written in a variety of modern frameworks and languages, including React, Node.js, Python, Rust, and Static Sites. It uses Docker for containerization and a dynamic Nginx reverse proxy to provide a seamless and secure deployment experience.

---

## ✨ Features  

### 🔐 Authentication & Authorization  
- Secure **JWT-based authentication** middleware.  
- Role-based access control (RBAC) with **user and admin claims**.  
- Protects the entire API with token validation.  

### 📦 CRUD API  
- **Users**: register, login, update, delete.  
- **Deployments**: create, update, restart, delete.  
- **Admins**: manage users and monitor deployments.  

### 🐳 Containerized Deployments  
**Docker-Powered Containerization:** Uses Docker to ensure consistency across environments and provides isolation for each deployed application.
- Detects project type automatically:  
  - React, Next.js, Vue, Node.js  
  - Rust  
  - Python  
  - Static sites  
- Generates a **Dockerfile** dynamically.  
- Builds Docker images and runs containers with isolated resources.  
- Resolves port conflicts with a **port pool manager**.  

### 🌐 Reverse Proxy with Nginx  
**Dynamic Nginx Reverse Proxy:** Manages an Nginx server to route traffic to the correct container, providing a clean and professional public-facing URL.
- Each app gets a unique live URL:
```bash
http://<repo>-<branch>.localhost
```

### **Comprehensive Logging & Status Tracking:** Tracks every step of the deployment process, providing real-time status updates and detailed logs for easy debugging.

### 💻 Tech Stack
- Backend: Rust with Actix-Web
- Database: PostgreSQL
- Containerization: Docker
- Reverse Proxy: Nginx
- Authentication: JWT (JSON Web Tokens)
- Database ORM: SQLx


### 🏗️ Architecture
The system is built around a microservices-like architecture where each component has a specific role:
1. **Deployment Service:** The core service written in Rust that orchestrates the entire deployment pipeline.
2. **Docker Manager:** Manages the lifecycle of Docker images and containers, handling builds, runs, and cleanups.
3. **Nginx Site Manager:** Dynamically generates and reloads Nginx configuration files to create and manage live URLs for each deployment.
4. **Database Repository:** Handles all interactions with the PostgreSQL database, providing a clean separation of concerns.
5. **Auth Middleware:** A custom middleware that secures API endpoints using JWT.

## 🚀 Getting Started  
### Prerequisites 
- Rust toolchain (stable)
  - [Rust](https://www.rust-lang.org/)
- Docker
  - [Docker](https://www.docker.com/)
- Nginx
  - [Nginx](https://nginx.org/) (running as a container named `cicd-nginx`)  
- PostgreSQL
  - [PostgreSQL](https://www.postgresql.org/download/)



### Setup  

# Clone this repo
```bash
git clone https://github.com/your-username/paas-platform.git
```
#### Set up environment variables:
- Create a .env file in the root directory.
```bash
cp .env.example .env
# Now, open the .env file and edit the variables
nano .env
````
### Start the infrastructure services:
Use Docker Compose to bring up the database and Nginx server.
```docker
docker-compose up -d
```
### Run migrations:
Run your database migrations to create the necessary tables.
```migration
# Ensure you have sqlx-cli installed first: `cargo install sqlx-cli`
sqlx migrate run --database-url "$DATABASE_URL"
````
cd paas-platform
# Run migrations / database setup (if required)
# Configure your .env file for DB + JWT secret

# Start the backend
```bash
cargo run
```
```json
POST /deploy
{
  "user_id": 1,
  "repo_url": "https://github.com/user/my-react-app.git",
  "repo_name": "my-react-app",
  "branch": "main"
}
```
### Result:
- Docker container built and started.
- Nginx proxy created.
- App live at:
```url
http://my-react-app-main.localhost
```
### 🛠 Roadmap
- Automatic redeployment via webhooks (GitHub/GitLab).
- HTTPS support with auto SSL/TLS.
- Frontend dashboard for managing deployments.
- Auto-scaling containers based on load.


### 🤝 Contributing
Contributions, issues, and feature requests are welcome!
Feel free to fork the repo and open a PR.
### 📜 License
This project is licensed under the **MIT License**. See the LICENSE file for details.
###  🙌 Acknowledgements
- Rust
- Docker
- Nginx
- Inspiration from platforms like Heroku, Vercel, Netlify.
```yaml
---
Do you want me to also add **example Dockerfiles** (React, Rust, Python, etc.) in the README so people can see how deployments are built automatically?

````
