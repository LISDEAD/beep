use leptos::prelude::*;
use app::App;

mod app;

pub fn main() {
    // 新版本挂载方式：不需要传递 cx 给组件
    mount_to_body(App);
}
    