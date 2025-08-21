use leptos::prelude::*;
use wasm_bindgen::{JsValue, JsCast};
use wasm_bindgen::closure::Closure;
use leptos::web_sys::{Event, HtmlInputElement};
use std::sync::{Arc, Mutex};
use crate::timer_logic::TimerState;
use js_sys::Reflect;
use wasm_bindgen_futures::spawn_local;

// 创建一个新类型包装器以安全地实现Send和Sync
pub struct SyncClosure(Closure<dyn FnMut(Event)>);

// 在WebAssembly单线程环境中，我们可以安全地标记SyncClosure为Send
unsafe impl Send for SyncClosure {}

// 确保SyncClosure可以在线程间安全共享
unsafe impl Sync for SyncClosure {}

// 设置计时器更新事件监听
pub fn setup_timer_event_listener(timer_state: &Arc<Mutex<TimerState>>) {
    if let Some(window) = leptos::web_sys::window() {
        let timer_state_borrowed = timer_state.lock().unwrap();
        let remaining = timer_state_borrowed.remaining_seconds.clone();
        let set_remaining = timer_state_borrowed.set_remaining_seconds.clone();
        let set_running = timer_state_borrowed.set_is_running.clone();

        // 创建事件回调
        // 使用Arc和Mutex管理闭包生命周期以满足Send + Sync约束
        let closure = Arc::new(
            Mutex::new(
                Some(
                    SyncClosure(
                        Closure::wrap(Box::new(move |event: Event| {
                            let _ = js_sys::Reflect::get(&event, &JsValue::from_str("detail"))
                                .and_then(|detail| {
                                    detail
                                        .as_f64()
                                        .ok_or(JsValue::from_str("detail is not a number"))
                                })
                                .map(|value| {
                                    set_remaining.set(value as u32);
                                    if value == 0.0 {
                                        set_running.set(false);
                                    }
                                });
                        }))
                    )
                )
            )
        );

        // 获取回调函数引用
        let locked_closure = closure.lock().unwrap();
        let some_closure = locked_closure.as_ref().unwrap();
        let inner_closure = &some_closure.0;
        let js_callback = inner_closure.as_ref().unchecked_ref::<js_sys::Function>();
        let _ = window.add_event_listener_with_callback("timer_update", js_callback);

        // 使用wasm_spawn_local确保在WebAssembly主线程中执行清理
        let closure_clone = Arc::clone(&closure);
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(closure) = closure_clone.lock().unwrap().take() {
                closure.0.forget();
            }
        });
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