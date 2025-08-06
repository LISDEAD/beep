use leptos::prelude::*;
use serde::Deserialize;
use serde_wasm_bindgen::from_value;
use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{window, console, MouseEvent}; // 导入MouseEvent类型（关键修复）
use js_sys::Promise;
use std::f64::consts::PI;

// 定义计时器更新事件的数据结构
#[derive(Debug, Deserialize)]
struct TimerUpdate {
    remaining: i32,
}

// 组件定义：适配 Leptos 0.8.6 的组件宏
#[component]
pub fn APP() -> impl IntoView {
    let total_seconds = 90 * 60;
    
    // 使用推荐的 signal()
    let (remaining_seconds, set_remaining_seconds) = signal(total_seconds);
    let (is_running, set_is_running) = signal(false);
    
    // 格式化时间显示
    let formatted_time = move || {
        let secs = remaining_seconds.get();
        let mins = secs / 60;
        let secs = secs % 60;
        format!("{mins:02}:{secs:02}")
    };
    
    // 计算圆环进度
    let circle_progress = move || {
        100.0 - (remaining_seconds.get() as f64 / total_seconds as f64) * 100.0
    };
    
    // 计算圆环偏移量
    let circle_dashoffset = move || {
        let circumference = 2.0 * PI * 100.0;
        circumference - (circle_progress() / 100.0) * circumference
    };
    
    // 开始/暂停计时器（使用MouseEvent类型参数，与click事件匹配）
    let toggle_timer = move |_e: MouseEvent| {  // 关键修复：使用MouseEvent
        let is_running_val = is_running.get();
        set_is_running.set(!is_running_val);
        
        spawn_local(async move {
            if let Some(window) = window() {
                if let Some(tauri) = window.get("__TAURI__") {
                    let tauri = JsValue::from(tauri);
                    if let Ok(invoke) = js_sys::Reflect::get(&tauri, &JsValue::from_str("invoke")) {
                        let cmd = if is_running_val {
                            JsValue::from_str("pause_timer")
                        } else {
                            JsValue::from_str("start_timer")
                        };
                        if let Ok(invoke_fn) = invoke.dyn_into::<js_sys::Function>() {
                            let call_result = invoke_fn.call1(&JsValue::NULL, &cmd);
                            if let Ok(promise) = call_result.and_then(|v| v.dyn_into::<Promise>()) {
                                let _ = JsFuture::from(promise).await;
                            }
                        }
                    }
                }
            }
        });
    };
    
    // 重置计时器（使用MouseEvent类型参数）
    let reset_timer = move |_e: MouseEvent| {  // 关键修复：使用MouseEvent
        set_is_running.set(false);
        spawn_local(async move {
            if let Some(window) = window() {
                if let Some(tauri) = window.get("__TAURI__") {
                    let tauri = JsValue::from(tauri);
                    if let Ok(invoke) = js_sys::Reflect::get(&tauri, &JsValue::from_str("invoke")) {
                        let cmd = JsValue::from_str("reset_timer");
                        if let Ok(invoke_fn) = invoke.dyn_into::<js_sys::Function>() {
                            let call_result = invoke_fn.call1(&JsValue::NULL, &cmd);
                            if let Ok(promise) = call_result.and_then(|v| v.dyn_into::<Promise>()) {
                                let _ = JsFuture::from(promise).await;
                            }
                        }
                    }
                }
            }
        });
    };
    
    // 监听计时器更新事件
    Effect::new(move |_| {
        spawn_local(async move {
            if let Some(window) = window() {
                if let Some(tauri) = window.get("__TAURI__") {
                    let tauri = JsValue::from(tauri);
                    if let Ok(event) = js_sys::Reflect::get(&tauri, &JsValue::from_str("event")) {
                        if let Ok(listen) = js_sys::Reflect::get(&event, &JsValue::from_str("listen")) {
                            let event_name = JsValue::from_str("timer_update");
                            if let Ok(listen_fn) = listen.dyn_into::<js_sys::Function>() {
                                let call_result = listen_fn.call1(&JsValue::NULL, &event_name);
                                if let Ok(promise) = call_result.and_then(|v| v.dyn_into::<Promise>()) {
                                    match JsFuture::from(promise).await {
                                        Ok(listener) => {
                                            if let Ok(listener) = listener.dyn_into::<js_sys::AsyncIterator>() {
                                                loop {
                                                    let next_promise = match listener.next() {
                                                        Ok(p) => p,
                                                        Err(e) => {
                                                            console::error_1(&JsValue::from(format!("迭代器错误: {e:?}")));
                                                            break;
                                                        }
                                                    };
                                                    let next_result = match JsFuture::from(next_promise).await {
                                                        Ok(v) => v,
                                                        Err(e) => {
                                                            console::error_1(&JsValue::from(format!("等待结果错误: {e:?}")));
                                                            break;
                                                        }
                                                    };
                                                    let done = js_sys::Reflect::get(&next_result, &JsValue::from_str("done"))
                                                        .and_then(|d| d.as_bool().ok_or_else(|| JsValue::from_str("done不是bool")))
                                                        .unwrap_or(true);
                                                    if done { break; }
                                                    
                                                    let value = match js_sys::Reflect::get(&next_result, &JsValue::from_str("value")) {
                                                        Ok(v) => v,
                                                        Err(e) => {
                                                            console::error_1(&JsValue::from(format!("获取值错误: {e:?}")));
                                                            continue;
                                                        }
                                                    };
                                                    if let Ok(update) = from_value::<TimerUpdate>(value) {
                                                        set_remaining_seconds.set(update.remaining);
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => console::error_1(&JsValue::from(format!("监听失败: {e:?}"))),
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    });
    
    // 监听计时器完成事件
    Effect::new(move |_| {
        spawn_local(async move {
            if let Some(window) = window() {
                if let Some(tauri) = window.get("__TAURI__") {
                    let tauri = JsValue::from(tauri);
                    if let Ok(event) = js_sys::Reflect::get(&tauri, &JsValue::from_str("event")) {
                        if let Ok(listen) = js_sys::Reflect::get(&event, &JsValue::from_str("listen")) {
                            let event_name = JsValue::from_str("timer_complete");
                            if let Ok(listen_fn) = listen.dyn_into::<js_sys::Function>() {
                                let call_result = listen_fn.call1(&JsValue::NULL, &event_name);
                                if let Ok(promise) = call_result.and_then(|v| v.dyn_into::<Promise>()) {
                                    match JsFuture::from(promise).await {
                                        Ok(listener) => {
                                            if let Ok(listener) = listener.dyn_into::<js_sys::AsyncIterator>() {
                                                loop {
                                                    let next_promise = match listener.next() {
                                                        Ok(p) => p,
                                                        Err(e) => {
                                                            console::error_1(&JsValue::from(format!("迭代器错误: {e:?}")));
                                                            break;
                                                        }
                                                    };
                                                    let next_result = match JsFuture::from(next_promise).await {
                                                        Ok(v) => v,
                                                        Err(e) => {
                                                            console::error_1(&JsValue::from(format!("等待结果错误: {e:?}")));
                                                            break;
                                                        }
                                                    };
                                                    let done = js_sys::Reflect::get(&next_result, &JsValue::from_str("done"))
                                                        .and_then(|d| d.as_bool().ok_or_else(|| JsValue::from_str("done不是bool")))
                                                        .unwrap_or(true);
                                                    if done { break; }
                                                    
                                                    set_is_running.set(false);
                                                    set_remaining_seconds.set(0);
                                                }
                                            }
                                        }
                                        Err(e) => console::error_1(&JsValue::from(format!("监听失败: {e:?}"))),
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    });
    
    // 视图宏：确保元素结构正确，事件处理函数匹配
    view! {
        <div class="flex flex-col items-center justify-center min-h-screen bg-gray-50">
            <div class="relative w-64 h-64 md:w-80 md:h-80">
                <svg class="w-full h-full" viewBox="0 0 240 240">
                    <circle 
                        cx="120" cy="120" r="100" 
                        fill="none" stroke="#e6e6e6" stroke-width="10"
                    />
                    <circle 
                        cx="120" cy="120" r="100" 
                        fill="none" 
                        stroke={if circle_progress() < 70.0 { "#4ade80" } else if circle_progress() < 90.0 { "#facc15" } else { "#ef4444" }} 
                        stroke-width="10" stroke-linecap="round"
                        transform="rotate(-90 120 120)"
                        stroke-dasharray="628.3185307"
                        stroke-dashoffset=circle_dashoffset
                        class="transition-all duration-1000 ease-in-out"
                    />
                </svg>
                <div class="absolute inset-0 flex flex-col items-center justify-center">
                    <span class="text-4xl md:text-5xl font-bold text-gray-800">{formatted_time}</span>
                    <div class="flex gap-2 mt-4">
                        <button 
                            on:click=toggle_timer
                            class="px-4 py-2 rounded-full bg-blue-500 text-white font-medium hover:bg-blue-600 transition-colors"
                        >
                            {if is_running.get() { "暂停" } else { "开始" }}
                        </button>
                        <button 
                            on:click=reset_timer
                            class="px-4 py-2 rounded-full bg-gray-200 text-gray-800 font-medium hover:bg-gray-300 transition-colors"
                        >
                            "重置"
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
