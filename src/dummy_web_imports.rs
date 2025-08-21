use std::clone::Clone;
use std::fmt;

// 为非WebAssembly环境定义简单的Event和JsValue模拟
pub struct Event;
pub struct JsValue;

// 定义必要的trait实现，但避免使用不安全的空指针
impl fmt::Debug for JsValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JsValue(dummy)")
    }
}

// 为虚拟类型添加必要的方法
impl Event {
    pub fn target(&self) -> Option<EventTarget> {
        None
    }
}

// 简化的JsValue实现
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

// 实现用于非WebAssembly环境的简单转换
impl From<u32> for JsValue {
    fn from(_: u32) -> Self {
        Self
    }
}

impl From<&str> for JsValue {
    fn from(_: &str) -> Self {
        Self
    }
}