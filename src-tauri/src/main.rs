// 添加必要的导入
use tauri::{Emitter, State, AppHandle};  // 关键：导入 Emitter trait
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

// 示例：修复 start_timer 命令
#[tauri::command]
fn start_timer(app: AppHandle, state: State<Arc<Mutex<TimerState>>>) {
    // 正确克隆 Arc（注意：state 是 State<Arc<Mutex<TimerState>>> 类型）
    let state_ptr = Arc::clone(&state.inner());  // 关键：通过 .inner() 获取 Arc 引用
    let app_clone = app.clone();

    thread::spawn(move || {
        loop {
            // 锁定状态（获取 MutexGuard）
            let mut current_state = state_ptr.lock().unwrap();
            
            if !current_state.is_running {
                break;
            }
            
            if current_state.remaining_seconds > 0 {
                current_state.remaining_seconds -= 1;
                // 发送更新事件（使用 emit_to，需要 Emitter trait）
                let _ = app_clone.emit_to(
                    "main",  // 窗口标签
                    "timer_update", 
                    current_state.remaining_seconds
                );
            } else {
                current_state.is_running = false;
                break;
            }
            
            // 释放锁，避免阻塞
            drop(current_state);
            thread::sleep(Duration::from_secs(1));
        }
    });
}

// 其他命令（pause_timer/reset_timer/set_total_seconds）采用相同模式修复
#[tauri::command]
fn pause_timer(state: State<Arc<Mutex<TimerState>>>) {
    let mut current_state = state.inner().lock().unwrap();
    current_state.is_running = false;
}

#[tauri::command]
fn reset_timer(state: State<Arc<Mutex<TimerState>>>) {
    let mut current_state = state.inner().lock().unwrap();
    current_state.remaining_seconds = current_state.total_seconds;
    current_state.is_running = false;
}

#[tauri::command]
fn set_total_seconds(state: State<Arc<Mutex<TimerState>>>, seconds: u32) {
    let mut current_state = state.inner().lock().unwrap();
    current_state.total_seconds = seconds;
    current_state.remaining_seconds = seconds;
}