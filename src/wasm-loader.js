// src/wasm-loader.js
// WebAssembly模块加载器

// 导入通用工具函数
import { validateWasmMagicNumber, isHtmlResponse, createWbgProxy, instantiateWasmModule, loadJavaScriptWrapper } from './wasm-utils.js';

/**
 * 初始化WebAssembly模块
 * @param {string} [customPath] - 自定义WASM文件路径
 * @returns {Promise<object>} - 初始化后的WASM模块导出
 */
async function initWebAssembly(customPath = null) {
    try {
        // 检查是否已经初始化
        if (window.wasm && typeof window.wasm === 'object') {
            console.log('WASM模块已经初始化');
            return window.wasm;
        }

        // 使用提供的路径或默认路径
        const wasmUrl = customPath || '/dist/beep-ui-7eb184cf2d003b81_bg.wasm';
        console.log(`尝试加载WASM文件: ${wasmUrl}`);

        // 获取WASM二进制文件
        const response = await fetch(wasmUrl);
        if (!response.ok) {
            console.error(`WASM文件请求失败: ${response.status} ${response.statusText}`);
            throw new Error(`无法加载WASM文件: ${response.status} ${response.statusText}`);
        }

        // 检查文件类型
        const contentType = response.headers.get('content-type');
        if (contentType && contentType.includes('text/html')) {
            console.error('服务器返回了HTML文件而不是WASM文件，可能是服务器配置问题');
            // 尝试加载JavaScript包装器
            const jsWrapperUrl = wasmUrl.replace('_bg.wasm', '.js');
            console.log(`尝试加载JavaScript包装器: ${jsWrapperUrl}`);
            const wrapperLoaded = await loadJavaScriptWrapper(jsWrapperUrl);
            if (!wrapperLoaded) {
                throw new Error('WASM文件请求返回HTML且无法加载JavaScript包装器');
            }
            return window.wasm;
        }

        // 读取WASM文件内容
        let bytes;
        try {
            bytes = await response.arrayBuffer();
        } catch (e) {
            console.error('读取WASM文件时出错:', e);
            // 尝试重新请求
            const freshResponse = await fetch(wasmUrl);
            if (!freshResponse.ok) {
                throw new Error(`无法重新加载WASM文件: ${freshResponse.status} ${freshResponse.statusText}`);
            }
            bytes = await freshResponse.arrayBuffer();
        }

        // 验证WASM文件格式
        if (!validateWasmMagicNumber(bytes)) {
            console.error('WASM文件格式错误: 不是有效的WebAssembly二进制文件');
            // 检查是否是HTML文件
            if (isHtmlResponse(bytes)) {
                console.error('检测到返回的是HTML文件，这通常表示服务器配置问题');
                // 尝试加载JavaScript包装器
                const jsWrapperUrl = wasmUrl.replace('_bg.wasm', '.js');
                console.log(`尝试加载JavaScript包装器: ${jsWrapperUrl}`);
                const wrapperLoaded = await loadJavaScriptWrapper(jsWrapperUrl);
                if (!wrapperLoaded) {
                    throw new Error('WASM文件格式错误且无法加载JavaScript包装器');
                }
                return window.wasm;
            }
            throw new Error('WASM文件格式错误: 不是有效的WebAssembly二进制文件');
        }

        console.log(`成功加载WASM二进制文件，大小: ${bytes.byteLength} 字节`);

        // 实例化WASM模块
        const module = await instantiateWasmModule(bytes);

        // 保存到全局作用域
        window.wasm = module;

        // 尝试调用start方法（如果存在）
        if (module.start) {
            console.log('尝试调用WebAssembly模块的start方法');
            module.start();
        }

        return module;
    } catch (error) {
        console.error('WebAssembly初始化失败:', error);
        throw error;
    }
}

/**
 * 设置WebAssembly初始化
 * 页面加载完成后初始化WebAssembly
 */
function setupWebAssembly() {
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', () => {
            initWebAssembly().catch(error => {
                console.error('WebAssembly初始化失败:', error);
            });
        });
    } else {
        // 如果页面已经加载完成，直接初始化
        initWebAssembly().catch(error => {
            console.error('WebAssembly初始化失败:', error);
        });
    }
}

// 导出函数，供其他文件使用
window.setupWebAssembly = setupWebAssembly;
window.initWebAssembly = initWebAssembly;