# Nonebot-rs
fork自这位大佬的[仓库](https://github.com/abrahum/nonebot-rs)，对其进行了删改，可以用作llonebot的事件处理器框架。

很显然这和具有强大功能的nonebot不是很搭得上边...不过对于一些简单的QQbot项目来说已经足够了。

## 使用方法

先安装好llonebot [安装教程](https://llonebot.com/guide/getting-started)

clone仓库后，在项目根目录下创建配置文件`Nonebotrs.toml`（或者直接运行`cargo run --bin bot_example`，项目也会创建一个同样的配置文件）

```toml
[global]                     # 全局设置
debug = true                 # 开启 debug log
superusers = ["33222444"]    # 全局管理员QQ
nicknames = ["nickname"]     # 全局 Bot 昵称
command_starts = ["/"]       # 全局命令起始符

[ws_server]                  # 反向 WS 服务器
host = "127.0.0.1"           # 监听 host
port = 8088                  # 监听 port
access_token = "AccessToken" # 连接鉴权使用

[bots.BotID1111]             # Bot 设置
superusers = ["334455667"]   # 管理员QQ
nicknames = ["nickname"]     # Bot 昵称
command_starts = ["/"]       # 命令起始符
ws_server = "server address" # 正向 WS 服务器地址（缺省不启用正向 WS 连接）
access_token = "AccessToken" # 连接鉴权使用,在llonebot那儿配置

# [bots.BotID2222]             # 可以有多个bot,BotID不同即可
# superusers = ["123456"]      # 管理员QQ
# ......

```
然后运行`cargo run --bin bot_example`即可启动一个示例bot。

PS: 推荐使用正向ws连接，反向ws连接仍在测试中...

使用正向连接的方法：

1. 在llonebot中启用正向ws服务器，设置好端口号和access_token（以8080为例），消息格式选择消息段
2. 编辑llonebot的docker-compose.yml文件，在ports下添加一行` - “8080:8080”`
3. 在Nonebotrs.toml中设置好ws_server和access_token，如果docker和框架跑在同一台机子，则ws_server为ws://127.0.0.1:8080
4. 重启llonebot容器: `sudo docker compose down` `sudo docker compose up -d`
5. 运行bot程序: `cargo run --bin bot_example`

## 编写自己的bot

在项目根目录下运行`cargo new 你bot的名字`，这里以mybot为例子。

在`Cargo.toml`中添加对nonebot_rs的依赖：

```toml
[package]
name = "mybot"
version = "0.1.0"
edition = "2024"

[dependencies]
nonebot_rs = {path = "../nonebot_rs"} # 最主要的依赖

[dependencies.tracing] # 用来写log的
version = "0.1"
features = ["std"]

```

在`mybot/src/main.rs`中编写bot代码：

```rust
use nonebot_rs;
fn main() {
    let mut nb = nonebot_rs::Nonebot::new();
    nb.run()
}

```
随后在项目根目录下的`Cargo.toml`内添加`mybot`到`workspace`的`member`里
```toml
[workspace]
members = [ 
    "bot_example",
    "nonebot_rs",
    "mybot",    # 这儿
]

```

最后运行`cargo run --bin mybot`即可，你会得到一个只能接收消息，但不响应任何事件的bot。

## 如何编写事件处理器？
可以阅读`bot_example`下的代码，这里面已经涵盖了大多事件的编写方法
