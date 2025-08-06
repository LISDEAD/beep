use once_cell::sync::OnceCell;
use std::sync::Mutex;
use tauri::{Emitter, Manager, Runtime, WebviewWindow};
use tokio::time::{interval, Duration};


#[derive(Debug)]
struct AppState {
    remaining_seconds: i32,
    total_seconds: i32,
    timer_handle: Option<tokio::task::JoinHandle<()>>,
}

impl AppState {
    fn new(total_seconds: i32) -> Self {
        AppState {
            remaining_seconds: total_seconds,
            total_seconds,
            timer_handle: None,
        }
    }
}

static STATE: OnceCell<Mutex<AppState>> = OnceCell::new();

fn send_update_event<R: Runtime>(window: &WebviewWindow<R>, remaining: i32) {
    let _ = window.emit("timer_update", remaining);
}

fn send_complete_event<R: Runtime>(window: &WebviewWindow<R>) {
    let _ = window.emit("timer_complete", ());
}

fn main() {
    let total_seconds = 90 * 60;
    STATE.set(Mutex::new(AppState::new(total_seconds))).unwrap();

    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_webview_window("main").expect("主窗口不存在");
            let state = STATE.get().unwrap().lock().unwrap();
            send_update_event(&window, state.remaining_seconds);
            drop(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_timer,
            pause_timer,
            reset_timer
        ])
        .run(tauri::generate_context!())
        .expect("运行Tauri应用时出错");
}

#[tauri::command]
fn start_timer<R: Runtime>(window: WebviewWindow<R>) {
    let mut state = STATE.get().unwrap().lock().unwrap();
    if state.timer_handle.is_some() {
        return;
    }
    
    let remaining = state.remaining_seconds;
    let handle = tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(1));
        let mut remaining = remaining;
        
        loop {
            interval.tick().await;
            
            if remaining <= 0 {
                send_complete_event(&window);
                break;
            }
            
            remaining -= 1;
            send_update_event(&window, remaining);
            
            let mut state = STATE.get().unwrap().lock().unwrap();
            state.remaining_seconds = remaining;
            
            if remaining <= 0 {
                state.timer_handle.take();
                break;
            }
        }
    });
    
    state.timer_handle = Some(handle);
}

#[tauri::command]
fn pause_timer() {
    let mut state = STATE.get().unwrap().lock().unwrap();
    if let Some(handle) = state.timer_handle.take() {
        handle.abort();
    }
}

#[tauri::command]
fn reset_timer<R: Runtime>(window: WebviewWindow<R>) {
    let mut state = STATE.get().unwrap().lock().unwrap();
    if let Some(handle) = state.timer_handle.take() {
        handle.abort();
    }
    state.remaining_seconds = state.total_seconds;
    send_update_event(&window, state.remaining_seconds);
}