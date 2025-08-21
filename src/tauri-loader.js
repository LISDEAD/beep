// tauri-loader.js
// 加载Tauri脚本
function loadTauriScript() {
    return new Promise((resolve, reject) => {
        const script = document.createElement('script');
        // 使用与package.json一致的版本
        script.src = 'https://unpkg.com/@tauri-apps/api@2.7.0/dist/tauri.iife.js';
        script.onload = function() {
            console.log('Tauri脚本加载完成');
            resolve(window.__TAURI__);
        };
        script.onerror = function() {
            console.error('Tauri脚本加载失败，尝试备用CDN');
            // 尝试使用备用CDN
            script.src = 'https://cdn.jsdelivr.net/npm/@tauri-apps/api@2.7.0/dist/tauri.iife.js';
            script.onload = function() {
                console.log('备用CDN加载Tauri脚本成功');
                resolve(window.__TAURI__);
            };
            script.onerror = function() {
                console.error('备用CDN加载Tauri脚本也失败');
                reject(new Error('无法加载Tauri脚本'));
            };
            document.head.appendChild(script);
        };
        document.head.appendChild(script);
    });
}