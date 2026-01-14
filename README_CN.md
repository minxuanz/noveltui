[English](./README.md) | [中文](./README_CN.md)

# Noveltui

一个基于终端界面的小说阅读器，由 [ratatui](https://github.com/ratatui/ratatui) 驱动。

适合配合tmux使用

## 功能特性
- **轻量高效**：二进制文件小，内存占用低
- **章节解析**：通过正则表达式自动检测章节并生成目录
- **书签管理**：添加、删除和管理书签，便于快速定位
- **自动阅读模式**：无需手动翻页，自动滚动阅读
- **在线阅读**：dzstui 支持从网站获取并阅读小说（开发中）

## 安装

### 从源代码构建
```bash
git clone https://github.com/minxuanz/noveltui.git
cd noveltui
cargo build --release
```

`target/release/noveltui` 从本地txt文件阅读  
`target/release/dzstui` 从网站阅读

## 使用方法

```bash
./noveltui 小说路径.txt
```

```bash
./dzstui --url 网站地址
# 例如：
./dzstui --url https://ixdzs8.com/read/508569/p1.html
```
> 提示：目前 dzstui 仅支持 `https://ixdzs8.com/` 网站, 且需安装chrome浏览器

## 支持情况
### 操作系统
- Windows
- Linux
- macOS（未测试）

### 格式与编码
- txt 文件（支持 UTF-8、GBK、GB2312 等编码）

### 章节检测（noveltui）
可通过 `--regex <自定义正则表达式>` 来指定标题匹配规则。
```bash
./noveltui --regex="^(\d+)([\u4e00-\u9fff0-9]+)$" 小说路径.txt
```

### 快捷键（noveltui）

| 按键 | 操作 |
|------|------|
| `q` | 添加书签并退出 |
| `Q` | 直接退出 |
| `j` / `↓` | 向下滚动 |
| `k` / `↑` | 向上滚动 |
| `n` | 下一章 |
| `p` | 上一章 |
| `m` | 切换书签 |
| `M` | 删除所有书签 |
| `空格键` | 切换自动阅读模式 |
| `b` | 打开书签菜单 |
| `Ctrl` + `z` | 暂停程序(unix) |

<div align="center">

![内容界面](./assets/content.png)
*内容界面*

</div>

<div align="center">

![目录界面](./assets/toc.png)
*目录界面*

</div>

<div align="center">

![书签界面](./assets/bookmark.png) 
*书签界面*

</div>

<div align="center">

![tmux中运行](./assets/tmux.png)
*在 tmux 中运行*

</div>

### dzstui（在线阅读，开发中）

<div align="center">

![dzstui界面](./assets/dzstui.png)

</div>