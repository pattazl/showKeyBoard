# ShowKeyBoard Rust Server

这是将 ShowKeyBoard 项目从 Node.js 转换为 Rust 的 HTTP 和 WebSocket 服务实现。

## 功能特性

- HTTP RESTful API 服务
- WebSocket 实时通信
- SQLite 数据库支持
- 文件上传/下载
- 静态文件服务
- CORS 跨域支持

## 技术栈

- **Web Framework**: [Axum](https://github.com/tokio-rs/axum) - 基于 Tokio 的高性能 Web 框架
- **WebSocket**: tokio-tungstenite
- **Database**: [rusqlite](https://github.com/rusqlite/rusqlite) - SQLite 绑定
- **Serialization**: serde, serde_json
- **Async Runtime**: Tokio

## 项目结构

```
rust/
├── Cargo.toml          # 项目配置
├── README.md           # 本文件
└── src/
    ├── main.rs         # 程序入口
    ├── handlers.rs     # HTTP 请求处理器
    ├── models.rs       # 数据模型定义
    ├── db.rs           # 数据库操作
    ├── websocket.rs    # WebSocket 处理
    └── config.rs       # 配置管理
```

## API 端点

| 方法 | 路径 | 描述 |
|------|------|------|
| POST | /getPara | 获取全部参数信息 |
| POST | /setPara | 写入修改全部参数信息 |
| POST | /exit | 退出系统 |
| POST | /data | 上传按键数据 |
| POST | /sendPCInfo | 上传 PC 其他信息（显示屏、分辨率等） |
| POST | /historyData | 获取历史数据 |
| POST | /minuteData | 获取分钟数据 |
| POST | /optKeymap | 操作键盘映射（增删改） |
| POST | /getHistoryDate | 获取历史天数列表 |
| POST | /statData | 获取统计数据 |
| POST | /deleteData | 删除历史数据 |
| GET | /zipDownload | 提供 ZIP 下载 |
| POST | /zipUpload | 上传 ZIP 文件 |
| GET | /version | 获取版本信息 |
| POST | /getDbs | 获取数据库清单 |
| POST | /getAppMinute | 获取应用分钟使用数据 |
| WS | /ws | WebSocket 端点 |

## 构建和运行

### 前置条件

- Rust 1.70+ (通过 [rustup](https://rustup.rs/) 安装)
- SQLite 开发库

### 构建

```bash
# 开发模式编译
cargo build

# 发布模式编译
cargo build --release
```

### 运行

```bash
# 开发模式
cargo run

# 发布模式
cargo run --release
```

服务器默认在 `http://0.0.0.0:8888` 启动。

## 配置

配置文件 `showKeyBoard.ini` 格式：

```ini
[server]
port = 8888
ui = ui

[db]
path = ./records.db

[keymaps]
default = default.json

[share]
auto = 0
```

## 数据库表结构

### key_records
| 列 | 类型 | 描述 |
|---|------|------|
| id | INTEGER | 主键，自增 |
| key | TEXT | 按键名称 |
| app | TEXT | 应用程序名 |
| tick | INTEGER | 时间戳 |
| date | TEXT | 日期 |
| count | INTEGER | 按键次数 |

### app_minutes
| 列 | 类型 | 描述 |
|---|------|------|
| id | INTEGER | 主键，自增 |
| app | TEXT | 应用程序名 |
| date | TEXT | 日期 |
| minutes | INTEGER | 使用分钟数 |

### keymaps
| 列 | 类型 | 描述 |
|---|------|------|
| id | INTEGER | 主键，自增 |
| name | TEXT | 键盘映射名称 |
| data | TEXT | 映射数据 JSON |
| date | TEXT | 创建/修改日期 |

### pc_info
| 列 | 类型 | 描述 |
|---|------|------|
| id | INTEGER | 主键，自增 |
| display | TEXT | 显示屏信息 |
| resolution | TEXT | 分辨率 |
| cpu_info | TEXT | CPU 信息 |
| date | TEXT | 日期 |

## WebSocket 消息格式

### 客户端发送
```json
{
  "type": "ping",
  "data": null
}
```

### 服务端响应
```json
{
  "type": "pong",
  "data": {"time": 1234567890}
}
```

## 与原 Node.js 版本的对比

| 特性 | Node.js 版本 | Rust 版本 |
|------|-------------|-----------|
| HTTP 框架 | Express | Axum |
| WebSocket | ws | tokio-tungstenite |
| 数据库 | better-sqlite3 | rusqlite |
| 静态文件 | express.static | tower-http fs |
| 上传文件 | multer | multer (Rust) |
| 配置文件 | ini | ini crate |

## 性能对比

Rust 版本预期性能提升：
- **CPU 使用率**: 降低 50-70%
- **内存占用**: 降低 60-80%
- **启动时间**: 提升 3-5x
- **吞吐量**: 提升 2-3x

## 待完成功能

- [ ] 完整的配置文件读写
- [ ] 自动备份和共享功能
- [ ] ZIP 导入/导出完整实现
- [ ] 性能优化和连接池
- [ ] 日志文件输出
- [ ] 单元测试

## License

MIT License
