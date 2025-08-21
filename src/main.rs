// 声明所有模块
mod app;
mod tauri_utils;
mod timer_logic;
mod wasm_specific;
mod dummy_web_imports;

use app::App; // 导入 App 组件
use leptos::prelude::*;

fn main() {
    mount_to_body(|| {
        view! {
            <App/> // 使用与定义一致的大写名称
        }
    })
}