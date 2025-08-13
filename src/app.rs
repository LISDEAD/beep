use js_sys::Reflect;
use leptos::prelude::*;
use leptos_meta::*;
use std::f64::consts::PI;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{self, Event, HtmlInputElement};

const TOTAL_SECONDS: u32 = 60;

#[component]
pub fn App() -> impl IntoView {
    let (title, _set_title) = signal("计时器应用");

    // 响应式状态
    let remaining_seconds = RwSignal::new(TOTAL_SECONDS);
    let is_running = RwSignal::new(false);
    let total_seconds = RwSignal::new(TOTAL_SECONDS);

    provide_meta_context();

    // 监听后端计时器更新事件
    Effect::new(move |_| {
        let window = web_sys::window().expect("Failed to get window");
        let remaining = remaining_seconds.clone();
        let running = is_running.clone();

        // 创建事件回调
        let callback = Closure::wrap(Box::new(move |event: Event| {
            if let Ok(detail) = Reflect::get(&event, &"detail".into()) {
                if let Some(value) = detail.as_f64() {
                    remaining.set(value as u32);
                    if value == 0.0 {
                        running.set(false);
                    }
                }
            }
        }) as Box<dyn FnMut(Event)>);

        // 保存回调的JS引用并注册事件
        let js_callback = callback.as_ref().unchecked_ref::<js_sys::Function>();
        let _ = window.add_event_listener_with_callback("timer_update", js_callback);

        // 存储回调指针
        let callback_ptr: *const Closure<dyn FnMut(Event)> = &callback;

        // 创建清理闭包
        let cleanup_closure = move || {
            if let Some(window) = web_sys::window() {
                unsafe {
                    let callback: &Closure<dyn FnMut(Event)> = &*callback_ptr;
                    let _ = window.remove_event_listener_with_callback(
                        "timer_update",
                        callback.as_ref().unchecked_ref(),
                    );
                }
            }
        };

        // 强制转换闭包类型以满足Send + Sync约束
        let send_sync_closure = unsafe {
            std::mem::transmute::<Box<dyn FnOnce()>, Box<dyn FnOnce() + Send + Sync>>(Box::new(
                cleanup_closure,
            ))
        };

        // 注册清理函数
        on_cleanup(send_sync_closure);

        // 确保回调生命周期
        callback.forget();
    });

    // 修复：调用后端命令（支持带参数）
    let call_backend = |cmd: &str, args: Option<js_sys::Object>| {
        if let Some(window) = web_sys::window() {
            if let Ok(tauri) = Reflect::get(&window, &"__TAURI__".into()) {
                if let Ok(invoke) = Reflect::get(&tauri, &"invoke".into()) {
                    if let Ok(invoke_fn) = invoke.dyn_into::<js_sys::Function>() {
                        let args = if let Some(args) = args {
                            js_sys::Array::of2(&JsValue::from_str(cmd), &JsValue::from(args))
                        } else {
                            js_sys::Array::of1(&JsValue::from_str(cmd))
                        };
                        let _ = invoke_fn.apply(&JsValue::undefined(), &args);
                    }
                }
            }
        }
    };

    // 计时器控制函数（修复调用方式）
    let start_timer = move |_| {
        call_backend("start_timer", None);
        is_running.set(true);
    };

    let pause_timer = move |_| {
        call_backend("pause_timer", None);
        is_running.set(false);
    };

    let reset_timer = move |_| {
        call_backend("reset_timer", None);
        remaining_seconds.set(total_seconds.get_untracked());
        is_running.set(false);
    };

    // 更新总时间（修复参数传递）
    let update_total_time = move |ev: Event| {
        let input = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
        if let Some(input) = input {
            if let Ok(value) = input.value().parse::<u32>() {
                total_seconds.set(value);
                remaining_seconds.set(value);
                
                // 正确传递参数
                let args = js_sys::Object::new();
                let _ = Reflect::set(&args, &"seconds".into(), &JsValue::from(value));
                call_backend("set_total_seconds", Some(args));
            }
        }
    };

    // 圆环进度计算
    let circumference = 2.0 * PI * 100.0;
    let stroke_dashoffset = move || {
        let remaining = remaining_seconds.get() as f64;
        let total = total_seconds.get() as f64;
        if total == 0.0 {
            0.0
        } else {
            circumference * (1.0 - remaining / total)
        }
    };

    view! {
        <Title text=title />
        // 使用容器类确保整体居中
        <main class="container min-h-screen flex flex-col items-center justify-center bg-gray-50 p-1 pt-0">
                // 彻底修复下方元素居中问题，确保与圆环严格对齐
    <div class="relative w-6 h-6">
    // 圆环容器自身也添加mx-auto确保居中
      <svg class="absolute inset-0 w-full h-full" viewBox="0 0 40 40">
        // 定义阴影滤镜
        <defs>
          <filter id="textShadow" x="-20%" y="-20%" width="140%" height="140%">
            <feDropShadow dx="0.3" dy="0.3" stdDeviation="0.2" flood-color="#000" flood-opacity="0.2"/>
          </filter>
        </defs>


        // 背景圆环
        <circle cx="20" cy="20" r="10" fill="none" stroke="#e6e6e6" stroke-width="1"/>


        // 进度圆环
        <circle
          cx="20" cy="20" r="10"
          fill="none" stroke="#3b82f6" stroke-width="1"
          stroke-dasharray={format!("{}", circumference)}
          stroke-dashoffset={move || stroke_dashoffset().to_string()}
          stroke-linecap="round"
          transform="rotate(-90 20 20)"
          class="transition-all duration-300 ease-in-out"
        />


        // 倒计时文字
        <text
          x="20.1" y="20.8"
          text-anchor="middle"
          dominant-baseline="middle"
          font-size="3.2"
          font-family="monospace"
          fill="#3b3b3bff"
          filter="url(#textShadow)"
          class="font-bold"
        >
          {move || format!("{}s", remaining_seconds.get())}
        </text>
      </svg>
    </div>


            {/* 按钮和输入框区域 - 确保居中 */}
            <div class="flex flex-col items-center gap-6">
                // 按钮组
                <div class="flex flex-wrap gap-4 justify-center">
                    <button
                        on:click=start_timer
                        disabled=move || is_running.get()
                        class="px-6 py-3 bg-blue-600 text-white rounded-full hover:bg-blue-700 disabled:bg-gray-400 transition-colors"
                    >
                        "开始"
                    </button>
                    <button
                        on:click=pause_timer
                        disabled=move || !is_running.get()
                        class="px-6 py-3 bg-amber-600 text-white rounded-full hover:bg-amber-700 disabled:bg-gray-400 transition-colors"
                    >
                        "暂停"
                    </button>
                    <button
                        on:click=reset_timer
                        class="px-6 py-3 bg-gray-600 text-white rounded-full hover:bg-gray-700 transition-colors"
                    >
                        "重置"
                    </button>
                </div>

                // 输入区
                <div class="flex items-center gap-3 p-2 w-full max-w-xs">
                    <label for="total-time" class="text-gray-700 dark:text-gray-300 text-lg">"总时间(秒):"</label>
                    <input
                        id="total-time"
                        type="number"
                        value=move || total_seconds.get().to_string()
                        on:change=update_total_time
                        min="1"
                        class="w-28 p-2 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-800 text-gray-900 dark:text-white text-lg"
                    />
                </div>
            </div>
        </main>
    }
}
