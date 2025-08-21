// src/wasm-initializer.js
// WebAssembly模块加载和初始化功能

// 导入通用工具函数
import { validateWasmMagicNumber, isHtmlResponse, instantiateWasmModule, loadJavaScriptWrapper } from './wasm-utils.js';

/**
 * 加载WASM二进制文件
 * @param {string} wasmUrl - WASM文件URL
 * @returns {Promise<ArrayBuffer>} - WASM二进制数据
 */
async function loadWasmBinary(wasmUrl) {
    try {
        const response = await fetch(wasmUrl);
        if (!response.ok) {
            throw new Error(`WASM文件请求失败: ${response.status} ${response.statusText}`);
        }

        const contentType = response.headers.get('content-type');
        if (contentType && !contentType.includes('application/wasm') && 
            !contentType.includes('octet-stream') && !contentType.includes('binary')) {
            console.warn(`WASM文件类型不是预期的: ${contentType}`);
        }

        return await response.arrayBuffer();
    } catch (error) {
        console.error('加载WASM二进制文件时出错:', error);
        throw error;
    }
}

/**
 * 尝试加载备用WASM路径
 * @param {string} originalPath - 原始路径
 * @returns {Promise<ArrayBuffer|null>} - WASM二进制数据或null
 */
async function loadAlternativeWasmPath(originalPath) {
    // 尝试几个可能的备用路径
    const alternativePaths = [
        '/dist/beep-ui_bg.wasm',
        '/beep-ui_bg.wasm',
        '/src/beep-ui_bg.wasm'
    ];

    for (const path of alternativePaths) {
        if (path === originalPath) continue;

        try {
            console.log(`尝试备用WASM路径: ${path}`);
            const bytes = await loadWasmBinary(path);
            if (validateWasmMagicNumber(bytes)) {
                console.log(`成功加载备用WASM路径: ${path}`);
                return bytes;
            }
        } catch (e) {
            // 忽略错误，继续尝试下一个路径
        }
    }

    return null;
}

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
        const wasmUrl = customPath || '/dist/beep-ui_bg.wasm';
        console.log(`尝试加载WASM文件: ${wasmUrl}`);

        // 尝试加载WASM二进制文件
        let bytes;
        try {
            bytes = await loadWasmBinary(wasmUrl);
        } catch (error) {
            // 尝试加载备用路径
            bytes = await loadAlternativeWasmPath(wasmUrl);
            if (!bytes) {
                throw new Error('无法加载WASM文件，所有路径都尝试失败');
            }
        }

        // 验证WASM文件格式
        if (!validateWasmMagicNumber(bytes)) {
            // 检查是否是HTML文件
            if (isHtmlResponse(bytes)) {
                console.error('服务器返回了HTML文件而不是WASM文件，可能是服务器配置问题');
                // 尝试加载JavaScript包装器
                const jsWrapperUrl = wasmUrl.replace('_bg.wasm', '.js');
                console.log(`尝试加载JavaScript包装器: ${jsWrapperUrl}`);
                const wrapperLoaded = await loadJavaScriptWrapper(jsWrapperUrl);
                if (!wrapperLoaded) {
                    throw new Error('WASM文件格式错误且无法加载JavaScript包装器');
                }

                // 检查是否通过包装器初始化成功
                if (window.wasm && typeof window.wasm === 'object') {
                    console.log('WASM模块已通过JavaScript包装器初始化');
                    return window.wasm;
                }

                // 如果包装器没有直接初始化，尝试调用初始化函数
                if (typeof window.initWasm === 'function') {
                    console.log('尝试调用window.initWasm函数');
                    const module = await window.initWasm();
                    if (module) {
                        window.wasm = module;
                        return module;
                    }
                }

                throw new Error('JavaScript包装器加载成功，但未能初始化WASM模块');
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
 * 加载WebAssembly模块的入口函数
 */
function loadWasmModule() {
    // 页面加载完成后初始化
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

// 导出必要的函数
window.loadWasmModule = loadWasmModule;
window.initWebAssembly = initWebAssembly;

// 自动加载WASM模块
loadWasmModule();