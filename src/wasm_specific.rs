use leptos::prelude::*;
use wasm_bindgen::{JsValue, JsCast};
use wasm_bindgen::closure::Closure;
use leptos::web_sys::{Event, HtmlInputElement, window, console};
use std::sync::{Arc, Mutex};
use crate::timer_logic::TimerState;
use wasm_bindgen_futures::spawn_local;
use leptos::prelude::request_animation_frame;


// 创建一个新类型包装器以安全地实现Send和Sync
pub struct SyncClosure(Closure<dyn FnMut(Event)>);

// 在WebAssembly单线程环境中，我们可以安全地标记SyncClosure为Send
unsafe impl Send for SyncClosure {}

// 确保SyncClosure可以在线程间安全共享
unsafe impl Sync for SyncClosure {}

// 设置计时器更新事件监听
pub fn setup_timer_event_listener(timer_state: &Arc<Mutex<TimerState>>) {
    if let Some(window) = window() {
        // 创建一个新的Arc引用，用于在闭包中使用
        let timer_state_clone = Arc::clone(timer_state);

        // 创建事件回调
        let closure = Closure::wrap(Box::new(move |event: Event| {
            // 从事件中获取detail属性
            if let Ok(detail_value) = js_sys::Reflect::get(&event, &JsValue::from_str("detail")) {
                if let Some(remaining_seconds) = detail_value.as_f64() {
                    // 转换为u32
                    let seconds = remaining_seconds as u32;
                    
                    // 创建一个新的克隆用于动画帧内部
                    let timer_state_clone2 = Arc::clone(&timer_state_clone);
                    
                    // 使用Leptos的request_animation_frame确保在正确的响应式上下文中更新
                    request_animation_frame(move || {
                        // 安全地获取TimerState并更新
                        if let Ok(mut timer_state) = timer_state_clone2.lock() {
                            // 在响应式上下文中更新信号
                            timer_state.set_remaining_seconds.set(seconds);
                            
                            // 当倒计时结束时，设置is_running为false
                            if remaining_seconds == 0.0 {
                                timer_state.set_is_running.set(false);
                            }
                        }
                    });
                }
            }
        }) as Box<dyn FnMut(Event)>);

        // 获取回调函数引用并添加事件监听器
        let js_callback = closure.as_ref().unchecked_ref::<js_sys::Function>();
        if let Err(err) = window.add_event_listener_with_callback("timer_update", js_callback) {
            console::error_1(&JsValue::from(format!("Failed to add event listener: {:?}", err)));
        }

        // 防止闭包被垃圾回收
        closure.forget();
    }
}

// WebAssembly环境下更新总时间的处理函数
pub fn handle_update_total_time(event: &Event, timer_state: &Arc<Mutex<TimerState>>) {
    let target = event.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
    if let Some(input) = target {
        if let Ok(parsed_value) = input.value().parse::<u32>() {
            timer_state.lock().unwrap().update_total_time(parsed_value);
        }
    }
}