use wasm_bindgen::prelude::*;  // 正确导入wasm_bindgen宏
use leptos::mount::mount_to_body;  // 正确导入挂载函数
use app::App;

mod app;

#[wasm_bindgen]
pub fn run() {
    // 初始化Leptos应用
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    
    // 挂载应用到body
    mount_to_body(App);
}
