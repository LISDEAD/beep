use leptos::prelude::*;
use leptos::ev::Event;
use leptos_meta::*;
// 重新导入PI，因为它被用于计算圆周长
use std::f64::consts::PI;
// 删除未使用的Rc和RefCell导入

// 在非WebAssembly环境中导入spawn_local函数
#[cfg(not(target_arch = "wasm32"))]
use leptos::task::spawn_local;

// 只在WebAssembly环境中导入Arc和Mutex
#[cfg(target_arch = "wasm32")]
use std::sync::{Arc, Mutex};

// 只在WebAssembly环境中导入js_sys和web_sys
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsValue, JsCast};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local as wasm_spawn_local;

// 创建一个新类型包装器以安全地实现Send和Sync
#[cfg(target_arch = "wasm32")]
struct SyncClosure(Closure<dyn FnMut(Event)>);

// 在WebAssembly单线程环境中，我们可以安全地标记SyncClosure为Send
#[cfg(target_arch = "wasm32")]
unsafe impl Send for SyncClosure {}

// 确保SyncClosure可以在线程间安全共享
#[cfg(target_arch = "wasm32")]
unsafe impl Sync for SyncClosure {}
#[cfg(target_arch = "wasm32")]
use js_sys;
#[cfg(target_arch = "wasm32")]
use web_sys::{self, HtmlInputElement, console}; // 删除未使用的MouseEvent和EventTarget导入

// 为非WebAssembly环境导入虚拟类型
#[cfg(not(target_arch = "wasm32"))]
mod dummy_web_imports {
    use std::clone::Clone;
    use leptos::wasm_bindgen::JsCast as LeptosJsCast;
    use leptos::wasm_bindgen::JsValue as LeptosJsValue;

    pub struct Event;
    pub struct JsValue;

    // 为虚拟类型添加必要的方法
    impl Event {
        pub fn target(&self) -> Option<EventTarget> {
            None
        }
    }

    // 实现AsRef<JsValue> trait
    impl AsRef<LeptosJsValue> for Event {
        fn as_ref(&self) -> &LeptosJsValue {
            // 注意：这是一个不安全的实现，仅用于编译通过
            unsafe { &*(0 as *const LeptosJsValue) }
        }
    }

    // 实现Into<JsValue> trait
    impl Into<LeptosJsValue> for Event {
        fn into(self) -> LeptosJsValue {
            // 注意：这是一个不安全的实现，仅用于编译通过
            unsafe { (*(0 as *mut LeptosJsValue)).clone() }
        }
    }

    // 实现leptos的JsCast trait
    impl LeptosJsCast for Event {
        fn unchecked_from_js(_: LeptosJsValue) -> Self {
            Self
        }

        fn unchecked_from_js_ref(_: &LeptosJsValue) -> &Self {
            // 注意：这是一个不安全的实现，仅用于编译通过
            unsafe { &*(0 as *const Self) }
        }

        fn instanceof(_: &LeptosJsValue) -> bool {
            true
        }
    }

    impl JsValue {
        pub fn from_str(_: &str) -> Self {
            Self
        }

        pub fn from(_: u32) -> Self {
            Self
        }

        pub fn as_f64(&self) -> Option<f64> {
            Some(0.0)
        }

        pub fn is_undefined(&self) -> bool {
            false
        }

        pub const UNDEFINED: Self = Self;
    }

    impl Clone for JsValue {
        fn clone(&self) -> Self {
            Self
        }
    }

    // 为了编译通过，定义其他必要的结构体
    pub struct EventTarget;
    pub struct HtmlInputElement;
}
#[cfg(not(target_arch = "wasm32"))]
use self::dummy_web_imports::JsValue;
#[cfg(not(target_arch = "wasm32"))]
use leptos::prelude::event_target_value;
// 移除未使用的LeptosEvent导入
// use leptos::ev::Event as LeptosEvent;

const TOTAL_SECONDS: u32 = 60;

#[component]
pub fn App() -> impl IntoView {
    let (title, _set_title) = signal("计时器应用");

    // 响应式状态
    let (remaining_seconds, set_remaining_seconds) = signal(TOTAL_SECONDS);
    let (is_running, set_is_running) = signal(false);
    let (total_seconds, set_total_seconds) = signal(TOTAL_SECONDS);

    provide_meta_context();

    // 监听后端计时器更新事件 - 仅在WebAssembly环境中
    #[cfg(target_arch = "wasm32")]
    Effect::new(move |_| {
        if let Some(window) = web_sys::window() {
            let _remaining = remaining_seconds.clone();
            let _running = is_running.clone();

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
                                        set_remaining_seconds.set(value as u32);
                                        if value == 0.0 {
                                            set_is_running.set(false);
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
            wasm_spawn_local(async move {
                if let Some(closure) = closure_clone.lock().unwrap().take() {
                    closure.0.forget();
                }
            });

            // Rc和RefCell已在文件顶部导入
        }
    });

    // 非WebAssembly环境下的空实现
    #[cfg(not(target_arch = "wasm32"))]
    Effect::new(move |_| {});

    // 调用后端命令
    // 在非WebAssembly环境中提供tauri_invoke的实现
    #[cfg(not(target_arch = "wasm32"))]
    async fn tauri_invoke(cmd: String, _args: JsValue) -> Result<JsValue, JsValue> {
        println!("Mock invoke for '{}' in non-wasm environment", cmd);
        Ok(JsValue::UNDEFINED)
    }

    // 修复Tauri invoke函数导入和Object.get trait bounds问题
    #[cfg(target_arch = "wasm32")]
    async fn tauri_invoke(cmd: String, args: JsValue) -> Result<JsValue, JsValue> {
        // 获取window和TAURI对象
        let window = web_sys::window().ok_or(JsValue::from_str("未找到window对象"))?;
        let tauri = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__"))
            .map_err(|_| JsValue::from_str("未找到__TAURI__对象"))?;

        // 检查invoke函数是否存在且是函数类型
        let invoke_fn = match js_sys::Reflect::get(&tauri, &JsValue::from_str("invoke")) {
            Ok(value) => {
                match value.dyn_into::<js_sys::Function>() {
                    Ok(func) => func,
                    Err(_) => {
                        web_sys::console::error_1(&JsValue::from_str("invoke不是函数类型"));
                        return Err(JsValue::from_str("invoke不是函数类型"));
                    }
                }
            },
            Err(_) => {
                web_sys::console::error_1(&JsValue::from_str("未找到invoke函数"));
                return Err(JsValue::from_str("未找到invoke函数"));
            }
        };

        // 检查并等待Tauri初始化完成
        // 尝试获取初始化状态，确保它是布尔值
        let is_initialized = match js_sys::Reflect::get(&tauri, &JsValue::from_str("isInitialized")) {
            Ok(value) => {
                if let Some(bool_val) = value.as_bool() {
                    bool_val
                } else {
                    // 如果不是布尔值，记录错误并默认未初始化
                    web_sys::console::error_1(&JsValue::from_str("isInitialized不是布尔值，默认未初始化"));
                    false
                }
            },
            Err(_) => {
                // 未找到isInitialized属性，默认未初始化
                false
            }
        };

        // 如果未初始化，等待初始化完成
        if !is_initialized {
            // 创建一个Promise等待Tauri初始化完成
            let promise = js_sys::Promise::new(&mut |resolve, reject| {
                // 尝试获取onInit函数
                match js_sys::Reflect::get(&tauri, &JsValue::from_str("onInit")) {
                    Ok(on_init) => {
                        // 尝试转换为函数类型
                        match on_init.dyn_into::<js_sys::Function>() {
                            Ok(on_init_fn) => {
                                let callback = Closure::wrap(Box::new(move || {
                                    resolve.call0(&JsValue::UNDEFINED).unwrap();
                                }) as Box<dyn FnMut()>);
                                // 调用onInit函数
                                if on_init_fn.call1(&tauri, &callback.as_ref().unchecked_ref()).is_err() {
                                    web_sys::console::error_1(&JsValue::from_str("调用onInit失败"));
                                    reject.call1(&JsValue::UNDEFINED, &JsValue::from_str("调用onInit失败")).unwrap();
                                }
                                callback.forget();
                            },
                            Err(_) => {
                                web_sys::console::error_1(&JsValue::from_str("onInit不是函数类型"));
                                // 即使onInit不是函数类型，也尝试直接resolve，避免阻塞
                                resolve.call0(&JsValue::UNDEFINED).unwrap();
                            }
                        }
                    },
                    Err(_) => {
                        web_sys::console::error_1(&JsValue::from_str("未找到onInit函数"));
                        // 未找到onInit函数，尝试直接resolve
                        resolve.call0(&JsValue::UNDEFINED).unwrap();
                    }
                }
            });

            // 等待Promise完成
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(_) => {},
                Err(e) => {
                    web_sys::console::error_1(&JsValue::from_str(&format!("等待Tauri初始化失败: {:?}", e)));
                    // 即使等待失败也继续执行，尝试直接调用invoke
                }
            }
        }

        // 使用之前已经检查过的invoke_fn调用函数
        js_sys::Reflect::apply(&invoke_fn, &tauri, &js_sys::Array::of2(&JsValue::from_str(&cmd), &args))
    }

    // 定义call_backend函数
    let call_backend = move |cmd: String, args: JsValue| {
        #[cfg(target_arch = "wasm32")]
        {
            wasm_spawn_local(async move {
                match tauri_invoke(cmd.clone(), args.clone()).await {
                    Ok(value) => {
                        web_sys::console::log_1(&JsValue::from_str(&format!(
                            "调用成功: {:?}",
                            value
                        )));
                    }
                    Err(e) => {
                        web_sys::console::error_1(&JsValue::from_str(&format!(
                            "调用失败: {:?}",
                            e
                        )));
                    }
                }
            });
        }

        // 非WASM环境下，我们需要使用leptos的event_target_value函数
        #[cfg(not(target_arch = "wasm32"))]
        {
            spawn_local(async move {
                match tauri_invoke(cmd.clone(), args.clone()).await {
                    Ok(_) => {
                        println!("调用成功");
                    }
                    Err(_) => {
                        println!("调用失败");
                    }
                }
            });
        }
    };

    // 计时器控制函数 - 移除具体类型注解以避免类型不匹配
    let start_timer = move |_| {
        set_is_running.set(true);
        call_backend("start_timer".to_string(), JsValue::from(total_seconds.get_untracked()));
    };

    let pause_timer = move |_| {
        set_is_running.set(false);
        call_backend("pause_timer".to_string(), JsValue::UNDEFINED);
    };

    let reset_timer = move |_| {
        set_remaining_seconds.set(total_seconds.get_untracked());
        set_is_running.set(false);
        call_backend("reset_timer".to_string(), JsValue::UNDEFINED);
    };

    // 更新总时间的函数 - 修正闭包参数类型
    // 修复未使用参数警告
// 使用通用事件类型 - 添加类型注解
let update_total_time = move |ev: Event| {
        #[cfg(not(target_arch = "wasm32"))]
        {
            // 使用传入的事件参数
            let value = event_target_value(&ev);
            if let Ok(parsed_value) = value.parse::<u32>() {
                set_total_seconds.set(parsed_value);
                set_remaining_seconds.set(parsed_value);
                // 确保命令名称与后端注册的一致
                call_backend("set_total_seconds".to_string(), JsValue::from(parsed_value));
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let target = ev.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
            if let Some(input) = target {
                if let Ok(parsed_value) = input.value().parse::<u32>() {
                    set_total_seconds.set(parsed_value);
                    set_remaining_seconds.set(parsed_value);
                    // 确保命令名称与后端注册的一致
                    call_backend("set_total_seconds".to_string(), JsValue::from(parsed_value));
                }
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
        <main class="container min-h-screen flex flex-col items-center justify-center bg-gray-50 p-1 pt-0">
            <div class="relative w-6 h-6 mx-auto">
                <svg class="absolute inset-0 w-full h-full" viewBox="0 0 40 40">
                    <defs>
                        <filter id="textShadow" x="-20%" y="-20%" width="140%" height="140%">
                            <feDropShadow dx="0.3" dy="0.3" stdDeviation="0.2" flood-color="#000" flood-opacity="0.2"/>
                        </filter>
                    </defs>

                    <circle cx="20" cy="20" r="10" fill="none" stroke="#e6e6e6" stroke-width="1.5"/>

                    <circle
                        cx="20" cy="20" r="10"
                        fill="none" stroke="#3b82f6" stroke-width="1.5"
                        stroke-dasharray={format!("{}", circumference)}
                        stroke-dashoffset={move || stroke_dashoffset().to_string()}
                        stroke-linecap="round"
                        transform="rotate(-90 20 20)"
                        class="transition-all duration-300 ease-in-out"
                    />

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

            <div class="flex flex-col items-center gap-6">
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

                <div class="flex items-center gap-3 p-2 w-full max-w-xs">
                    <label for="total-time" class="text-gray-700 dark:text-gray-300 text-lg">"总时间(秒):"</label>
                    <input
                        id="total-time"
                        type="number"
                        value=move || total_seconds.get().to_string()
                        on:change=update_total_time
                        min=1
                        class="w-28 p-2 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-800 text-gray-900 dark:text-white text-lg"
                    />
                </div>
            </div>
        </main>
    }
}
