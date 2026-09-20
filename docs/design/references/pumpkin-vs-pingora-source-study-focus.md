# Pumpkin-MC / Pumpkin vs Cloudflare / Pingora：源码研究重点方向

## 1. Pumpkin-MC / Pumpkin

> **核心目标：学习“大型 Rust Application 如何组织和演进”**

### 重点研究

| 研究方向 | 重点关注 |
|---|---|
| **Workspace / Crate 组织** | Workspace 如何拆分、Crate 职责边界、依赖方向 |
| **领域模型设计** | Domain Type、Entity、组件、状态模型如何组织 |
| **模块边界** | 不同业务模块如何解耦，公共能力放在哪里 |
| **Trait 设计** | Trait 如何抽象业务行为、扩展点如何设计 |
| **ECS / 数据驱动设计** | ECS 的组织方式、系统与数据之间的关系 |
| **异步架构** | Tokio、Task、Channel、异步任务之间如何协作 |
| **事件 / 消息机制** | Event、Command、消息传递以及模块间通信 |
| **配置与启动流程** | Application Bootstrap、配置加载、Runtime 初始化 |
| **错误处理** | Domain Error、Infrastructure Error 如何分层 |
| **测试组织** | Unit Test、Integration Test、测试辅助代码如何组织 |

### 推荐源码阅读路线

```text
Workspace
    ↓
Crate / Package
    ↓
Application Entry
    ↓
Core Domain
    ↓
Domain Model
    ↓
Trait / Component
    ↓
Event / System
    ↓
Async Runtime
    ↓
完整业务流程
```

### 最值得带走的能力

```text
大型 Rust Application
        ↓
领域拆分
        ↓
Crate 拆分
        ↓
模块边界
        ↓
依赖控制
        ↓
业务流程组织
```

**关键词：**

> `Workspace` · `Domain` · `Crate` · `Module` · `ECS` · `Trait` · `Event` · `Async`

---

# 2. Cloudflare / Pingora

> **核心目标：学习“生产级 Rust Infrastructure / Framework 如何设计”**

### 重点研究

| 研究方向 | 重点关注 |
|---|---|
| **Workspace / Crate 组织** | Core、Proxy、Network 等能力如何拆分 |
| **Framework 分层** | Core 能力与具体 Application 如何解耦 |
| **Trait / Extension Point** | 如何设计可扩展 API、Hook、生命周期接口 |
| **依赖方向** | Framework → Infrastructure → Runtime 的依赖关系 |
| **Runtime 架构** | Tokio、Task、Future、Runtime 生命周期 |
| **Network Abstraction** | TCP、Connection、Listener 等抽象 |
| **HTTP 抽象** | Request、Response、Session、Protocol 层次 |
| **Connection 管理** | Connection Pool、Reuse、Timeout、Shutdown |
| **Proxy Pipeline** | Request → Proxy → Upstream → Response |
| **错误 / Retry / Timeout** | 基础设施场景下的可靠性设计 |
| **生命周期管理** | Graceful Shutdown、连接生命周期、资源释放 |
| **可扩展性设计** | Framework 如何让上层业务进行定制 |

### 推荐源码阅读路线

```text
Workspace
    ↓
Core
    ↓
Runtime
    ↓
Network
    ↓
Connection
    ↓
HTTP
    ↓
Proxy
    ↓
Trait / Extension Point
    ↓
完整 Request Pipeline
```

尤其建议重点追踪：

```text
Incoming Connection
        ↓
      Accept
        ↓
    Connection
        ↓
      HTTP
        ↓
     Request
        ↓
      Proxy
        ↓
     Upstream
        ↓
    Response
        ↓
Connection Reuse
```

### 最值得带走的能力

```text
Infrastructure
      ↓
    Core
      ↓
   Trait API
      ↓
Extension Point
      ↓
 Application
```

**关键词：**

> `Framework` · `Trait` · `Extension` · `Runtime` · `Network` · `Connection` · `Proxy` · `Async` · `Reliability`

---

# 3. 两个仓库应该怎么分工学习

```text
                 Rust 工程能力
                      │
          ┌───────────┴───────────┐
          ↓                       ↓
      Pumpkin                  Pingora
          │                       │
          ↓                       ↓
  Application Architecture   Infrastructure Architecture
          │                       │
          ↓                       ↓
    Domain Modeling          Framework Design
    Crate Organization       Trait / Extension
    Module Boundary           Runtime
    Event / ECS               Network
    Business Flow             Connection
          │                       │
          └───────────┬───────────┘
                      ↓
             自己的 Rust Template
```

## 4. 最终研究目标

不要以**“把两个仓库代码读完”**为目标，而是分别回答下面这些问题：

### Pumpkin：回答

```text
一个大型 Rust Application
应该如何拆？

一个 Domain
应该放在哪里？

一个功能
应该属于哪个 Crate？

Crate 之间
应该如何控制依赖？

业务模块之间
应该如何通信？
```

### Pingora：回答

```text
一个 Rust Framework
应该如何设计？

Core 与 Application
如何解耦？

Trait 应该在哪里定义？

Extension Point
应该如何设计？

Runtime / Network / HTTP
如何分层？

如何让上层业务
在不修改 Core 的情况下扩展？
```

---

# 5. 对 Rust Template 的最终沉淀

建议最后不要直接复制 Pumpkin 或 Pingora 的目录，而是提炼成自己的设计原则：

```text
Pumpkin
   ↓
学习 Application Architecture
   ↓
Crate / Domain / Module / Event
   │
   │
   ├──────────────┐
   │              │
   ↓              ↓
Rust Starter Template
   ↑              ↑
   │              │
Pingora
   ↓
学习 Infrastructure Architecture
   ↓
Core / Trait / Extension / Runtime
```

最终重点沉淀为 **6 个架构问题**：

1. **Crate 怎么拆？**
2. **Module 边界怎么定？**
3. **依赖方向怎么控制？**
4. **Core 与业务怎么解耦？**
5. **Trait / Extension Point 怎么设计？**
6. **Runtime / Infrastructure 能力怎么抽象？**

> **Pumpkin 重点学“怎么组织一个大型 Rust 应用”，Pingora 重点学“怎么设计一个可扩展 Rust 基础设施框架”。**
