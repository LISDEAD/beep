mod app; // 声明 app 模块

use app::App; // 导入名为 APP 的组件
use leptos::prelude::*;

fn main() {
    mount_to_body(|| {
        view! {
            <App/> // 使用与定义一致的大写名称
        }
    })
}