# 文档样式故障排除

## § 符号仍然显示？

如果你看到 "Modules§" 而不是 "Modules"，请尝试以下方法：

### 方法 1: 强制刷新浏览器
- **macOS**: `Cmd + Shift + R`
- **Windows/Linux**: `Ctrl + Shift + R`
- 或者清除浏览器缓存

### 方法 2: 使用隐私模式
```bash
# 重新生成文档
make docs

# 然后在浏览器中使用隐私/无痕模式打开
# Safari: Cmd + Shift + N
# Chrome: Cmd + Shift + N
# Firefox: Cmd + Shift + P
```

### 方法 3: 手动验证 CSS
检查 CSS 是否正确加载：
```bash
# 查看生成的 HTML 是否包含内联样式
grep "\.anchor" target/doc/vex/index.html

# 查看 custom.css 是否存在
ls -la target/doc/custom.css

# 查看 CSS 内容
head -80 target/doc/custom.css | grep -A 10 "anchor"
```

### 方法 4: 完全重新生成
```bash
# 清理旧文件
rm -rf target/doc

# 重新生成
make docs
```

## 验证样式是否生效

打开文档后，你应该看到：
- ✅ "Modules" 而不是 "Modules§"
- ✅ "Structs" 而不是 "Structs§"
- ✅ "Functions" 而不是 "Functions§"
- ✅ 清爽的标题，没有锚点符号

## 技术细节

我们使用了多种方法来隐藏 § 符号：

1. **内联 CSS** (在 `<head>` 中)
```css
.anchor {
    display: none !important;
    visibility: hidden !important;
    opacity: 0 !important;
}
```

2. **外部 CSS** (custom.css)
```css
.anchor,
a.anchor,
h2 .anchor,
h3 .anchor,
h4 .anchor,
.section-header .anchor {
    display: none !important;
    visibility: hidden !important;
    opacity: 0 !important;
    width: 0 !important;
    height: 0 !important;
    font-size: 0 !important;
}
```

3. **CSS 版本号** (强制刷新)
```html
<link rel="stylesheet" href="../custom.css?v=2">
```

## 仍然有问题？

如果以上方法都不行，可能是：
1. 浏览器不支持某些 CSS 属性
2. 浏览器扩展干扰了样式
3. 需要更新浏览器版本

尝试：
- 禁用浏览器扩展
- 使用不同的浏览器
- 检查浏览器控制台是否有 CSS 错误
