mod app; // 声明 app 模块

use app::App; // 导入名为 APP 的组件
use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! {
            <App/> // 使用与定义一致的大写名称
        }
    })
}