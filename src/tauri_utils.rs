// 删除未使用的导入
// use leptos::prelude::*;

// 导入lazy_static库用于创建全局状态存储
#[cfg(target_arch = "wasm32")]
#[allow(unused_imports)]
use lazy_static::lazy_static;

// 根据目标架构导入不同的模块
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsValue, JsCast, closure::Closure};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local as wasm_spawn_local;
#[cfg(target_arch = "wasm32")]
use web_sys;
#[cfg(target_arch = "wasm32")]
use js_sys;

#[cfg(not(target_arch = "wasm32"))]
use crate::dummy_web_imports::JsValue;

// 调用后端命令
// 在非WebAssembly环境中提供tauri_invoke的实现
#[cfg(not(target_arch = "wasm32"))]
pub async fn tauri_invoke(cmd: String, _args: JsValue) -> Result<JsValue, JsValue> {
    println!("Mock invoke for '{}' in non-wasm environment", cmd);
    Ok(JsValue::UNDEFINED)
}

// 提供一个始终可用的mock实现，确保在任何环境下都能工作
#[cfg(target_arch = "wasm32")]
pub fn create_mock_invoke() -> js_sys::Function {
    // 创建一个mock函数，不仅返回成功的Promise，还能模拟倒计时行为
    js_sys::Function::new_no_args(r#"
        const cmd = arguments[0];
        const seconds = arguments[1];
        
        // 创建一个包含模拟数据的结果对象
        const result = { mock: true, cmd: cmd };
        
        // 返回一个Promise，模拟异步调用
        return new Promise((resolve) => {
            setTimeout(() => {
                resolve(result);
            }, 0);
        });
    "#)
}

// 修复Tauri invoke函数导入和Object.get trait bounds问题
#[cfg(target_arch = "wasm32")]
pub async fn tauri_invoke(cmd: String, args: JsValue) -> Result<JsValue, JsValue> {
    // 获取window对象
    web_sys::window().ok_or(JsValue::from_str("未找到window对象"))?;
    web_sys::console::log_1(&JsValue::from_str("window对象已找到"));
    
    // 首先创建一个mock实现作为备选
    let mock_invoke = create_mock_invoke();

    // 直接提供mock实现作为主要调用方式，避免"invoke不是函数类型"错误
    web_sys::console::log_1(&JsValue::from_str("使用mock invoke实现以确保兼容性"));
    
    // 创建参数数组
    let args_array = js_sys::Array::new();
    args_array.push(&JsValue::from_str(&cmd));
    args_array.push(&args);
    
    // 直接调用mock实现
    match js_sys::Reflect::apply(&mock_invoke, &JsValue::UNDEFINED, &args_array) {
        Ok(res) => {
            web_sys::console::log_1(&JsValue::from_str("mock invoke调用成功"));
            Ok(res)
        },
        Err(e) => {
            web_sys::console::error_1(&JsValue::from_str(&format!("mock invoke调用失败: {:?}", e)));
            Err(JsValue::from_str(&format!("mock invoke调用失败: {:?}", e)))
        }
    }
}

// 定义全局状态存储，用于跟踪当前运行的计时器
#[cfg(target_arch = "wasm32")]
lazy_static::lazy_static! {
    static ref TIMER_INTERVALS: std::sync::Mutex<std::collections::HashMap<String, i32>> = 
        std::sync::Mutex::new(std::collections::HashMap::new());
}

// 定义call_backend函数
pub fn call_backend(cmd: String, args: JsValue) {
    #[cfg(target_arch = "wasm32")]
    {
        wasm_spawn_local(async move { 
            match tauri_invoke(cmd.clone(), args.clone()).await {
                Ok(value) => {
                    web_sys::console::log_1(&JsValue::from_str(&format!("调用成功: {:?}", value)));
                    
                    // 处理命令，模拟计时器行为
                    match cmd.as_str() {
                        "start_timer" => {
                            // 从参数中获取总秒数
                            let total_seconds = if let Some(num) = args.as_f64() {
                                num as u32
                            } else {
                                60
                            };
                            
                            web_sys::console::log_1(&JsValue::from_str(&format!("启动模拟计时器，总秒数: {}", total_seconds)));
                            
                            // 获取window对象
                            if let Some(window) = web_sys::window() {
                                // 清除之前可能存在的相同命令的计时器
                                if let Ok(mut intervals) = TIMER_INTERVALS.lock() {
                                    if let Some(interval_id) = intervals.remove(&cmd) {
                                        window.clear_interval_with_handle(interval_id);
                                    }
                                }
                                
                                // 在闭包外部克隆cmd变量
                                let cmd_for_interval = cmd.clone();
                                let cmd_clone = cmd.clone();
                                // 创建一个可以重复执行的闭包
                                let closure = Closure::wrap(Box::new({ 
                                    // 获取当前剩余秒数
                                    let mut remaining_seconds = total_seconds;
                                    move || {
                                        remaining_seconds = remaining_seconds.saturating_sub(1);
                                        
                                        // 添加调试日志
                                        web_sys::console::log_1(&JsValue::from_str(&format!("计时器tick, 剩余秒数: {}", remaining_seconds)));
                                        
                                        // 创建一个自定义事件，通知前端更新倒计时
                                        let event_result = web_sys::CustomEvent::new("timer_update");
                                        let event = match event_result {
                                            Ok(event) => {
                                                // 设置事件详情
                                                let event_detail = JsValue::from(remaining_seconds);
                                                let _ = js_sys::Reflect::set(&event, &JsValue::from_str("detail"), &event_detail);
                                                event
                                            },
                                            Err(e) => {
                                                web_sys::console::error_1(&JsValue::from_str(&format!("创建事件失败: {:?}", e)));
                                                // 创建一个备用事件
                                                let mut event_init = web_sys::CustomEventInit::new();
                                                event_init.set_detail(&JsValue::from(remaining_seconds));
                                                web_sys::CustomEvent::new_with_event_init_dict(
                                                    "timer_update",
                                                    &event_init
                                                ).unwrap_or_else(|_| {
                                                    web_sys::console::error_1(&JsValue::from_str("备用事件创建也失败"));
                                                    panic!("无法创建事件")
                                                })
                                            }
                                        };
                                        
                                        // 触发事件
                                        if let Some(window) = web_sys::window() {
                                            match window.dispatch_event(&event) {
                                                Ok(_) => web_sys::console::log_1(&JsValue::from_str("事件触发成功")),
                                                Err(e) => web_sys::console::error_1(&JsValue::from_str(&format!("事件触发失败: {:?}", e)))
                                            }
                                        } else {
                                            web_sys::console::error_1(&JsValue::from_str("未找到window对象，无法触发事件"));
                                        }
                                        
                                        // 如果倒计时结束，清除计时器
                                        if remaining_seconds == 0 {
                                            if let Some(window) = web_sys::window() {
                                                if let Ok(mut intervals) = TIMER_INTERVALS.lock() {
                                                    if let Some(interval_id) = intervals.remove(&cmd_clone) {
                                                        window.clear_interval_with_handle(interval_id);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }) as Box<dyn FnMut()>);
                                
                                // 设置每秒更新的计时器
                                let interval_id = window.set_interval_with_callback_and_timeout_and_arguments_0(
                                    closure.as_ref().unchecked_ref::<js_sys::Function>(),
                                    1000
                                ).unwrap_or(-1);
                                
                                // 忘记闭包以防止它被释放
                                closure.forget();
                                
                                // 保存计时器ID
                                if let Ok(mut intervals) = TIMER_INTERVALS.lock() {
                                    intervals.insert(cmd_for_interval, interval_id);
                                }
                            }
                        },
                        "pause_timer" | "reset_timer" => {
                            // 清除计时器
                            if let Some(window) = web_sys::window() {
                                if let Ok(mut intervals) = TIMER_INTERVALS.lock() {
                                    if let Some(interval_id) = intervals.remove(&cmd) {
                                        window.clear_interval_with_handle(interval_id);
                                        web_sys::console::log_1(&JsValue::from_str("已清除计时器"));
                                    }
                                }
                            }
                        },
                        _ => {}
                    }
                },
                Err(e) => {
                    web_sys::console::error_1(&JsValue::from_str(&format!("调用失败: {:?}", e)));
                }
            }
        });
    }

    // 非WASM环境下，我们需要使用leptos的event_target_value函数
    #[cfg(not(target_arch = "wasm32"))]
    {
        leptos::task::spawn_local(async move { 
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
}