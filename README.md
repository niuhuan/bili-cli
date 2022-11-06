bili-cli
=========

哔哩哔哩视频命令行下载器

## 用户界面

![](images/down_bv.gif)

## 实现功能

- [x] 用户
  - [x] 登录
  - [x] 个人信息
- [x] 视频下载
  - [x] 高清视频下载并合并
  - [x] BV下载
  - [x] EP/SS下载
  - [x] 来自手机的短视频/短链接
  - [x] 断点续传
    - [x] dash模式 (-r 参数)
  - [x] 集合下载时选择EP

## 如何使用

1. 将ffmpeg命令添加到PATH (合并必须使用)
2. 运行本软件

```shell

# 登录你的账号
cargo login

# 在控制台打印二维码
cargo login -c

# 打印下载帮助
cargo run -- down -h

# 下载合集时如果出现异常可以尝试增加ss参数
cargo run -- down --ss

```
