//! Volt — a fast AI coding agent.
//!
//! Crate root: wires up logging, app state, Tauri plugins and command handlers.

pub mod agent;
pub mod ai;
pub mod commands;
pub mod compaction;
pub mod config;
pub mod error;
pub mod mcp;
pub mod state;
pub mod tools;
pub mod updater;
pub mod workspace;

use crate::commands::plan::PlanStore;
use crate::state::AppState;
use tauri::Manager;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Entry point shared by the binary and (optionally) mobile targets.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_process::init())
        .manage(std::sync::Arc::new(AppState::new()))
        .manage(PlanStore::default())
        .invoke_handler(tauri::generate_handler![
            // workspace
            commands::workspace::open_project,
            commands::workspace::list_dir,
            commands::workspace::read_file,
            commands::workspace::current_project,
            // provider
            commands::provider::list_providers,
            commands::provider::save_provider,
            commands::provider::delete_provider,
            commands::provider::set_active_provider,
            commands::provider::test_connection,
            // chat / agent
            commands::chat::send_message,
            commands::chat::stop_generation,
            commands::chat::respond_approval,
            // session
            commands::session::list_sessions,
            commands::session::create_session,
            commands::session::load_session,
            commands::session::delete_session,
            // settings
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::app_info,
            commands::settings::context_tokens,
            // plan / canvas
            commands::plan::get_plan,
            commands::plan::set_plan,
            commands::plan::update_plan_step,
            // mcp
            commands::mcp::list_mcp_servers,
            commands::mcp::save_mcp_server,
            commands::mcp::delete_mcp_server,
            commands::mcp::connect_mcp_server,
            commands::mcp::connect_all_mcp,
            commands::mcp::disconnect_mcp_server,
            commands::mcp::list_mcp_tools,
            // plugins / skills
            commands::plugins::list_plugins,
            commands::plugins::save_plugin,
            commands::plugins::toggle_plugin,
            commands::plugins::delete_plugin,
            // updater
            commands::updater::check_update,
            commands::updater::download_update,
            commands::updater::install_update,
        ])
        .setup(|app| {
            tracing::info!("⚡ Volt starting up…");
            let handle = app.handle().clone();
            if let Err(e) = config::bootstrap(&handle) {
                tracing::error!("config bootstrap failed: {e}");
            }
            // Auto-connect enabled MCP servers in the background.
            let state = handle.state::<std::sync::Arc<AppState>>().inner().clone();
            tauri::async_runtime::spawn(async move {
                let servers = state.config.get().mcp_servers;
                for cfg in servers.into_iter().filter(|m| m.enabled) {
                    if let Err(e) = state.mcp.connect(&cfg) {
                        tracing::warn!("MCP '{}' failed to connect: {e}", cfg.name);
                    }
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Volt");
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("volt_lib=debug,info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();
}
