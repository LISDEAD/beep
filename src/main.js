// src/main.js
// 应用程序入口点

// 导入所需函数
import { loadTauriScript } from './tauri-loader.js';
import { initWebAssembly } from './wasm-loader.js';

/**
 * 初始化应用程序
 */
async function initApp() {
    try {
        // 加载Tauri脚本
        console.log('开始加载Tauri脚本...');
        let tauri;
        try {
            tauri = await loadTauriScript();
            console.log('Tauri已加载', tauri);
            console.log('Tauri对象类型:', typeof tauri);
            console.log('Tauri.invoke类型:', typeof tauri?.invoke);
        } catch (error) {
            console.error('加载Tauri脚本时出错:', error);
        }

        // 确保invoke函数存在且是函数类型
        if (tauri && typeof tauri === 'object' && typeof tauri.invoke === 'function') {
            console.log('Tauri invoke函数已准备就绪');
            // 将Tauri对象挂载到window，供WebAssembly访问
            window.__TAURI__ = tauri;
            console.log('Tauri对象已挂载到window');
            
            // 使用轮询机制确保Tauri完全初始化
            let pollCount = 0;
            const maxPollCount = 10; // 最多轮询10次
            const pollInterval = 200; // 每200毫秒轮询一次
            
            const pollTauriInit = () => {
                pollCount++;
                
                console.log(`轮询Tauri初始化状态 (${pollCount}/${maxPollCount})`);
                console.log('window.__TAURI__类型:', typeof window.__TAURI__);
                console.log('window.__TAURI__.invoke类型:', typeof window.__TAURI__?.invoke);
                
                if (window.__TAURI__ && typeof window.__TAURI__ === 'object' && typeof window.__TAURI__.invoke === 'function') {
                    console.log('Tauri已完全初始化，开始加载WebAssembly');
                    initWebAssembly().then(() => {
                        console.log('WebAssembly初始化完成');
                    }).catch(error => {
                        console.error('WebAssembly初始化失败:', error);
                    });
                } else if (pollCount >= maxPollCount) {
                    console.error('超过最大轮询次数，Tauri仍未完全初始化');
                } else {
                    setTimeout(pollTauriInit, pollInterval);
                }
            };
            
            // 开始轮询
            pollTauriInit();
        } else {
            console.error('Tauri invoke函数不可用或不是函数类型');
        }
    } catch (error) {
        console.error('应用初始化失败:', error);
    }
}

// 等待页面加载完成
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initApp);
} else {
    // 如果页面已经加载完成，直接初始化
    initApp();
}