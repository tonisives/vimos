mod config;
pub mod ipc;
mod keyboard;
mod vim;
mod window;

use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, State};

use config::Settings;
use ipc::{IpcCommand, IpcResponse};
use keyboard::{check_accessibility_permission, request_accessibility_permission, KeyboardCapture};
use vim::{ProcessResult, VimMode, VimState};
use window::setup_indicator_window;

/// Application state shared across commands
pub struct AppState {
    settings: Mutex<Settings>,
    vim_state: Arc<Mutex<VimState>>,
    keyboard_capture: KeyboardCapture,
}

/// Check if accessibility permission is granted
#[tauri::command]
fn check_permission() -> bool {
    check_accessibility_permission()
}

/// Request accessibility permission (shows system dialog)
#[tauri::command]
fn request_permission() -> bool {
    request_accessibility_permission()
}

/// Get current vim mode
#[tauri::command]
fn get_vim_mode(state: State<AppState>) -> String {
    let vim_state = state.vim_state.lock().unwrap();
    vim_state.mode().as_str().to_string()
}

/// Get current settings
#[tauri::command]
fn get_settings(state: State<AppState>) -> Settings {
    let settings = state.settings.lock().unwrap();
    settings.clone()
}

/// Update settings
#[tauri::command]
fn set_settings(state: State<AppState>, new_settings: Settings) -> Result<(), String> {
    let mut settings = state.settings.lock().unwrap();
    *settings = new_settings;
    settings.save()
}

/// Start keyboard capture
#[tauri::command]
fn start_capture(state: State<AppState>) -> Result<(), String> {
    state.keyboard_capture.start()
}

/// Stop keyboard capture
#[tauri::command]
fn stop_capture(state: State<AppState>) {
    state.keyboard_capture.stop()
}

/// Check if keyboard capture is running
#[tauri::command]
fn is_capture_running(state: State<AppState>) -> bool {
    state.keyboard_capture.is_running()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    // Create vim state and mode receiver
    let (vim_state, mode_rx) = VimState::new();
    let vim_state = Arc::new(Mutex::new(vim_state));
    let vim_state_for_callback = Arc::clone(&vim_state);

    // Create keyboard capture
    let keyboard_capture = KeyboardCapture::new();

    // Set up the callback
    keyboard_capture.set_callback(move |event| {
        let mut state = vim_state_for_callback.lock().unwrap();
        match state.process_key(event) {
            ProcessResult::Suppress => None,
            ProcessResult::PassThrough => Some(event),
            ProcessResult::ModeChanged(_mode) => None,
        }
    });

    // Load settings
    let settings = Settings::load();

    // Create app state with shared vim state
    let app_state = AppState {
        settings: Mutex::new(settings),
        vim_state: Arc::clone(&vim_state),
        keyboard_capture,
    };

    // Store mode receiver for setup
    let mode_rx = Arc::new(Mutex::new(mode_rx));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            check_permission,
            request_permission,
            get_vim_mode,
            get_settings,
            set_settings,
            start_capture,
            stop_capture,
            is_capture_running,
        ])
        .setup(move |app| {
            // Get the indicator window and configure it
            if let Some(indicator_window) = app.get_webview_window("indicator") {
                if let Err(e) = setup_indicator_window(&indicator_window) {
                    log::error!("Failed to setup indicator window: {}", e);
                }
            }

            // Set up mode change event emission
            let app_handle = app.handle().clone();
            let mut rx = mode_rx.lock().unwrap().resubscribe();

            // Spawn task to forward mode changes to frontend
            tauri::async_runtime::spawn(async move {
                while let Ok(mode) = rx.recv().await {
                    log::info!("Mode changed to: {:?}", mode);
                    let _ = app_handle.emit("mode-change", mode.as_str());
                }
            });

            // Auto-start keyboard capture if permission is granted
            if check_accessibility_permission() {
                let state: State<AppState> = app.state();
                if let Err(e) = state.keyboard_capture.start() {
                    log::error!("Failed to start keyboard capture: {}", e);
                } else {
                    log::info!("Keyboard capture started automatically");
                }
            } else {
                log::warn!("Accessibility permission not granted, requesting...");
                // Request permission - this will show the system dialog
                request_accessibility_permission();
            }

            // Start IPC server for CLI control
            let vim_state_for_ipc = Arc::clone(&vim_state);
            let app_handle_for_ipc = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let handler = move |cmd: IpcCommand| -> IpcResponse {
                    let mut state = vim_state_for_ipc.lock().unwrap();
                    match cmd {
                        IpcCommand::GetMode => {
                            IpcResponse::Mode(state.mode().as_str().to_string())
                        }
                        IpcCommand::Toggle => {
                            let new_mode = state.toggle_mode();
                            let _ = app_handle_for_ipc.emit("mode-change", new_mode.as_str());
                            IpcResponse::Mode(new_mode.as_str().to_string())
                        }
                        IpcCommand::Insert => {
                            state.set_mode_external(VimMode::Insert);
                            let _ = app_handle_for_ipc.emit("mode-change", "insert");
                            IpcResponse::Ok
                        }
                        IpcCommand::Normal => {
                            state.set_mode_external(VimMode::Normal);
                            let _ = app_handle_for_ipc.emit("mode-change", "normal");
                            IpcResponse::Ok
                        }
                        IpcCommand::Visual => {
                            state.set_mode_external(VimMode::Visual);
                            let _ = app_handle_for_ipc.emit("mode-change", "visual");
                            IpcResponse::Ok
                        }
                        IpcCommand::SetMode(mode_str) => {
                            match mode_str.to_lowercase().as_str() {
                                "insert" | "i" => {
                                    state.set_mode_external(VimMode::Insert);
                                    let _ = app_handle_for_ipc.emit("mode-change", "insert");
                                    IpcResponse::Ok
                                }
                                "normal" | "n" => {
                                    state.set_mode_external(VimMode::Normal);
                                    let _ = app_handle_for_ipc.emit("mode-change", "normal");
                                    IpcResponse::Ok
                                }
                                "visual" | "v" => {
                                    state.set_mode_external(VimMode::Visual);
                                    let _ = app_handle_for_ipc.emit("mode-change", "visual");
                                    IpcResponse::Ok
                                }
                                _ => IpcResponse::Error(format!("Unknown mode: {}", mode_str)),
                            }
                        }
                    }
                };

                if let Err(e) = ipc::start_ipc_server(handler).await {
                    log::error!("IPC server error: {}", e);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
