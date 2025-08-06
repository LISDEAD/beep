use leptos::prelude::*;
use leptos::mount::mount_to;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlElement, window as web_window, Document};

// 导入app模块及组件
pub mod app;
use app::APP;

use log::Level;

#[wasm_bindgen]
pub fn hydrate() {
    // 初始化日志
    _ = console_log::init_with_level(Level::Debug);
    console_error_panic_hook::set_once();

    // 获取浏览器窗口和文档
    let window = web_window().expect("浏览器窗口未找到");
    let document: Document = window.document().expect("文档对象未找到");

    // 获取根元素
    let root = document
        .get_element_by_id("app")
        .expect("app元素未找到")
        .dyn_into::<HtmlElement>()
        .expect("根元素必须是HtmlElement");

    // Leptos 0.8.6 最终挂载方案（闭包无参数）
    mount_to(root, || view! { <APP /> });
}

// 通知逻辑
#[tauri::command]
fn show_notification() -> Result<(), String> {
    let args = [
        "-Command",
        "New-BurntToastNotification -Title '倒计时结束' -Text '90分钟已结束！'",
    ];
    match std::process::Command::new("powershell")
        .args(&args)
        .status()
    {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("通知发送失败: {}", e)),
    }
}