use std::process::Command;
use tauri::command;
#[command]
fn trigger_notification() -> Result<(), String> {
    // 根据操作系统调用对应的通知命令
    #[cfg(windows)]
    {
        // Windows 使用 PowerShell 的 BurntToast 模块
        Command::new("powershell")
            .args(&[
                "-Command",
                "New-BurntToastNotification -Title ' 倒计时结束 ' -Text '90 分钟已结束！'",
            ])
            .status()
            .map_err(|e| format!("Windows 通知失败: {}", e))?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![trigger_notification])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
