use leptos::prelude::*;
use crate::tauri_utils::call_backend;

// 根据目标架构导入不同的模块
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

#[cfg(not(target_arch = "wasm32"))]
use crate::dummy_web_imports::JsValue;

// 常量定义
pub const TOTAL_SECONDS: u32 = 60;

// 计时器状态管理
#[derive(Clone)]
pub struct TimerState {
    pub remaining_seconds: Signal<u32>,
    pub set_remaining_seconds: WriteSignal<u32>,
    pub is_running: Signal<bool>,
    pub set_is_running: WriteSignal<bool>,
    pub total_seconds: Signal<u32>,
    pub set_total_seconds: WriteSignal<u32>,
}

impl TimerState {
    // 创建新的计时器状态
    pub fn new() -> Self {
        let (remaining_seconds, set_remaining_seconds) = signal(TOTAL_SECONDS);
        let (is_running, set_is_running) = signal(false);
        let (total_seconds, set_total_seconds) = signal(TOTAL_SECONDS);

        Self {
            remaining_seconds: remaining_seconds.into(),
            set_remaining_seconds,
            is_running: is_running.into(),
            set_is_running,
            total_seconds: total_seconds.into(),
            set_total_seconds,
        }
    }

    // 开始计时器
    pub fn start(&self) {
        self.set_is_running.set(true);
        call_backend(
            "start_timer".to_string(),
            JsValue::from(self.total_seconds.get_untracked())
        );
    }

    // 暂停计时器
    pub fn pause(&self) {
        self.set_is_running.set(false);
        call_backend("pause_timer".to_string(), JsValue::UNDEFINED);
    }

    // 重置计时器
    pub fn reset(&self) {
        let total = self.total_seconds.get_untracked();
        self.set_remaining_seconds.set(total);
        self.set_is_running.set(false);
        call_backend("reset_timer".to_string(), JsValue::UNDEFINED);
    }

    // 更新总时间
    pub fn update_total_time(&self, new_total: u32) {
        self.set_total_seconds.set(new_total);
        self.set_remaining_seconds.set(new_total);
        call_backend("set_total_seconds".to_string(), JsValue::from(new_total));
    }

    // 计算圆环进度
    pub fn stroke_dashoffset(&self) -> f64 {
        let circumference = 2.0 * std::f64::consts::PI * 100.0;
        let remaining = self.remaining_seconds.get() as f64;
        let total = self.total_seconds.get() as f64;

        if total == 0.0 {
            0.0
        } else {
            circumference * (1.0 - remaining / total)
        }
    }
}