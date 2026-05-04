# DNS Manager — Windows 11 Tauri 桌面应用计划

## 上下文

构建一个 Windows 11 桌面应用，使用 Tauri 打包，通过 API 聚合管理 DNSPod、Cloudflare、阿里云 AliDNS 上的域名和 DNS 记录。利用 `.trae` skills 目录中的开发流水线（requirement-analyst → system-architect → task-planner → spec-coder）。

## 技术栈

| 层 | 技术 | 说明 |
|---|---|---|
| 桌面壳 | Tauri 2.x | 原生 Windows 11 集成 |
| 前端框架 | React 18 + TypeScript + Vite | SPA 架构 |
| UI 组件 | shadcn/ui + Tailwind CSS + lucide-react | 深色/浅色主题 |
| 后端语言 | Rust (stable) + tokio | Tauri 原生后端 |
| HTTP 客户端 | reqwest | 连接池、超时、TLS |
| 序列化 | serde + serde_json | JSON 映射 |
| 异步 trait | async-trait | async fn in trait |
| 签名 | hmac, sha2, hex, base64 | DNSPod v3 / AliDNS 签名 |
| 凭据存储 | windows-rs → Windows Credential Manager | DPAPI 加密 |

## 项目结构

```
dns-manager/
├── src-tauri/                    # Rust 后端
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   ├── src/
│   │   ├── main.rs               # 入口 + 窗口设置
│   │   ├── lib.rs                # Tauri app builder, 命令注册
│   │   ├── providers/
│   │   │   ├── mod.rs            # DnsProvider trait + 工厂函数
│   │   │   ├── dnspod.rs         # 腾讯云 DNSPod v3 API
│   │   │   ├── cloudflare.rs     # Cloudflare REST v4 API
│   │   │   └── alidns.rs         # 阿里云 AliDNS API
│   │   ├── models/
│   │   │   ├── mod.rs
│   │   │   ├── domain.rs         # Domain, DomainSummary
│   │   │   ├── record.rs         # DnsRecord, RecordType, Create/UpdateRequest
│   │   │   ├── credential.rs     # ProviderCredential, CredentialInput, CredentialSecretData
│   │   │   └── search.rs         # SearchQuery, SearchResult
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── credentials.rs    # 凭据 CRUD
│   │   │   ├── domains.rs        # 跨提供商域名聚合
│   │   │   ├── records.rs        # DNS 记录 CRUD
│   │   │   └── export.rs         # Zone 文件导出
│   │   ├── security/
│   │   │   ├── mod.rs
│   │   │   └── credential_manager.rs  # Windows 凭据管理器封装
│   │   ├── error.rs              # 统一错误类型
│   │   └── retry.rs              # 指数退避 + 速率限制
├── src/                          # React 前端
│   ├── main.tsx
│   ├── App.tsx                   # 路由 + Provider 包装
│   ├── index.css                 # Tailwind + CSS 变量
│   ├── routes/
│   │   ├── Dashboard.tsx
│   │   ├── Credentials.tsx
│   │   ├── Domains.tsx
│   │   ├── DomainDetail.tsx
│   │   ├── Search.tsx
│   │   └── Settings.tsx
│   ├── components/
│   │   ├── layout/               # AppShell, Sidebar, Header
│   │   ├── credentials/          # CredentialList, CredentialForm, CredentialCard
│   │   ├── domains/              # DomainList, DomainCard, DomainStats
│   │   ├── records/              # RecordTable, RecordForm, RecordRow
│   │   ├── search/               # GlobalSearch, SearchResults
│   │   └── common/               # ProviderBadge, ConfirmDialog, LoadingSpinner, ErrorAlert
│   ├── hooks/                    # useCredentials, useDomains, useRecords, useSearch, useTheme
│   ├── context/                  # AppContext, ThemeContext
│   ├── lib/                      # tauri.ts (typed invoke wrappers), utils.ts
│   └── types/                    # domain.ts, record.ts, credential.ts, search.ts
├── package.json
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.ts
└── index.html
```

## 核心架构

### Provider Trait（Rust — 架构基石）

定义在 `src-tauri/src/providers/mod.rs`，使用 `#[async_trait]` 宏：

```rust
#[async_trait]
pub trait DnsProvider {
    async fn list_domains(&self) -> Result<Vec<Domain>, ProviderError>;
    async fn get_domain(&self, domain_id: &str) -> Result<Domain, ProviderError>;
    async fn list_records(&self, domain_id: &str) -> Result<Vec<DnsRecord>, ProviderError>;
    async fn create_record(&self, domain_id: &str, record: &CreateRecordRequest) -> Result<DnsRecord, ProviderError>;
    async fn update_record(&self, domain_id: &str, record_id: &str, record: &UpdateRecordRequest) -> Result<DnsRecord, ProviderError>;
    async fn delete_record(&self, domain_id: &str, record_id: &str) -> Result<(), ProviderError>;
    async fn search_records(&self, domain_id: &str, query: &str) -> Result<Vec<DnsRecord>, ProviderError>;
    async fn export_zone(&self, domain_id: &str) -> Result<String, ProviderError>;
    fn provider_type(&self) -> ProviderType;
}
```

### 三大提供商认证方式

| 提供商 | API 地址 | 认证方式 | 复杂度 |
|--------|----------|----------|--------|
| Cloudflare | api.cloudflare.com/client/v4 | Bearer Token (最简单) | 低 |
| DNSPod | dnspod.tencentcloudapi.com | TC3-HMAC-SHA256 签名 v3 | 高 |
| AliDNS | alidns.aliyuncs.com | HMAC-SHA1 查询参数签名 | 中 |

### IPC 通信流

```
React Frontend → invoke("command", args) → Tauri IPC → Rust Command Handler
→ Provider Factory → DnsProvider impl → reqwest → DNS API
```

凭据永远不会传递到前端：前端只看到 `ProviderCredential`（id + label + provider_type），秘密数据仅存在于 Rust 后端内存中。

### 安全设计

- Windows Credential Manager 存储秘密（DPAPI 加密绑定用户）
- 每条凭据独立存储（target: `dns-manager/{uuid}`）
- `CredentialSecretData` 实现 `Debug` 时遮盖所有字段
- 错误消息剥离凭据数据
- Tauri CSP 头部限制 `default-src 'self'`
- 所有 API 调用通过 HTTPS（reqwest 默认 TLS + 证书验证）

### 跨提供商聚合

`list_domains` / `search_records` 使用 `tokio::join!` 并发查询所有已配置的提供商，部分失败不影响整体结果，最终合并返回。

## 路由设计

| 路由 | 页面 | 功能 |
|------|------|------|
| `/` | Dashboard | 提供商统计卡片、域名统计、快捷操作 |
| `/credentials` | Credentials | 凭据列表、添加/编辑/删除/测试连接 |
| `/domains` | Domains | 按提供商分组的聚合域名列表 |
| `/domains/:provider/:domainId` | DomainDetail | DNS 记录 CRUD（表格 + 表单对话框） |
| `/search` | Search | 全局跨提供商搜索 |
| `/settings` | Settings | 主题切换、关于 |

## 前端组件树

```
<App>
  <ThemeProvider>
    <AppProvider>
      <AppShell>
        <Sidebar />                ← 导航链接 + 提供商连接状态
        <Header />                 ← 面包屑 + 主题切换 + 搜索触发
        <Outlet />                 ← 路由内容
          ├── Dashboard（统计卡片）
          ├── Credentials（凭据管理）
          ├── Domains（域名列表）
          ├── DomainDetail（记录 CRUD）
          ├── Search（全局搜索）
          └── Settings（主题设置）
      </AppShell>
    </AppProvider>
  </ThemeProvider>
</App>
```

## 实施序列

### Phase 1: 脚手架与基础设施
1. 初始化 Tauri 2.x + React + TypeScript + Vite 项目
2. 配置 Tailwind CSS + shadcn/ui
3. 搭建 Rust 项目模块结构（models/, providers/, commands/, security/）
4. 实现 `error.rs`、`retry.rs`、所有数据模型（models/）
5. 实现 `security/credential_manager.rs`（Windows 凭据管理器集成）
6. 构建 AppShell、Sidebar、Header 布局组件
7. 设置路由、ThemeContext、深色/浅色主题

### Phase 2: 凭据管理
8. 实现凭据 Tauri commands（save, list, delete, test）
9. 构建 Credentials 页面（CredentialList + CredentialForm）
10. 接入 AppContext 全局凭据状态

### Phase 3: 提供商实现（逐个进行）
11. 实现 DnsProvider trait + Cloudflare（最简单 API）
12. 构建 Domains 页面 + DomainDetail 页面
13. 实现 DNSPod（签名 v3 — 最复杂认证）
14. 实现 AliDNS（查询参数签名）
15. 构建 RecordTable + RecordForm 完整 CRUD

### Phase 4: 搜索与导出
16. 实现跨提供商 search_records 命令
17. 构建 Search 页面（GlobalSearch + SearchResults）
18. 实现 zone file 导出
19. 添加批量删除操作

### Phase 5: 打磨
20. Dashboard 统计页面
21. 加载骨架屏、空状态、错误边界
22. 键盘快捷键、无障碍审计
23. Windows 安装器配置（MSI 打包）

## 关键文件（实施重点）

- **`src-tauri/src/providers/mod.rs`** — DnsProvider trait 定义 + 工厂函数（最重要的架构决策）
- **`src-tauri/src/security/credential_manager.rs`** — Windows 凭据管理器封装（安全基础）
- **`src-tauri/src/commands/records.rs`** — DNS 记录 CRUD 命令（最复杂命令处理器）
- **`src/components/layout/AppShell.tsx`** — 应用壳布局（整体 UI 框架）
- **`src/lib/tauri.ts`** — 类型化 invoke 封装（Rust ↔ TypeScript IPC 契约边界）

## 验证方式

1. **构建验证**：`cargo build` 在 `src-tauri/` 中无错误编译
2. **前端验证**：`npm run build` 通过 Vite 构建无错误
3. **集成测试**：每个提供商实现完成后，使用真实的测试凭据验证 list_domains → list_records → CRUD 流程
4. **凭据安全验证**：确认 `get_credentials` 返回的数据不包含任何 secret_id/secret_key/api_token
5. **错误处理验证**：模拟网络断开、无效凭据、速率限制场景
6. **跨提供商验证**：同时配置 3 个提供商，验证聚合域名列表和跨提供商搜索
7. **打包验证**：`npm run tauri build` 生成 Windows MSI 安装包并测试安装运行
