// tauri-loader.js
// 加载Tauri脚本
function loadTauriScript() {
    return new Promise((resolve, reject) => {
        const script = document.createElement('script');
        // 使用CSP允许的CDN
        script.src = 'https://cdn.tauri.app/2.0.0/tauri.iife.js';
        script.onload = function() {
            console.log('Tauri脚本加载完成');
            console.log('window.__TAURI__类型:', typeof window.__TAURI__);
            console.log('window.__TAURI__.invoke类型:', typeof window.__TAURI__?.invoke);
            if (window.__TAURI__ && typeof window.__TAURI__ === 'object' && typeof window.__TAURI__.invoke === 'function') {
                console.log('Tauri invoke函数已准备就绪');
                resolve(window.__TAURI__);
            } else {
                console.error('Tauri加载完成但invoke函数不可用');
                reject(new Error('Tauri加载完成但invoke函数不可用'));
            }
        }
        script.onerror = function() {
            console.error('Tauri脚本加载失败，尝试备用CDN');
            // 尝试使用备用CDN (符合CSP要求)
            script.src = 'https://cdn.tauri.app/2.0.0/tauri.iife.js';
            script.onload = function() {
                console.log('备用CDN加载Tauri脚本成功');
                console.log('window.__TAURI__类型:', typeof window.__TAURI__);
                console.log('window.__TAURI__.invoke类型:', typeof window.__TAURI__?.invoke);
                if (window.__TAURI__ && typeof window.__TAURI__ === 'object' && typeof window.__TAURI__.invoke === 'function') {
                    console.log('Tauri invoke函数已准备就绪');
                    resolve(window.__TAURI__);
                } else {
                    console.error('Tauri加载完成但invoke函数不可用');
                    reject(new Error('Tauri加载完成但invoke函数不可用'));
                }
            };
            script.onerror = function() {
                console.error('备用CDN加载Tauri脚本也失败');
                reject(new Error('无法加载Tauri脚本'));
            };
            // 脚本已经添加到文档中，不需要再次添加
            // document.head.appendChild(script);
        };
        document.head.appendChild(script);
    });
}

// 导出函数，供其他文件使用
 export { loadTauriScript };