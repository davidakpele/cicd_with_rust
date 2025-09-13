use std::sync::Arc;
use axum::{
    middleware::{self},
    response::{Html, IntoResponse},
    routing::{delete, get, post, put},
    Extension, Json, Router,
    http::StatusCode,
};
use serde_json::json;
use sqlx::PgPool;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa::path;
use serde::Serialize;

use crate::{controllers::{deployment_controller::deploy_project, github_account_controller::{delete_repo_by_id, fetch_repo_by_id, fetch_user_by_username, get_user_account_by_id, list_branches, list_repos_from_git}}, ws::ws_channel::WsBroadcaster};
use crate::controllers::{
    auth_controller::{login_user, register_user},
    user_controller::{delete_user, get_user_by_id, update_user},
};
use crate::middleware::auth::{AuthUser, AdminUser};
use crate::controllers::user_controller::{ApiResponse, UpdateUserRequest};
use crate::controllers::repo_controller::{delete_repo, list_repos, get_repo, create_repo};
use crate::controllers::job_controller::{update_job_status, get_job, create_job};
use crate::responses::responses::{SafeUser};
use crate::bootstrap::github_auth_grant::{github_login, github_callback, AppState};
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use crate::config::settings::Settings;


struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("x-api-key"))),
            )
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
struct ApiErrorSchema;
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::controllers::job_controller::update_job_status, 
        crate::controllers::job_controller::get_job, 
        crate::controllers::job_controller::create_job,
        crate::controllers::repo_controller::list_repos, 
        crate::controllers::repo_controller::get_repo, 
        crate::controllers::repo_controller::create_repo,
        crate::controllers::repo_controller::delete_repo,
        crate::controllers::auth_controller::login_user,
        crate::controllers::auth_controller::register_user,
        crate::controllers::user_controller::get_user_by_id,
        crate::controllers::user_controller::update_user,
        crate::controllers::user_controller::delete_user,
    ),
    components(
        schemas(
            UpdateUserRequest,
            SafeUser,
            ApiResponse<SafeUser>,
            ApiResponse<ApiErrorSchema>
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "User", description = "User management API"),
        (name = "Auth", description = "Authentication and registration API")
    )
)]
struct ApiDoc;

// Handler to serve the Swagger UI HTML
async fn swagger_ui_handler() -> Html<String> {
    const SWAGGER_UI_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <title>Swagger UI</title>
  <link rel="stylesheet" type="text/css" href="https://cdnjs.cloudflare.com/ajax/libs/swagger-ui/4.15.5/swagger-ui.min.css" />
  <style>
    body { margin: 0; padding: 0; }
  </style>
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://cdnjs.cloudflare.com/ajax/libs/swagger-ui/4.15.5/swagger-ui-bundle.min.js"></script>
  <script>
    window.onload = function() {
      // Begin Swagger UI call
      window.ui = SwaggerUIBundle({
        url: "/api-docs/openapi.json",
        dom_id: '#swagger-ui',
        deepLinking: true,
        presets: [
          SwaggerUIBundle.presets.apis,
          SwaggerUIBundle.SwaggerUIStandalonePreset
        ],
        plugins: [
          SwaggerUIBundle.plugins.DownloadUrl
        ],
      });
    };
  </script>
</body>
</html>
"#;
    Html(SWAGGER_UI_HTML.to_string())
}

async fn api_doc_handler() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

async fn handler_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "error": "Not Found",
            "message": "The requested resource was not found"
        })),
    )
}

pub fn create_routes(pool: PgPool, broadcaster: Arc<WsBroadcaster>) -> Router {
    let _ = broadcaster;

    // Load GitHub OAuth config
    let settings = Settings::new();
    let oauth_client = BasicClient::new(
        ClientId::new(settings.git_client_id.clone()),
        Some(ClientSecret::new(settings.git_client_secret.clone())),
        AuthUrl::new("https://github.com/login/oauth/authorize".to_string()).unwrap(),
        Some(TokenUrl::new("https://github.com/login/oauth/access_token".to_string()).unwrap()),
    )
    .set_redirect_uri(
        RedirectUrl::new("http://localhost:5173/auth/github/callback".to_string()).unwrap(),
    );

    let state = Arc::new(AppState { oauth_client, db: pool.clone() });

     // GitHub OAuth routes
    let github_routes = Router::new()
        // authenticate user to login and authorize our platform to access user githhub account
        .route("/auth/github/login", get(github_login))
        // callback after authorize
        .route("/auth/github/callback", get(github_callback))
        .route("/github/repos/list/:id", get(get_user_account_by_id))
        // Fetch user profile from system
        .route("/user/profile", get(fetch_user_by_username))
        // fetch list of user repositories from github
        .route("/repos/:username/:repo/branches", get(list_branches))
        .layer(middleware::from_extractor::<AuthUser>())
        .layer(Extension(state.clone()));


    // Auth
    let auth_routes = Router::new()
        .route("/auth/register", post(register_user))
        .route("/auth/login", post(login_user))
        .layer(Extension(pool.clone()));

    // Users
    let user_routes = Router::new()
        .route("/users/:id", get(get_user_by_id))
        .route("/users/:id", put(update_user))
        .route("/users/:id", delete(delete_user))
        .layer(middleware::from_extractor::<AuthUser>())
        .layer(Extension(pool.clone()));

    
    // Jobs
     let job_routes = Router::new()
        .route("/jobs", post(create_job))
        .route("/jobs/:id", get(get_job))
        .route("/jobs/:id/status", put(update_job_status))
        .layer(middleware::from_extractor::<AuthUser>())
        .layer(Extension(pool.clone()));

    // Guthub Repository routes 
    let user_repo_routes = Router::new()
        .route("/github/:username/repos", get(list_repos_from_git))
        .route("/github/repos/:id", get(fetch_repo_by_id))
        .route("/github/repos/:id", delete(delete_repo_by_id))
       
        
        // .layer(middleware::from_extractor::<AuthUser>())
        .layer(Extension(pool.clone()));
    
    // Deployment 
    let user_project_deploy_routes = Router::new()
        .route("/deploy/project", post(deploy_project))
        .layer(middleware::from_extractor::<AuthUser>())
        .layer(Extension(pool.clone()));

     // Repos
    let repo_routes = Router::new()
        .route("/repos", post(create_repo).get(list_repos))
        .route("/repos/:id", get(get_repo).delete(delete_repo))
        .layer(middleware::from_extractor::<AuthUser>())
        .layer(Extension(pool.clone()));

    // Admin 
    let admin_routes = Router::new()
        .route("/admin/secret", get(|| async { "Admin Only" }))
        .layer(middleware::from_extractor::<AdminUser>())
        .layer(Extension(pool.clone()));

     Router::new()
        .route("/swagger-ui", get(swagger_ui_handler))
        .route("/api-docs/openapi.json", get(api_doc_handler))
        .merge(auth_routes)
        .merge(job_routes)
        .merge(repo_routes)
        .merge(user_routes)
        .merge(admin_routes)
        .merge(github_routes)   
        .merge(user_project_deploy_routes)
        .merge(user_repo_routes)   
        .fallback(handler_404)
        .layer(Extension(pool))
        .layer(Extension(broadcaster))
}
