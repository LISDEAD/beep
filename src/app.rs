use js_sys::{eval, Reflect};
use leptos::prelude::*;
use std::f64::consts::PI;
use tauri::Runtime;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::window;

#[component]
pub fn App() -> impl IntoView {
    const TOTAL_SECONDS: i32 = 90 * 60; // 90分钟

    // Leptos信号
    let (remaining_seconds, set_remaining_seconds) = signal(TOTAL_SECONDS);
    let (is_running, set_is_running) = signal(false);

    // 格式化时间显示
    let formatted_time = move || {
        let mins = remaining_seconds.get() / 60;
        let secs = remaining_seconds.get() % 60;
        format!("{mins:02}:{secs:02}")
    };

    // 圆环进度计算
    let circle_dashoffset = move || {
        let circumference = 2.0 * PI * 100.0;
        circumference * (1.0 - remaining_seconds.get() as f64 / TOTAL_SECONDS as f64)
    };

    // 注册JS回调（供后端调用）
    Effect::new(move |_| {
        let window = window().expect("获取窗口失败");
        let set_remaining = set_remaining_seconds;
        let set_running = set_is_running;

        // 计时器更新回调
        let update_callback = Closure::wrap(Box::new(move |remaining: i32| {
            set_remaining.set(remaining);
        }) as Box<dyn Fn(i32)>);

        // 计时完成回调
        let complete_callback = Closure::wrap(Box::new(move || {
            set_running.set(false);
        }) as Box<dyn Fn()>);

        // 设置window属性 - 明确指定类型参数解决推断问题
        unsafe {
            // 将回调转换为JsValue，明确指定unchecked_ref的类型
            let update_js: JsValue = update_callback
                .as_ref()
                .unchecked_ref::<js_sys::Function>()
                .into();
            let _ = Reflect::set(
                &window,
                &JsValue::from_str("timerUpdateCallback"),
                &update_js,
            );

            let complete_js: JsValue = complete_callback
                .as_ref()
                .unchecked_ref::<js_sys::Function>()
                .into();
            let _ = Reflect::set(
                &window,
                &JsValue::from_str("timerCompleteCallback"),
                &complete_js,
            );
        }

        // 防止闭包被回收
        update_callback.forget();
        complete_callback.forget();
    });
    
    // 替换原来的 call_backend 实现
    let call_backend = |cmd: &str| {
        let js_code = format!("window.__TAURI__.invoke('{}')", cmd);
        let _ = eval(&js_code);
    };

    // 开始/暂停计时器
    let toggle_timer = move |_| {
        let running = is_running.get();
        set_is_running.set(!running);

        let cmd = if running {
            "pause_timer"
        } else {
            "start_timer"
        };
        call_backend(cmd);
    };

    // 重置计时器
    let reset_timer = move |_| {
        set_is_running.set(false);
        call_backend("reset_timer");
    };

    view! {
        <div class="container flex flex-col items-center justify-center min-h-screen">
            <div class="relative w-64 h-64 mb-8">
                {/* 圆环背景 */}
                <svg class="w-full h-full" viewBox="0 0 240 240">
                    <circle
                        cx="120" cy="120" r="100"
                        fill="none" stroke="#e6e6e6" stroke-width="10"
                    />
                    // 进度圆环
                    <circle
                        cx="120" cy="120" r="100"
                        fill="none" stroke="#3b82f6" stroke-width="10"
                        stroke-dasharray={move || format!("{}", 2.0 * PI * 100.0)}  // 改为信号绑定
                        stroke-dashoffset={circle_dashoffset}
                        transform="rotate(-90 120 120)"
                        class="transition-all duration-300 ease-in-out"
                    />
                </svg>
                {/* 时间显示 */}
                <div class="absolute inset-0 flex items-center justify-center text-4xl font-bold">
                    {formatted_time}
                </div>
            </div>

            <div class="flex gap-4">
                <button
                    on:click=toggle_timer
                    class="px-6 py-2 bg-blue-500 text-white rounded-full hover:bg-blue-600"
                >
                    {move || if is_running.get() { "暂停" } else { "开始" }}
                </button>
                <button
                    on:click=reset_timer
                    class="px-6 py-2 bg-gray-500 text-white rounded-full hover:bg-gray-600"
                >
                    "重置"
                </button>
            </div>
        </div>
    }
}
