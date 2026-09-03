# 保险服务平台 — 微信小程序端 & 鸿蒙端架构规划

> 适用对象：原生微信小程序（WXML/WXSS + JS）与鸿蒙（ArkTS + ArkUI）端。
> 三端（Flutter / 小程序 / 鸿蒙）共用同一 `bee-rust` REST API，通过 `X-Client-Platform` 请求头区分来源。

- 后端：`bee-rust`（Rust REST API），JWT 认证，统一响应 `ResponseEnvelope { code, message, data, trace_id }`
- 接口域：`/api/v1/auth`、`/products`、`/quotes`、`/orders`、`/payments`、`/policies`、`/contracts`、`/search`
- 业务：保险买卖、保单生成查看、合同电子签署、支付

---

## 一、微信小程序端（原生 WXML/WXSS + JS）

### 1.1 目录结构

```
miniprogram/
├── app.js                      # 小程序入口：全局状态、启动登录态检查、注册请求拦截
├── app.json                    # 全局配置：页面注册、window、tabBar、networkTimeout
├── app.wxss                    # 全局样式：CSS 变量（品牌色/间距/圆角）、公共类
├── project.config.json         # 项目配置（appid、编译设置）
├── sitemap.json                # 索引配置
├── config/
│   ├── env.js                  # 环境切换：dev/staging/prod baseURL
│   └── index.js                # 常量：版本号、X-Client-Platform、错误码表、支付场景
├── utils/
│   ├── request.js              # 网络请求封装（核心）
│   ├── auth.js                 # 登录态管理：wx.login、code2session、token 存取、静默续期
│   ├── storage.js              # 存储封装（敏感数据、会话隔离）
│   ├── format.js               # 金额/日期/证件号格式化（保险术语展示）
│   ├── validate.js             # 表单校验（投保人/被保人、手机号、证件号）
│   ├── pay.js                  # 微信支付封装（统一下单→wx.requestPayment）
│   ├── sign.js                 # 合同签署（摘要防篡改、签署状态轮询）
│   └── tracker.js              # 埋点统计（业务转化漏斗）
├── components/
│   ├── product-card/           # 产品卡片（首页/列表）
│   ├── quote-stepper/          # 保额/保期选择步进器
│   ├── form-field/             # 表单项（含校验态）
│   ├── policy-card/            # 保单卡片
│   ├── contract-viewer/        # 合同 PDF/H5 预览容器
│   └── empty-state/            # 空态占位
└── pages/
    ├── index/                  # 首页（tab）
    ├── product/
    │   ├── list/               # 产品列表（分类/搜索）
    │   └── detail/             # 产品详情（保障责任、条款、投保入口）
    ├── quote/
    │   ├── index/              # 投保填写（被保人/受益人/保额/保期/健康告知）
    │   ├── confirm/            # 投保确认（试算保费、条款勾选、实名信息确认）
    │   └── result/             # 投保结果（提交后返回 quote_id / 跳转支付）
    ├── order/
    │   ├── list/               # 订单列表（状态筛选）
    │   └── detail/             # 订单详情（金额、状态、支付入口）
    ├── policy/
    │   ├── list/               # 保单列表（有效/待生效/已失效）
    │   └── detail/             # 保单详情（保障内容、PDF、续保）
    ├── contract/
    │   ├── list/               # 待签署/已签署合同列表
    │   └── sign/               # 合同签署页（条款阅读、签署、结果）
    ├── mine/                   # 我的（tab：个人信息、账户、服务入口）
    ├── login/                  # 登录/绑定手机号页
    ├── search/                 # 全局搜索（对接 /search）
    ├── webview/                # 通用 WebView（条款 H5、合同 PDF、客服）
    └── profile/
        ├── edit/               # 资料编辑（手机号/证件/地址）
        └── security/           # 账号安全（绑定手机号、退出登录）
```

### 1.2 页面规划（pages）

| 页面 | 路由 | 业务交互 |
|------|------|----------|
| 首页 | `/pages/index/index` | 头部搜索框（跳 /search）、轮播 Banner、推荐产品、分类入口（健康/意外/寿险/车险/财产）、快捷功能（我的保单/待签合同/在线客服） |
| 产品列表 | `/pages/product/list/index` | 分类 Tab、价格/热度/保障筛选、分页加载、产品卡片 |
| 产品详情 | `/pages/product/detail/index` | 保障责任、投保示例、条款说明（WebView 跳条款 H5）、「立即投保」CTA 按钮 |
| 投保填写 | `/pages/quote/index/index` | 投保人/被保人信息、受益人设置、保额保期选择（quote-stepper）、健康告知问答、保费试算 |
| 投保确认 | `/pages/quote/confirm/index` | 复核信息、试算保费展示、勾选并阅读投保须知/免责条款、提交投保 |
| 投保结果 | `/pages/quote/result/index` | 展示 quote_id、提示待支付/审核中、跳转订单或支付 |
| 订单列表 | `/pages/order/list/index` | 按状态 Tab（待支付/已支付/已取消/进行中）、订单卡片、去支付按钮 |
| 订单详情 | `/pages/order/detail/index` | 订单状态流转、金额明细、支付入口、关联保单/合同入口 |
| 保单列表 | `/pages/policy/list/index` | 有效/待生效/已失效筛选、保单卡片 |
| 保单详情 | `/pages/policy/detail/index` | 保障内容、PDF 保单（webview 预览）、续保/理赔入口 |
| 合同列表 | `/pages/contract/list/index` | 待签署/已签署、到期提醒 |
| 合同签署 | `/pages/contract/sign/index` | 条款逐段阅读（阅读计时）、电子签名、确认签署、结果页（落款+防篡改摘要） |
| 我的 | `/pages/mine/index` | 用户信息卡片、我的保单/订单/合同/实名、设置、退出登录 |
| 登录 | `/pages/login/index` | 微信一键登录、绑定手机号（getPhoneNumber） |
| 搜索 | `/pages/search/index` | 热搜、历史、结果列表（对接 /search） |
| 通用 WebView | `/pages/webview/index` | 条款 H5、PDF 保单、客服链接（白名单域名） |

**页面间关键流转（保险闭环）**：
```
首页 → 产品详情 → 投保填写 → 投保确认 → 提交 quote
     → 投保结果 → 生成 order → 微信支付 → 支付回调
     → 保单生成 → 保单详情 → 合同签署 → 签署完成（PDF 落款）
```

### 1.3 登录流程设计（wx.login → code2session → openid 直登 / 绑定微信 → JWT）

```
启动 app.js
  │
  ├─ 检查本地 JWT 是否存在且未过期（storage）
  │    ├─ 有效 → 进入业务（请求拦截器自动带 token）
  │    └─ 无效/过期/缺失 → 触发静默登录
  │
  └─ 静默登录（无需用户操作）:
       1. wx.login() 获取临时 code
       2. 调用后端 POST /api/v1/auth/wechat/login { code, platform: 'miniprogram' }
       3. 后端用 code 调微信 code2session 换取 openid/session_key，签发 JWT 返回
       4. 若用户已绑定手机号 → 直接返回 JWT → 存 storage → 完成
       5. 若未绑定手机号 → 返回 needBind=true → 进入绑定流程
  │
  └─ 绑定微信（未绑定 openid 的既有账号）:
       1. 用户先以既有方式登录账号（手机号/密码）
       2. 再次 wx.login() 获取新 code，调用 POST /api/v1/auth/wechat/bind { code }
       3. 后端用 code 调微信 code2session 换取 openid，写入 users.openid（该 openid 已被他人绑定 → 返回 40900）
       4. 返回业务页；此后可走微信一键登录（openid 直登）
```

**要点**：
- `code` 为一次性，**绝不在客户端缓存**；每次登录都重新 `wx.login`。
- 会话密钥 `session_key` 只留在后端，客户端永不接触，降低数据解密面。
- JWT 存储：短 Token 存 `wx.setStorageSync`（内存 + 本地），或用更安全方案（见第六章端侧安全）。
- 401 处理：请求拦截器收到 `code=401`（或 `code=UNAUTHORIZED`）时，静默重新走登录流程，成功后重放原请求（重放队列，避免并发重复登录）。
- 登出：`POST /api/v1/auth/logout`（后端 users.token_version+1，旧 refresh token 立即失效）→ 清空本地 token → 回登录页。

### 1.4 微信支付流程（统一下单 → wx.requestPayment）

```
用户点「去支付」
  │
  1. 前端 POST /api/v1/payments/wechat/prepay { orderId }  （带 JWT）
  2. 后端校验订单归属+状态，调微信【统一下单/JSAPI下单】，
     返回支付参数 { timeStamp, nonceStr, package(prepay_id), signType, paySign, ... }
  3. 前端调用 wx.requestPayment({ timeStamp, nonceStr, package, signType, paySign, success, fail })
  4. 支付结果处理：
       ├─ success → 后端回调确认 → 前端轮询 GET /api/v1/orders/{id} 刷新为已支付
       │          → 进入保单/合同流程
       ├─ fail (cancel) → 留在订单页，展示待支付，不误判为失败
       └─ 状态不明 → 主动调后端查询订单状态兜底（防丢单）
```

**要点**：
- **签名参数 paySign 由后端生成**，前端不接触商户密钥；`package` 中 prepay_id 由后端在 prepay 响应返回。
- 支付成功判定**以后端回调 + 订单查询为准**，不依赖 wx.requestPayment 的 success 回调（可能因网络中断返回 fail 但实际已扣款）。
- 金额一律后端计算，前端只展示，杜绝价格篡改。
- 支付前需在微信公众平台配置 `requestPayment` 支付目录与回调域名。

### 1.5 网络请求封装（utils/request.js）

核心能力：baseURL、token 注入、错误码统一、loading、重放、trace 上报。

```
request(options)
  ├─ 拼接 baseURL + path（config/env.js 按环境取）
  ├─ 头注入：
  │    Content-Type: application/json
  │    Authorization: Bearer <jwt>
  │    X-Client-Platform: miniprogram   ← 后端据此区分来源
  │    X-Request-Id: <uuid>             ← 与后端 trace_id 关联排查
  ├─ loading：可选 showLoading / 页面级自定义
  ├─ 发送 wx.request（method/data/timeout）
  ├─ 响应解析：
  │    ResponseEnvelope { code, message, data, trace_id }
  │    ├─ code === 0 / SUCCESS → resolve(data)
  │    ├─ code === 401/过期   → 触发静默重登 + 重放原请求
  │    ├─ code 业务错误       → 统一 toast(message) + reject({code,message})
  │    └─ HTTP 层错误(4xx/5xx) → 网络兜底 + 上报 trace_id
  └─ 并发去抖：登录刷新期间挂起其他请求，完成后再放行
```

错误码约定（与后端对齐，示例）：
```
0            SUCCESS      成功
40000        PARAM        参数错误
40100        UNAUTH       未登录/token 失效（触发重登）
40300        FORBIDDEN    越权（非本人资源）
40400        NOT_FOUND    资源不存在
40900        CONFLICT     状态冲突（重复投保/重复支付）
42900        RATE_LIMIT   限流
50000        SERVER       服务端错误（展示 trace_id 便于客服定位）
```

### 1.6 tabBar 与导航设计

`app.json` 配置 tabBar（3~4 个主入口，微信 tabBar 上限 5）：

| tab | 页面 | 说明 |
|-----|------|------|
| 首页 | `/pages/index/index` | 产品发现与营销 |
| 保单 | `/pages/policy/list/index` | 保单中心（tab 直达列表） |
| 我的 | `/pages/mine/index` | 个人中心与账户 |

> 说明：tabBar 建议 3 个主 Tab（首页/保单/我的）为主流保险 App 结构；订单、合同等高频二级入口放在「我的」与服务首页，避免 Tab 过多稀释主流程。如需 4 Tab，可在「保单」与「我的」间加「服务」（合同/理赔入口聚合）。

- **导航方式**：
  - Tab 切换：`wx.switchTab`（仅限 tabBar 页）。
  - 页面栈跳转：`wx.navigateTo`（详情/表单/签署等，可携带参数或通过全局 store 传复杂对象）。
  - 返回/回退：`wx.navigateBack`；投保结果页用 `wx.redirectTo` 或 `wx.reLaunch` 清栈，避免用户回退到「已提交」的表单。
  - 支付/登录中转：完成后用 `wx.redirectTo` 回到业务承接页。
- **业务守卫**：投保/支付/签署等敏感操作前校验登录态与实名，未登录跳转 `/pages/login/index` 并记录 redirect 回跳路径。
- **页面栈管理**：投保链路（填写→确认→结果）采用定向栈管理，防止重复提交与回退脏数据。

---

## 二、鸿蒙端（ArkTS + ArkUI）

### 2.1 目录结构

```
harmony/
├── AppScope/
│   ├── app.json5                # 应用配置：bundleName、versionCode、module 声明
│   └── resources/
│       └── base/
│           ├── element/string.json      # 字符串资源
│           └── media/                   # 应用图标/启动图
├── entry/
│   └── src/main/
│       ├── module.json5         # 模块配置：权限声明（网络/相机/定位）、EntryAbility
│       ├── ets/
│       │   ├── entryability/
│       │   │   └── EntryAbility.ets     # 应用入口 Ability（onCreate/onWindowStageCreate）
│       │   ├── pages/                   # 页面（ArkUI）
│       │   │   ├── Index.ets            # 首页
│       │   │   ├── ProductListPage.ets
│       │   │   ├── ProductDetailPage.ets
│       │   │   ├── QuotePage.ets        # 投保填写
│       │   │   ├── QuoteConfirmPage.ets
│       │   │   ├── QuoteResultPage.ets
│       │   │   ├── OrderListPage.ets
│       │   │   ├── OrderDetailPage.ets
│       │   │   ├── PolicyListPage.ets
│       │   │   ├── PolicyDetailPage.ets
│       │   │   ├── ContractListPage.ets
│       │   │   ├── ContractSignPage.ets
│       │   │   ├── MinePage.ets         # 我的
│       │   │   ├── LoginPage.ets
│       │   │   ├── SearchPage.ets
│       │   │   ├── WebPage.ets          # Web 组件容器（条款 H5 / PDF）
│       │   │   └── ProfileEditPage.ets
│       │   ├── common/                  # 公共资源/能力
│       │   │   ├── constants/
│       │   │   │   └── CommonConstants.ets   # 平台标识、错误码、路由表
│       │   │   ├── utils/
│       │   │   │   ├── HttpUtil.ets          # @ohos.net.http 封装（核心）
│       │   │   │   ├── AuthManager.ets       # JWT 存取/刷新/登录态
│       │   │   │   ├── StorageUtil.ets       # Preferences/安全存储封装
│       │   │   │   ├── FormatUtil.ets        # 金额/日期格式化
│       │   │   │   ├── ValidateUtil.ets      # 表单校验
│       │   │   │   ├── PayManager.ets        # 支付对接
│       │   │   │   └── SignUtil.ets          # 合同签署摘要/轮询
│       │   │   ├── components/               # 自定义组件
│       │   │   │   ├── ProductCard.ets
│       │   │   │   ├── QuoteStepper.ets
│       │   │   │   ├── FormField.ets
│       │   │   │   ├── PolicyCard.ets
│       │   │   │   └── EmptyState.ets
│       │   │   ├── models/                  # 数据模型（与 ResponseEnvelope 对齐）
│       │   │   │   ├── ResponseEnvelope.ets # { code, message, data, trace_id }
│       │   │   │   ├── ProductModel.ets
│       │   │   │   ├── QuoteModel.ets
│       │   │   │   ├── OrderModel.ets
│       │   │   │   ├── PolicyModel.ets
│       │   │   │   └── ContractModel.ets
│       │   │   └── store/                   # 全局状态（AppStorage / 单例）
│       │   │       └── GlobalStore.ets
│       │   ├── resources/                  # 页面级资源
│       │   │   └── base/
│       │   │       ├── element/            # 颜色/字符串/尺寸
│       │   │       ├── media/              # 图标/图片
│       │   │       └── profile/main_pages.json   # 路由表（pages 声明）
│       │   └── ets/ohosTest/               # 测试
│       └── resources/rawfile/               # 原始文件（本地静态条款等）
└── oh-package.json5            # 依赖声明
```

### 2.2 ArkTS 页面规划（与小程序/Flutter 功能对齐）

| 页面（ets） | 对应小程序 | 关键 ArkUI 组件/能力 |
|-------------|-----------|----------------------|
| Index.ets | 首页 | `Tabs`/`List`、搜索框、`Grid` 分类入口 |
| ProductListPage.ets | 产品列表 | `List`+`LazyForEach` 分页、`Tab` 分类、`Search` |
| ProductDetailPage.ets | 产品详情 | `Scroll`、保障卡片、`Button` CTA、`Web` 加载条款 |
| QuotePage.ets | 投保填写 | `TextInput`/`DatePicker`/`Radio`/`Checkbox`、健康告知 `Dialog`、保费试算 |
| QuoteConfirmPage.ets | 投保确认 | 信息复核 `List`、免责条款 `Web`、`Checkbox` 勾选、提交 |
| QuoteResultPage.ets | 投保结果 | 状态展示、跳支付/回订单 |
| OrderListPage.ets / OrderDetailPage.ets | 订单 | 状态 Tab、金额明细、支付入口 |
| PolicyListPage.ets / PolicyDetailPage.ets | 保单 | 保单卡片、PDF 预览（`Web`/文件）、续保 |
| ContractListPage.ets / ContractSignPage.ets | 合同 | 条款阅读、电子签名（Canvas 手写/图片）、确认签署、防篡改摘要展示 |
| MinePage.ets | 我的 | 用户卡片、服务入口 `Grid`、设置 |
| LoginPage.ets | 登录 | 账号/手机号登录、`Button` 登录 |
| SearchPage.ets | 搜索 | `Search`、热搜、历史、结果 `List` |
| WebPage.ets | webview | `Web` 组件（条款 H5 / PDF / 客服） |
| ProfileEditPage.ets | 资料编辑 | 表单编辑、保存 |

> 说明：鸿蒙端**不接入微信登录/微信支付**，走通用手机号+验证码登录与通用收银台/第三方支付 SDK（见 2.4）。业务功能面与小程序对齐，仅在账号体系与支付渠道上差异化。

### 2.3 网络层（@ohos.net.http 封装）

`HttpUtil.ets` 基于 `@ohos.net.http` 提供统一请求入口，行为与小程序 `request.js` 对齐。

```
HttpUtil.request(options)
  ├─ 拼接 baseURL + path（按 buildMode 取 env）
  ├─ 头注入：
  │    Content-Type: application/json
  │    Authorization: Bearer <jwt>
  │    X-Client-Platform: harmony        ← 后端据此区分鸿蒙来源
  │    X-Request-Id: <uuid>
  ├─ 创建 http.HttpRequest → request(url, { method, header, extraData, expectDataType })
  ├─ 响应解析：
  │    ResponseEnvelope { code, message, data, trace_id }
  │    ├─ code === 0/SUCCESS → resolve(data)
  │    ├─ 401/过期 → AuthManager 静默刷新 + 重放（并发去抖）
  │    ├─ 业务错误 → 统一 toast + reject
  │    └─ 网络异常（errno）→ 重试/降级 + 上报
  └─ 超时/连接复用：设置 connectTimeout/readTimeout；长列表分页用 Cancellable 支持取消
```

要点：
- **ArkTS 严格类型**：`ResponseEnvelope` 用 `interface`/`class` 建模，`data` 用泛型 `<T>`；JSON 解析用 `JSON.parse` 后强转，配合 `util` 校验。
- **并发安全**：ArkTS 默认单线程模型 + 异步回调，用 `Promise` 封装避免回调地狱；登录刷新用全局 `Promise` 去抖（同一时刻只刷新一次）。
- **错误处理**：统一 `HttpError`/`BusinessError` 区分网络层与业务层错误；`trace_id` 透传到全局日志用于客服排障。
- **后台/续传**：大文件（保单 PDF 下载）用 `@ohos.request`（Upload/Download）能力，配合进度条与断点续传。

### 2.4 登录 / 支付能力对接（@ohos 相关能力）

**登录（手机号 + 验证码）**：
```
LoginPage → 输入手机号 → 请求短信验证码（@ohos 网络）
  → 后端 POST /api/v1/auth/sms-code
  → 输入验证码 → POST /api/v1/auth/login { phone, code, platform:'harmony' }
  → 后端校验并签发 JWT → AuthManager 安全存储 → 进入业务
```
- 与小程序差异：无 `wx.login` 静默能力，需显式登录；Token 通过 `@ohos.security` 安全存储（见第六章）。
- 可选增强：接入 `@ohos.account` 系统账号（如 HarmonyOS 帐号体系）作为登录方式，但核心仍是手机号+验证码以保持三端一致。

**支付**：
```
OrderDetailPage → 点「去支付」
  → 后端 POST /api/v1/payments/{channel}/prepay { orderId, platform:'harmony' }
  → 后端返回收银台/支付 SDK 所需的支付参数（渠道标识：支付宝/银联/华为支付等）
  → 前端调用对应支付 SDK（鸿蒙支付服务 / 第三方 SDK）拉起收银台
  → 回调结果 → 以后端订单查询为准兜底 → 刷新订单状态
```
- 后端根据 `X-Client-Platform: harmony` + `channel` 参数返回相应支付渠道的预支付参数。
- 鸿蒙端通过 `module.json5` 声明所需权限与 SDK 依赖（`oh-package.json5` 引入支付 SDK）。
- 敏感操作（支付/签署）可要求二次验证（生物识别/系统指纹，`@ohos.biometrics`）。

**其它端侧能力**：
- 通知：`@ohos.notificationManager` 推送保单生效、续保、签署提醒。
- 网络权限：`module.json5` 声明 `ohos.permission.INTERNET`；正式包需配置 `useNormalizedOHMUrl` 与域名校验。
- PDF/文件：`@ohos.file.fs` 存储保单 PDF，`@ohos.pasteboard` 处理签署内容粘贴。

---

## 三、通用：三端（Flutter/小程序/鸿蒙）共用 REST API 的契约约束与差异处理

### 3.1 平台识别与头部契约

所有请求**必须**携带以下头，后端据此区分来源并做差异化处理：

| Header | 取值 | 说明 |
|--------|------|------|
| `X-Client-Platform` | `flutter` / `miniprogram` / `harmony` | 来源端标识，**必填** |
| `X-Client-Version` | 如 `1.0.0` | 端版本号，用于灰度/强更 |
| `Authorization` | `Bearer <jwt>` | 认证凭证（登录后） |
| `X-Request-Id` | UUID | 与响应 `trace_id` 关联，用于链路排查 |
| `Accept-Language` | `zh-CN` 等 | 多语言（保险条款展示） |

### 3.2 契约约束（统一约定）

1. **统一响应信封**：所有接口返回 `ResponseEnvelope { code, message, data, trace_id }`。三端解析逻辑一致，仅语言/类型机制不同。
2. **错误码规范**：`code` 为业务层约定（0/SUCCESS、40100 未登录、40300 越权、40900 冲突等）。HTTP 状态码仅表达传输层结果，业务以 `code` 为准。
3. **金额精度**：金额一律以**分（int64）**传输（`amount` 单位分），三端各自负责格式化展示，避免浮点误差。
4. **时间格式**：统一 `ISO 8601`（如 `2026-09-01T10:00:00+08:00`），或服务端返回 UTC 时间戳由端侧本地化。
5. **枚举语义**：订单/保单/合同状态、支付渠道、平台标识均用**稳定字符串枚举**（如 `order.status = "PENDING"|"PAID"|"CANCELLED"`），端侧建立映射表展示中文文案，禁止硬编码展示端变化无常。
6. **分页契约**：列表接口统一 `{ page, pageSize }` 入参，响应 `{ items, total, page, pageSize }`；三端分页逻辑一致。
7. **Idempotency**：投保/支付等写操作支持 `Idempotency-Key`（或客户端生成 requestId），防止重复提交产生重复订单。
8. **版本兼容**：后端接口采用 `/api/v1` 前缀版本化；端侧 `X-Client-Version` 用于服务端做向后兼容与强更提示。

### 3.3 三端差异处理矩阵

| 维度 | Flutter | 微信小程序 | 鸿蒙(ArkTS) |
|------|---------|-----------|-------------|
| 语言/运行时 | Dart | JS | TypeScript(ArkTS) |
| 登录方式 | 手机号+验证码 | wx.login + 绑定手机号 | 手机号+验证码（可接系统帐号） |
| 支付渠道 | 通用收银台/支付宝等 | 微信支付 | 华为支付/支付宝/银联等 |
| 网络库 | dio | wx.request | @ohos.net.http |
| 状态管理 | Riverpod/Bloc | 页面 data + 全局 app.globalData | AppStorage/单例 |
| UI 框架 | Widget | WXML/WXSS | ArkUI |
| 本地存储 | shared_preferences/secure | wx.setStorageSync | Preferences/@ohos.security |
| 推送 | FCM/厂商 | 订阅消息 | @ohos.notificationManager |
| 平台头值 | `flutter` | `miniprogram` | `harmony` |

**后端需提供的差异化能力（配合 X-Client-Platform）**：
- `/auth/wechat/login` 仅对 `miniprogram` 开放；`/auth/login` 对 flutter/harmony 开放。
- `/payments/*/prepay` 根据平台返回不同支付渠道参数（wechat / alipay / unionpay / huawei-pay）。
- 相同业务域（products/quotes/orders/policies/contracts/search）三端完全共用，仅在登录/支付两个能力上分流。

### 3.4 三端共用的数据流（保险闭环）

```
产品浏览(/products) → 投保试算(/quotes POST) → 订单生成(/orders POST)
  → 支付(/payments prepay + 回调) → 保单生成(/policies)
  → 合同签署(/contracts 创建+签署) → 保单/合同查看(PDF)
```
三个端在此闭环上完全一致，仅在「登录」与「支付」两处平台化差异。

---

## 四、端侧安全

### 4.1 敏感数据存储

| 数据 | 小程序 | 鸿蒙 | 建议 |
|------|--------|------|------|
| JWT 访问令牌 | `wx.setStorageSync`（短 Token）+ 尽量缩短有效期；进阶可存内存+AppLaunch 校验 | `@ohos.security`（安全存储/`ohos.security` 保险箱）或 `Preferences` 加密 | 长效刷新令牌务必安全存储；访问令牌尽量短生命周期 |
| 手机号/证件号 | 加密后存储，展示脱敏（138****1234 / 110**********12） | 同左 + 安全存储 | 敏感 PII 本地不落明文 |
| 用户资料/草稿 | `wx.setStorageSync`（非敏感） | `Preferences` | 区分敏感与非敏感 |

**通行原则**：能不入本地就不入本地；必须入则加密；展示一律脱敏（手机号/证件号/地址）；退出登录时清理会话数据。

### 4.2 JWT 防泄漏

- **短过期 + 刷新**：访问令牌短时效（如 15~30 分钟），配合刷新令牌续期；过期自动静默刷新（不打扰用户）。
- **仅内存承载**（鸿蒙/Flutter 可做）：访问令牌存内存单例，冷启动从安全存储恢复；小程序受限于 API 用 `wx.setStorageSync`，但结合后端 `jti` 校验 + 过期即弃降低风险窗口。
- **不写入日志/埋点**：任何日志、上报、trace 均不落 `Authorization` 与完整 token。
- **Header 传输**：仅经 HTTPS（`request` 强制 https）；小程序 `request` 合法域名配置仅放生产域名。
- **防重放/CSRF**：`X-Request-Id` + 后端校验来源平台与设备指纹；关键操作（支付/签署）要求二次验证。
- **绑定上下文**：JWT 可绑定设备指纹（客户端生成随机 deviceId 上报），后端校验同设备续期，被盗跨设备使用即失效。

### 4.3 越权防护（业务层）

- **对象级鉴权**：所有资源接口（订单/保单/合同详情）由后端按 `user_id` 校验归属，客户端不依赖前端隐藏按钮来防越权。
- **水平越权**：禁止仅凭 `id` 可访问他人资源；后端必须校验 `resource.user_id === token.user_id`。
- **垂直越权**：角色权限（投保人/受益人/管理员）由后端 `role` 判定，前端仅做 UI 级隐藏。
- **签名/摘要防篡改**：
  - 投保/支付金额后端权威计算，前端只展示。
  - 合同电子签署：客户端对合同正文计算摘要（如 SHA-256），连同签名一并提交后端；后端比对服务端算出的摘要与客户端提交值，防止正文被篡改。合同落款展示摘要，供后续验真。
  - 敏感写请求可加请求签名（HMAC/端密钥）防中间人改写。
- **健康告知/实名真实性**：关键告知项与实名认证走后端权威核验（对接第三方实名/健康接口），客户端仅采集不判定。

### 4.4 平台安全清单

**小程序**：
- `app.json` 配置合法 request/uploadFile/downloadFile 域名（仅 https）。
- 不引入未授权第三方 SDK；登录 `code` 一次性、不缓存；`session_key` 后端持有。
- 敏感操作 `wx.requestPayment` 参数由后端生成；禁止在代码包硬编码商户密钥。
- 代码混淆与分包；`project.config.json` 开启 `setting` 校验合法域名。

**鸿蒙**：
- `module.json5` 最小权限原则（仅 INTERNET 等必要权限）。
- HTTPS 证书校验（`@ohos.net.http` 默认校验证书链，不关闭校验）。
- 使用 `@ohos.security` 安全存储承载令牌与密钥；生物识别门禁用于支付/签署。
- 正式签名（HarmonyOS 证书）发布，避免调试签名泄漏。
- 禁运明文日志；发布包 `minify` 混淆。
