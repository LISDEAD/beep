use leptos::prelude::*;

// 根据目标架构导入不同的模块
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsValue, JsCast};
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

// 修复Tauri invoke函数导入和Object.get trait bounds问题
#[cfg(target_arch = "wasm32")]
pub async fn tauri_invoke(cmd: String, args: JsValue) -> Result<JsValue, JsValue> {
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
pub fn call_backend(cmd: String, args: JsValue) {
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