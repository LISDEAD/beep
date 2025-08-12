// 添加必要的导入
use tauri::{Emitter, State, AppHandle};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// 定义计时器状态结构体
#[derive(Default)]
struct TimerState {
    remaining_seconds: u32,
    total_seconds: u32,
    is_running: bool,
}

fn main() {
    tauri::Builder::default()
        .manage(Arc::new(Mutex::new(TimerState::default())))  // 管理状态
        .invoke_handler(tauri::generate_handler![
            start_timer,
            pause_timer,
            reset_timer,
            set_total_seconds
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// 修复start_timer命令（核心功能修复）
#[tauri::command]
fn start_timer(app: AppHandle, state: State<Arc<Mutex<TimerState>>>) {
    let state_ptr = Arc::clone(&state.inner());
    let app_clone = app.clone();

    // 启动前先设置为运行状态（关键修复）
    {
        let mut current_state = state_ptr.lock().unwrap();
        current_state.is_running = true;
    }

    thread::spawn(move || {
        loop {
            let mut current_state = state_ptr.lock().unwrap();
            
            if !current_state.is_running {
                break;
            }
            
            if current_state.remaining_seconds > 0 {
                current_state.remaining_seconds -= 1;
                // 发送更新事件到前端
                let _ = app_clone.emit_to(
                    "main",
                    "timer_update", 
                    current_state.remaining_seconds
                );
            } else {
                current_state.is_running = false;
                break;
            }
            
            drop(current_state);  // 释放锁，避免阻塞
            thread::sleep(Duration::from_secs(1));
        }
    });
}

// 暂停计时器
#[tauri::command]
fn pause_timer(state: State<Arc<Mutex<TimerState>>>) {
    let mut current_state = state.inner().lock().unwrap();
    current_state.is_running = false;
}

// 重置计时器
#[tauri::command]
fn reset_timer(state: State<Arc<Mutex<TimerState>>>) {
    let mut current_state = state.inner().lock().unwrap();
    current_state.remaining_seconds = current_state.total_seconds;
    current_state.is_running = false;
}

// 设置总时间
#[tauri::command]
fn set_total_seconds(state: State<Arc<Mutex<TimerState>>>, seconds: u32) {
    let mut current_state = state.inner().lock().unwrap();
    current_state.total_seconds = seconds;
    current_state.remaining_seconds = seconds;
}