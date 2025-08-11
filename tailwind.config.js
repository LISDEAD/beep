module.exports = {
  content: [
    "./src/**/*.rs",    // 扫描Leptos组件中的类名（view!宏内）
    "./index.html",     // 扫描HTML中的类名
    "./src/**/*.html"   // 如有其他HTML文件也加上
  ],
  theme: { extend: {} }
}