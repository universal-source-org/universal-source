# Universal Source

**Write once. Run everywhere. — 编写一次，随处运行。**

**面向可移植内容源的跨平台规范与运行时。**

[English](./README.md) · [v0.1 规范草案](./spec/README.md) · [最小 JSON 源](./examples/json/minimal/README.md)

Universal Source 的目标是让同一份内容源在 **Android、Android TV、iOS、iPadOS、tvOS、macOS、Windows 和 Linux** 上运行。这是项目目标，并不代表目前已支持这些平台。仓库当前提供规范基础和示例，尚未实现参考运行时。

## 核心契约

**标准定义行为和接口，不限定实现语言。** 内容源提供少量操作：

| 操作 | 用途 |
| --- | --- |
| `home` | 列出入口分类和内容。 |
| `category` | 按分类分页列出内容。 |
| `search` | 按查询词分页查找内容。 |
| `detail` | 提供内容详情和可播放条目。 |
| `play` | 将可播放条目解析为媒体资源。 |

宿主提供受控的 HTTP、Cookie、存储、HTML 解析、JSON、加密、URL 工具和日志服务。源需要声明支持的操作、必需的宿主服务以及申请的权限。声明本身不等于授权。

```text
声明式 JSON 源或 JavaScript 源
              ↓
   Universal Source 规范
              ↓
        符合规范的运行时
              ↓
      移动端 / 电视 / 桌面宿主
```

规范独立于参考运行时存在。只要保持契约规定的行为，实现可以使用不同语言和平台。

## v0.1 范围与状态

当前是**实验性的 v0.1 草案**，不是正式发布，也不承诺生产环境兼容性。草案定义清单、五个源操作、通用数据和错误规则、宿主能力边界、生命周期以及最小静态声明式格式。JavaScript 是计划支持的源引擎；在宣称运行时实现可互操作之前，仍需明确其执行绑定和各宿主服务的详细约定。

架构方向是：

1. 平台中立的规范。
2. 用于简单场景的声明式 JSON，首先支持静态数据。
3. 用于复杂逻辑的 JavaScript。
4. 面向已有生态的兼容适配器。

未来可能使用 Rust 实现共享运行时核心，但 Rust 不属于标准要求。WebAssembly 留待后续探索；它不是 v0.1 引擎，本仓库也未实现它。

v0.1 不包含推荐系统、账号、同步、DRM、字幕、评论、弹幕、下载、播放器 UI 或平台专属应用。首个版本也不引入通用抓取语言、原生插件或应用专属 API。

## 已有生态

TVBox、FongMi、drpy、XBPQ 和 XYQ 是既有生态和灵感来源。它们提供有价值的兼容案例，但不决定本项目的身份或核心 API。导入器和适配器应在合理可行时复用已有源，并明确报告不支持的行为。Android API、Java/JAR 加载和旧格式约定应放在兼容层，而不是新标准中。目前尚未实现适配器，也未作出兼容性保证。

## 仓库结构

项目保持为单一 monorepo：

| 目录 | 用途 |
| --- | --- |
| [`spec/`](./spec/README.md) | 独立规范、清单 Schema 和 RFC。 |
| `runtime/` | 为未来参考运行时预留。 |
| [`examples/`](./examples/json/minimal/README.md) | 展示契约的小型源示例。 |
| [`conformance/`](./conformance/README.md) | 清单 Schema 验证和测试样本；源操作一致性检查尚待实现。 |
| [`docs/`](./docs/PROJECT_CHARTER.md) | 项目章程、设计原则、当前架构、路线图、决策记录和任务计划。 |

建议先阅读[规范索引](./spec/README.md)，再查看[示例清单和源](./examples/json/minimal/README.md)。示例媒体地址只是占位符，阅读示例不需要网络访问。目前没有可执行示例的运行时命令。

## 参与贡献

请先阅读 [CONTRIBUTING.md](./CONTRIBUTING.md) 和 [AGENTS.md](./AGENTS.md)，了解仓库工作流程和上下文索引。[路线图](./docs/ROADMAP.md) 记录当前阶段及其完成标准。

请使用 [RFC 模板](./spec/rfcs/0000-template.md) 提出契约变更，说明具体场景、可移植行为、迁移影响，以及独立实现如何验证一致性。小型示例和明确的旧生态兼容问题，比推测性的抽象更有价值。
