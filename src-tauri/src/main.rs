use once_cell::sync::OnceCell;
use std::sync::Mutex;
use tauri::{Manager, Runtime, WebviewWindow};
use tokio::time::{interval, Duration};

#[derive(Debug)]
struct AppState {
    remaining_seconds: i32,
    total_seconds: i32,
    timer_handle: Option<tokio::task::JoinHandle<()>>,
}

impl AppState {
    fn new(total_seconds: i32) -> Self {
        Self {
            remaining_seconds: total_seconds,
            total_seconds,
            timer_handle: None,
        }
    }
}

static STATE: OnceCell<Mutex<AppState>> = OnceCell::new();

// 调用前端回调（Tauri 2.7.0 兼容方式）
fn call_js_callback<R: Runtime>(window: &WebviewWindow<R>, callback_name: &str, value: i32) {
    // 直接通过 eval 调用前端挂载在 window 上的回调
    let _ = window.eval(&format!(
        "if (window.{0}) {{ window.{0}({1}); }}",
        callback_name, value
    ));
}

fn main() {
    let total_seconds = 90 * 60;
    STATE.set(Mutex::new(AppState::new(total_seconds))).unwrap();

    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_webview_window("main").expect("主窗口不存在");
            let state = STATE.get().unwrap().lock().unwrap();
            // 初始化时同步时间到前端
            call_js_callback(&window, "timerUpdateCallback", state.remaining_seconds);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![start_timer, pause_timer, reset_timer])
        .run(tauri::generate_context!())
        .expect("运行Tauri应用失败");
}

#[tauri::command]
fn start_timer<R: Runtime>(window: WebviewWindow<R>) {
    let mut state = STATE.get().unwrap().lock().unwrap();
    if state.timer_handle.is_some() {
        return; // 避免重复启动
    }
    
    let remaining = state.remaining_seconds;
    let window_clone = window.clone();
    
    // 启动计时器任务
    state.timer_handle = Some(tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(1));
        let mut remaining = remaining;
        
        loop {
            interval.tick().await;
            remaining -= 1;
            
            // 通知前端更新时间
            call_js_callback(&window_clone, "timerUpdateCallback", remaining);
            STATE.get().unwrap().lock().unwrap().remaining_seconds = remaining;
            
            if remaining <= 0 {
                // 计时结束
                call_js_callback(&window_clone, "timerCompleteCallback", 0);
                STATE.get().unwrap().lock().unwrap().timer_handle.take();
                break;
            }
        }
    }));
}

#[tauri::command]
fn pause_timer() {
    let mut state = STATE.get().unwrap().lock().unwrap();
    if let Some(handle) = state.timer_handle.take() {
        handle.abort(); // 终止计时器任务
    }
}

#[tauri::command]
fn reset_timer<R: Runtime>(window: WebviewWindow<R>) {
    let mut state = STATE.get().unwrap().lock().unwrap();
    if let Some(handle) = state.timer_handle.take() {
        handle.abort(); // 终止现有任务
    }
    // 重置时间
    state.remaining_seconds = state.total_seconds;
    call_js_callback(&window, "timerUpdateCallback", state.remaining_seconds);
}