use leptos::prelude::*;
use leptos_meta::*;
use std::sync::Arc;
use std::sync::Mutex;

// 从模块中导入所需的内容
use crate::timer_logic::{TimerState, TOTAL_SECONDS};

// 根据目标架构导入不同的模块

// 为WebAssembly环境导入必要的类型
#[cfg(target_arch = "wasm32")]
use crate::wasm_specific::setup_timer_event_listener;

#[component]
pub fn App() -> impl IntoView {
    let (title, _set_title) = signal("计时器应用");

    provide_meta_context();

    // 创建计时器状态并使用Arc<Mutex>包装以便线程安全共享
    let timer_state = Arc::new(Mutex::new(TimerState::new()));

    // 创建所有需要的克隆变量
    let timer_state_clone1 = Arc::clone(&timer_state);
    let timer_state_clone2 = Arc::clone(&timer_state);
    let timer_state_clone3 = Arc::clone(&timer_state);
    let timer_state_clone4 = Arc::clone(&timer_state);
    let timer_state_clone5 = Arc::clone(&timer_state);
    let timer_state_clone6 = Arc::clone(&timer_state);
    let timer_state_clone7 = Arc::clone(&timer_state);
    let timer_state_clone8 = Arc::clone(&timer_state);
    let timer_state_clone9 = Arc::clone(&timer_state);
    let timer_state_clone10 = Arc::clone(&timer_state);
    let timer_state_clone11 = Arc::clone(&timer_state);
    let timer_state_clone12 = Arc::clone(&timer_state);
    let timer_state_clone13 = Arc::clone(&timer_state);

    // 监听后端计时器更新事件 - 仅在WebAssembly环境中
    #[cfg(target_arch = "wasm32")] {
        // 设置计时器事件监听
        setup_timer_event_listener(&timer_state_clone1);
    }

    // 创建响应式信号
    let (remaining_seconds, set_remaining_seconds) = create_signal(
        timer_state_clone2.lock().unwrap().remaining_seconds.get_untracked()
    );
    let (is_running, set_is_running) = create_signal(
        timer_state_clone3.lock().unwrap().is_running.get_untracked()
    );
    let (total_seconds, set_total_seconds) = create_signal(
        timer_state_clone4.lock().unwrap().total_seconds.get_untracked()
    );

    // 创建响应式效果，监听TimerState中的变化
    create_effect(move |_| {
        if let Ok(ts) = timer_state_clone2.lock() {
            // 使用with方法确保在响应式上下文中获取信号值
            let remaining = ts.remaining_seconds.with(|s| *s);
            set_remaining_seconds.set(remaining);
        }
    });

    create_effect(move |_| {
        if let Ok(ts) = timer_state_clone3.lock() {
            // 使用with方法确保在响应式上下文中获取信号值
            let running = ts.is_running.with(|s| *s);
            set_is_running.set(running);
        }
    });

    create_effect(move |_| {
        if let Ok(ts) = timer_state_clone4.lock() {
            // 使用with方法确保在响应式上下文中获取信号值
            let total = ts.total_seconds.with(|s| *s);
            set_total_seconds.set(total);
        }
    });

    // 计时器控制函数
    let start_timer = move |_| {
        if let Ok(mut ts) = timer_state_clone8.lock() {
            ts.start();
        }
    };

    let pause_timer = move |_| {
        if let Ok(mut ts) = timer_state_clone9.lock() {
            ts.pause();
        }
    };

    let reset_timer = move |_| {
        if let Ok(mut ts) = timer_state_clone10.lock() {
            ts.reset();
        }
    };

    // 更新总时间的函数
    let update_total_time = move |ev: leptos::ev::Event| {
        #[cfg(target_arch = "wasm32")]
        crate::wasm_specific::handle_update_total_time(&ev, &timer_state_clone11);
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // 在非WebAssembly环境中，我们使用一个固定值
            if let Ok(mut ts) = timer_state_clone11.lock() {
                ts.update_total_time(10);
            }
        }
    };

    // 圆环进度计算
    let stroke_dashoffset = move || {
        if let Ok(ts) = timer_state_clone12.lock() {
            ts.stroke_dashoffset()
        } else {
            0.0
        }
    };

    view! {
        <Title text=title />
        <main class="container min-h-screen flex flex-col items-center justify-center bg-gray-50 p-1 pt-0">
            <div class="relative w-24 h-24 mx-auto">
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
                        stroke-dasharray="628.3185307179587"
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
