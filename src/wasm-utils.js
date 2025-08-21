// wasm-utils.js
// WebAssembly通用工具函数

/**
 * 验证WASM文件的魔法数字
 * @param {ArrayBuffer} bytes - WASM二进制数据
 * @returns {boolean} - 是否是有效的WASM文件
 */
export function validateWasmMagicNumber(bytes) {
    if (bytes.byteLength < 4) return false;
    const header = new Uint8Array(bytes.slice(0, 4));
    // WASM魔法数字: 0x00 0x61 0x73 0x6D
    return header[0] === 0x00 && header[1] === 0x61 && header[2] === 0x73 && header[3] === 0x6D;
}

/**
 * 检查是否是HTML响应
 * @param {ArrayBuffer} bytes - 响应数据
 * @returns {boolean} - 是否是HTML内容
 */
export function isHtmlResponse(bytes) {
    if (bytes.byteLength < 5) return false;
    const header = new Uint8Array(bytes.slice(0, 5));
    // HTML文件通常以<!DOC开头
    return header[0] === 0x3C && header[1] === 0x21 && header[2] === 0x44 && header[3] === 0x4F && header[4] === 0x43;
}

/**
 * 创建WBG代理对象，处理缺失的导入函数
 * @param {WebAssembly.Memory} memory - WebAssembly内存对象
 * @returns {Proxy} - WBG代理对象
 */
export function createWbgProxy(memory) {
    // 工具函数
    function getStringFromWasm0(ptr, len) {
        return new TextDecoder('utf-8').decode(new Uint8Array(memory.buffer, ptr, len));
    }

    return new Proxy({
        __wbindgen_object_drop_ref: function() {},
        __wbg_body_942ea927546a04ba: function() { return document.body; },
        __wbindgen_string_new: function(ptr, len) { return getStringFromWasm0(ptr, len); },
        __wbindgen_throw: function(ptr, len) { throw new Error(getStringFromWasm0(ptr, len)); },
        __wbindgen_number_get: function(obj) { return obj; },
        __wbindgen_number_new: function(val) { return val; },
        __wbindgen_boolean_get: function(obj) { return obj ? 1 : 0; },
        __wbindgen_boolean_new: function(val) { return val !== 0; },
        __wbindgen_is_undefined: function(obj) { return obj === undefined ? 1 : 0; },
        __wbindgen_is_null: function(obj) { return obj === null ? 1 : 0; },
        __wbindgen_document_d249400bd7bd996d: function() { return document; },
        __wbindgen_array_new: function(len) { return new Array(len); },
        __wbindgen_array_get: function(arr, idx) { return arr[idx]; },
        __wbindgen_array_set: function(arr, idx, val) { arr[idx] = val; }
    }, {
        get: function(target, prop) {
            if (prop in target) {
                return target[prop];
            }
            console.warn(`调用了未实现的函数: wbg.${prop}`);
            return function() { return 0; };
        }
    });
}

/**
 * 实例化WASM模块
 * @param {ArrayBuffer} bytes - WASM二进制数据
 * @returns {Promise<object>} - 实例化后的模块导出
 */
export async function instantiateWasmModule(bytes) {
    // 创建内存和表
    const memory = new WebAssembly.Memory({ initial: 10, maximum: 100 });
    const table = new WebAssembly.Table({ initial: 0, maximum: 1000, element: 'anyfunc' });

    // 工具函数
    function getStringFromWasm0(ptr, len) {
        return new TextDecoder('utf-8').decode(new Uint8Array(memory.buffer, ptr, len));
    }

    // 创建导入对象
    const importObject = {
        env: {
            memory: memory,
            table: table,
            getStringFromWasm0: getStringFromWasm0,
            __wbindgen_throw: function(ptr, len) {
                throw new Error(getStringFromWasm0(ptr, len));
            }
        },
        wbg: createWbgProxy(memory)
    };

    try {
        const module = await WebAssembly.instantiate(bytes, importObject);
        console.log('WebAssembly模块实例化成功');
        return module.instance.exports;
    } catch (error) {
        console.error('WebAssembly模块实例化失败:', error);
        throw error;
    }
}

/**
 * 加载JavaScript包装器
 * @param {string} jsWrapperUrl - JS包装器URL
 * @returns {Promise<boolean>} - 是否加载成功
 */
export async function loadJavaScriptWrapper(jsWrapperUrl) {
    try {
        const jsResponse = await fetch(jsWrapperUrl);
        if (!jsResponse.ok) {
            throw new Error(`无法加载JS包装器: ${jsResponse.status} ${jsResponse.statusText}`);
        }

        const jsContentType = jsResponse.headers.get('content-type');
        if (!jsContentType || !jsContentType.includes('javascript')) {
            throw new Error(`JS包装器文件类型不正确: ${jsContentType}`);
        }

        const jsCode = await jsResponse.text();
        if (jsCode.includes('<') && jsCode.includes('>')) {
            throw new Error('JS包装器文件包含HTML内容，不是有效的JavaScript');
        }

        // 创建脚本元素并执行
        const script = document.createElement('script');
        script.textContent = jsCode;
        document.head.appendChild(script);
        console.log('JavaScript包装器已执行');

        // 移除脚本元素（避免污染DOM）
        setTimeout(() => document.head.removeChild(script), 0);

        // 检查是否通过包装器初始化成功
        if (window.wasm && typeof window.wasm === 'object') {
            console.log('WASM模块已通过JavaScript包装器初始化');
            return true;
        }

        // 如果包装器没有直接初始化，尝试调用初始化函数
        if (typeof window.initWasm === 'function') {
            console.log('尝试调用window.initWasm函数');
            try {
                const module = await window.initWasm();
                if (module) {
                    window.wasm = module;
                    console.log('WASM模块已通过手动调用初始化');
                    return true;
                }
            } catch (e) {
                console.error('调用initWasm函数失败:', e);
            }
        }

        // 设置定时器定期检查，最多等待5秒
        return new Promise((resolve) => {
            let checkCount = 0;
            const checkInterval = setInterval(() => {
                checkCount++;
                if (window.wasm && typeof window.wasm === 'object') {
                    clearInterval(checkInterval);
                    console.log('WASM模块已通过JavaScript包装器初始化');
                    resolve(true);
                } else if (checkCount >= 10) {
                    clearInterval(checkInterval);
                    console.error('JavaScript包装器未能初始化WASM模块，超时');
                    resolve(false);
                }
            }, 500);
        });
    } catch (error) {
        console.error('加载JavaScript包装器时出错:', error);
        return false;
    }
}