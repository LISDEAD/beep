use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// 修正导入语句，Tauri v2中run函数位于tauri::Builder
use tauri::{command, AppHandle, Emitter, Builder};

// 定义计时器状态结构体
#[derive(Default)]
struct TimerState {
    remaining_seconds: u32,
    total_seconds: u32,
    is_running: bool,
}

// 启动计时器
#[command]
fn start_timer(app: AppHandle, state: tauri::State<Arc<Mutex<TimerState>>>) -> Result<(), String> {
    let state_clone = Arc::clone(&state.inner());
    let app_clone = app.clone();

    let mut current_state = state_clone.lock().map_err(|e| format!("无法获取锁: {}", e))?;
    if current_state.is_running {
        return Ok(());
    }
    current_state.is_running = true;
    drop(current_state);

    thread::spawn(move || {
        loop {
            let mut current_state = match state_clone.lock() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("获取锁失败: {}", e);
                    break;
                }
            };
            
            if !current_state.is_running {
                break;
            }
            
            if current_state.remaining_seconds > 0 {
                current_state.remaining_seconds -= 1;
                let remaining = current_state.remaining_seconds;
                let _ = app_clone.emit_to("main", "timer_update", remaining);
            } else {
                current_state.is_running = false;
                let _ = trigger_notification();
                break;
            }
            
            drop(current_state);
            thread::sleep(Duration::from_secs(1));
        }
    });

    Ok(())
}

// 暂停计时器
#[command]
fn pause_timer(state: tauri::State<Arc<Mutex<TimerState>>>) -> Result<(), String> {
    let mut timer_state = state.inner().lock().map_err(|e| format!("无法获取锁: {}", e))?;
    timer_state.is_running = false;
    Ok(())
}

// 重置计时器
#[command]
fn reset_timer(state: tauri::State<Arc<Mutex<TimerState>>>) -> Result<(), String> {
    let mut timer_state = state.inner().lock().map_err(|e| format!("无法获取锁: {}", e))?;
    timer_state.remaining_seconds = timer_state.total_seconds;
    timer_state.is_running = false;
    Ok(())
}

// 设置总时间
#[command]
fn set_total_seconds(state: tauri::State<Arc<Mutex<TimerState>>>, seconds: u32) -> Result<(), String> {
    let mut timer_state = state.inner().lock().map_err(|e| format!("无法获取锁: {}", e))?;
    timer_state.total_seconds = seconds;
    timer_state.remaining_seconds = seconds;
    Ok(())
}

// 触发通知
#[command]
fn trigger_notification() -> Result<(), String> {
    #[cfg(windows)]
    {
        Command::new("powershell")
            .args(&[
                "-Command",
                "New-BurntToastNotification -Title '倒计时结束' -Text '设定的时间已结束！'",
            ])
            .status()
            .map_err(|e| format!("Windows 通知失败: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("osascript")
            .args(&[
                "-e",
                "display notification \"设定的时间已结束！\" with title \"倒计时结束\" sound name \"default\"",
            ])
            .status()
            .map_err(|e| format!("macOS 通知失败: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("notify-send")
            .args(&["倒计时结束", "设定的时间已结束！"])
            .status()
            .map_err(|e| format!("Linux 通知失败: {}", e))?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 使用Builder构建并运行应用，适应Tauri v2的API变化
    Builder::default()
        .manage(Arc::new(Mutex::new(TimerState::default())))
        .invoke_handler(tauri::generate_handler![
            start_timer,
            pause_timer,
            reset_timer,
            set_total_seconds,
            trigger_notification
        ])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
    