// API imports grouped by module
use crate::api::{
    admin_user_api::AdminUserApi,
    asset_api::AssetApi,
    auth_api::AuthApi,
    cms_api::CmsApi,
    content_api::ContentApi,
    dashboard_api::DashboardApi,
    general_api::GeneralApi,
    handlers::asset::store_asset_api_handler::store_asset_api_handler,
    misc_api::MiscApi,
    setting_api::SettingApi,
};

// Proto server imports grouped by module
use crate::api::proto::{
    admin_user::admin_user_server::AdminUserServer,
    asset::asset_server::AssetServer,
    auth::auth_server::AuthServer,
    cms::cms_server::CmsServer,
    content::content_server::ContentServer,
    dashboard::dashboard_server::DashboardServer,
    general::general_service_server::GeneralServiceServer,
    misc::misc_server::MiscServer,
    setting::setting_server::SettingServer,
};
// Core application imports
use crate::{
    avored_state::AvoRedState,
    error::Error,
    middleware::{
        grpc_auth_middleware::check_auth,
        require_jwt_authentication::require_jwt_authentication,
        security_headers::add_security_headers,
    },
};
// External crate imports grouped by functionality
use axum::{
    http::HeaderValue,
    response::Html,
    routing::{get, post},
    Router,
};
use axum_tonic::{NestTonic, RestGrpcService};
use std::{env, fs::File, path::Path, sync::Arc};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt, Layer};

mod api;
mod avored_state;
mod error;
mod extensions;
mod middleware;
mod models;
mod providers;
mod repositories;
mod requests;
mod services;

const PER_PAGE: u64 = 10;

rust_i18n::i18n!("resources/locales");

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, AvoRed content management system!</h1>")
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    init_log();

    let state = Arc::new(AvoRedState::new().await?);

    let mut origins: Vec<HeaderValue> = vec![];
    for origin in &state.config.cors_allowed_app_url {
        origins.push(HeaderValue::from_str(origin).unwrap());
    }

    // const DEFAULT_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);
    // const DEFAULT_EXPOSED_HEADERS: [&str; 3] =
    //     ["grpc-status", "grpc-message", "grpc-status-details-bin"];
    // const DEFAULT_ALLOW_HEADERS: [&str; 5] =
    //     ["x-grpc-web", "content-type", "x-user-agent", "grpc-timeout", "authorization"];

    let cors = CorsLayer::new()
        .allow_origin(Any) // Allow all origins for local development
        .allow_headers(Any) // Allow all headers
        .allow_methods(Any) // Allow all methods
        .expose_headers(Any); // Expose all headers

    let misc_api = MiscApi {
        state: state.clone(),
    };
    let misc_server = MiscServer::new(misc_api);

    let cms_api = CmsApi {
        state: state.clone(),
    };
    let cms_server = CmsServer::new(cms_api);

    let dashboard_api = DashboardApi {
        state: state.clone(),
    };
    let dashboard_server = DashboardServer::with_interceptor(dashboard_api, check_auth);

    let auth_api = AuthApi {
        state: state.clone(),
    };
    let auth_server = AuthServer::new(auth_api);

    let admin_user_api = AdminUserApi {
        state: state.clone(),
    };
    let admin_user_server = AdminUserServer::with_interceptor(admin_user_api, check_auth);

    let content_api = ContentApi {
        state: state.clone(),
    };
    let content_server = ContentServer::with_interceptor(content_api, check_auth);

    let setting_api = SettingApi {
        state: state.clone(),
    };
    let setting_server = SettingServer::with_interceptor(setting_api, check_auth);

    let general_api = GeneralApi {
        state: state.clone(),
    };
    let general_server = GeneralServiceServer::with_interceptor(general_api, check_auth);

    let asset_api = AssetApi {
        state: state.clone(),
    };
    let asset_server = AssetServer::with_interceptor(asset_api, check_auth);

    let grpc_router = Router::new()
        .nest_tonic(misc_server)
        .nest_tonic(auth_server)
        .nest_tonic(dashboard_server)
        .nest_tonic(admin_user_server)
        .nest_tonic(content_server)
        .nest_tonic(setting_server)
        .nest_tonic(general_server)
        .nest_tonic(asset_server)
        .nest_tonic(cms_server)
        .layer(cors.clone())
        .layer(axum::middleware::from_fn(add_security_headers));

    let static_routing_service = ServeDir::new("public");

    let rest_router = Router::new()
        .route("/", get(handler))
        .route("/api/asset", post(store_asset_api_handler))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            require_jwt_authentication,
        ))
        .nest_service("/public", static_routing_service)
        .with_state(state)
        .layer(cors)
        .layer(axum::middleware::from_fn(add_security_headers));

    let service = RestGrpcService::new(rest_router, grpc_router);

    let port = env::var("PORT").unwrap_or("50051".to_string());

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .unwrap();

    println!(r"     _             ____          _ ");
    println!(r"    / \__   _____ |  _ \ ___  __| |");
    println!(r"   / _ \ \ / / _ \| |_) / _ \/ _` |");
    println!(r"  / ___ \ V / (_) |  _ <  __/ (_| |");
    println!(r" /_/   \_\_/ \___/|_| \_\___|\__,_|");

    println!();
    println!();

    println!("Server started: http://0.0.0.0:{}", port);

    axum::serve(listener, service.into_make_service())
        .await
        .unwrap();

    Ok(())
}

fn init_log() {
    let stdout_log = tracing_subscriber::fmt::layer().pretty();

    // A layer that logs events to a file.
    let file = File::create(Path::new("public").join("log").join("avored.log"));
    let file = match file {
        Ok(file) => file,
        Err(error) => panic!("Error: {:?}", error),
    };
    let debug_log = tracing_subscriber::fmt::layer().with_writer(Arc::new(file));

    // A layer that collects metrics using specific events.
    let metrics_layer = /* ... */ filter::LevelFilter::INFO;

    tracing_subscriber::registry()
        .with(
            stdout_log
                // Add an `INFO` filter to the stdout logging layer
                .with_filter(filter::LevelFilter::INFO)
                // Combine the filtered `stdout_log` layer with the
                // `debug_log` layer, producing a new `Layered` layer.
                .and_then(debug_log)
                // Add a filter to *both* layers that rejects spans and
                // events whose targets start with `metrics`.
                .with_filter(filter::filter_fn(|metadata| {
                    !metadata.target().starts_with("metrics")
                })),
        )
        .with(
            // Add a filter to the metrics label that *only* enables
            // events whose targets start with `metrics`.
            metrics_layer.with_filter(filter::filter_fn(|metadata| {
                metadata.target().starts_with("metrics")
            })),
        )
        .init();
}
