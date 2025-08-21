// utils.js
// 通用工具函数

// 验证模块路径是否有效
function validateModulePath(modulePath) {
    // 简单验证路径格式
    if (!path || typeof path !== 'string') {
        return false;
    }
    // 检查是否是有效的WASM或JS文件路径
    return path.endsWith('.wasm') || path.endsWith('.js');
}

// 检查响应状态
function checkResponseStatus(response, path) {
    if (!response.ok) {
        throw new Error(`请求失败，状态码: ${response.status}`);
    }
    return response;
}

// 检查内容类型
function checkContentType(response) {
    const contentType = response.headers.get('content-type');
    console.log('文件类型:', contentType);
    return contentType;
}

// 验证WASM二进制文件
function validateWasmBinary(bytes) {
    console.log('WASM二进制文件加载成功，大小:', bytes.byteLength);
    // 检查前几个字节是否是WASM魔术数字(00 61 73 6d)
    const view = new DataView(bytes);
    const magic = view.getUint32(0, true);
    if (magic !== 0x6d736100) {
        console.error('WASM文件格式错误: 不是有效的WebAssembly二进制文件');
        throw new Error('WASM文件格式错误');
    }
    return bytes;
}

// 工具函数：从WASM内存中获取字符串
function getStringFromWasm0(memory, ptr, len) {
    return new TextDecoder('utf-8').decode(new Uint8Array(memory.buffer, ptr, len));
}