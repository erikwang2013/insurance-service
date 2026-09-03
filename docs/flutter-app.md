# 保险服务平台 — Flutter 客户端架构规划

> 版本: v1.0 | 日期: 2026-09-01 | 状态: 待评审
> 适用端: iOS / Android / Web / 桌面(macOS / Windows / Linux)
> 关联后端: `bee-rust`(Rust REST API, JWT 认证, 统一响应 `ResponseEnvelope`)
> 说明: 原生微信小程序与鸿蒙 ArkTS 端共用同一 REST API,本规划仅覆盖 Flutter 客户端。

---

## 目录

1. [总体架构与分层](#1-总体架构与分层)
2. [Flutter 项目目录结构](#2-flutter-项目目录结构)
3. [技术选型](#3-技术选型)
4. [页面 / 功能规划(features)](#4-页面--功能规划features)
5. [关键数据模型 Dart 类](#5-关键数据模型-dart-类)
6. [统一响应处理](#6-统一响应处理)
7. [环境配置](#7-环境配置)
8. [多端差异处理](#8-多端差异处理)
9. [核心业务流(投保→支付→保单→签署)](#9-核心业务流)
10. [脚手架落地清单](#10-脚手架落地清单)

---

## 1. 总体架构与分层

采用 **Feature-First(特性优先)** 的模块化架构,自上而下分为三层:

```
┌─────────────────────────────────────────────────────────────┐
│                      presentation (UI)                        │
│   screens / widgets / pages — 各 feature 内的 UI 层           │
├─────────────────────────────────────────────────────────────┤
│                      application (应用层)                      │
│   providers / controllers / use_cases — 状态编排与业务命令      │
├─────────────────────────────────────────────────────────────┤
│                        domain (领域层)                        │
│   models / entities / repositories(抽象) / services(抽象)      │
├─────────────────────────────────────────────────────────────┤
│                      data (数据层)                            │
│   dio 网络 / api_client / local_storage / repository 实现      │
└─────────────────────────────────────────────────────────────┘
```

**核心设计原则:**

- **单向数据流**: UI 事件 → Provider/Controller → Repository → API → 状态回写 → UI 重建。
- **依赖倒置**: domain 层只定义抽象接口,data 层提供实现,通过 Riverpod 的 `Provider` / `overrideWith` 注入,便于测试与多端适配。
- **跨端复用最大化**: 业务逻辑、模型、网络层、状态管理全部与平台无关,仅在 `shared/` 的 platform 层做条件编译与能力探测。
- **按 feature 垂直切片**: 每个业务模块自包含(UI + 状态 + 数据),`core` 与 `shared` 提供横切能力(网络、路由、鉴权、通用组件)。

---

## 2. Flutter 项目目录结构

以 `app/` 为 Flutter 工程根目录。分层结构:

```
app/
├── pubspec.yaml
├── analysis_options.yaml
├── .env.dev
├── .env.prod
├── README.md
├── lib/
│   ├── main.dart                      # 入口,initialize + 运行 App
│   ├── app.dart                       # MaterialApp.router + 主题 + 多语言
│   │
│   ├── core/                          # 横切、与业务无关
│   │   ├── config/
│   │   │   ├── app_config.dart        # 运行期配置(ApiConfig、AppConfig)
│   │   │   └── env_config.dart        # 读取 .env 环境
│   │   ├── constants/
│   │   │   ├── api_paths.dart         # /api/v1/* 路径常量
│   │   │   └── app_constants.dart
│   │   ├── network/
│   │   │   ├── dio_client.dart        # Dio 单例构造(BASE + 拦截器)
│   │   │   ├── interceptors/
│   │   │   │   ├── auth_interceptor.dart      # JWT 注入
│   │   │   │   ├── token_refresh_interceptor.dart # 401 刷新重试
│   │   │   │   ├── error_interceptor.dart     # ResponseEnvelope 错误归一
│   │   │   │   └── logging_interceptor.dart   # 开发日志(trace_id)
│   │   │   ├── api_client.dart        # 泛型请求封装(get/post/put...)
│   │   │   ├── base_response.dart     # ResponseEnvelope 模型
│   │   │   └── exceptions.dart        # ApiException / BizException / NetworkException
│   │   ├── router/
│   │   │   └── app_router.dart        # go_router 路由表
│   │   ├── theme/
│   │   │   ├── app_theme.dart
│   │   │   └── app_colors.dart
│   │   ├── utils/
│   │   │   ├── validators.dart        # 投保表单校验(身份证/手机号/金额)
│   │   │   ├── formatters.dart        # 金额/日期/电话脱敏
│   │   │   └── result_state.dart      # 三态 AsyncValue 包装
│   │   └── storage/
│   │       ├── token_storage.dart     # 抽象
│   │       ├── secure_token_storage.dart # flutter_secure_storage 实现
│   │       └── prefs_storage.dart     # shared_preferences 实现
│   │
│   ├── shared/
│   │   ├── platform/
│   │   │   ├── platform_info.dart     # kIsWeb / Platform 封装
│   │   │   └── payment_launcher.dart  # 支付跳转统一抽象
│   │   ├── widgets/
│   │   │   ├── loading_view.dart
│   │   │   ├── empty_view.dart
│   │   │   ├── error_view.dart        # 通用错误 + 重试
│   │   │   ├── app_webview.dart       # 签署/充值 webview 封装
│   │   │   └── app_button.dart
│   │   └── responsive/
│   │       ├── responsive_widget.dart  # 手机/平板/桌面三档
│   │       └── desktop_scaffold.dart   # 桌面侧边栏布局
│   │
│   └── features/                      # 按业务垂直切片(详 §4)
│       ├── auth/
│       ├── home/
│       ├── product/
│       ├── quote/
│       ├── order/
│       ├── payment/
│       ├── policy/
│       ├── contract/
│       └── profile/
│
├── test/                             # 单元测试
│   ├── core/network/
│   ├── features/...
│   └── fixtures/
├── integration_test/                 # 集成/端到端测试
└── platform 目录(android/ ios/ web/ macos/ windows/ linux)
```

### 单个 feature 的内部结构(以内 `features/product` 为例)

```
features/product/
├── data/
│   ├── product_remote_datasource.dart   # Dio 调用 /api/v1/products
│   └── product_repository_impl.dart     # 实现 domain 抽象
├── domain/
│   ├── product.dart                     # 模型
│   ├── product_repository.dart          # 抽象接口
│   └── product_search_params.dart
├── application/
│   ├── product_provider.dart            # Riverpod providers
│   ├── product_list_controller.dart     # 列表分页/筛选状态
│   └── product_detail_controller.dart
└── presentation/
    ├── product_list_page.dart
    ├── product_detail_page.dart
    ├── product_search_page.dart
    └── widgets/
        ├── product_card.dart
        └── product_search_bar.dart
```

> 说明:轻量 feature(auth、home、profile)可省略 data/domain 深度嵌套,收敛为 `xxx_screen.dart` + `xxx_controller.dart` 即可,保持 YAGNI。

---

## 3. 技术选型

| 关注点 | 选型 | 理由 |
|---|---|---|
| 状态管理 | **Riverpod**(flutter_riverpod + Riverpod 2.x Codegen) | 见下 |
| 路由 | **go_router** | 声明式路由、与 URL(Web)天然对齐、支持 redirect/state/嵌套导航、深链 |
| 网络层 | **Dio** + 自研拦截器 | 拦截器机制成熟,契合 JWT/刷新/错误归一需求 |
| JSON 序列化 | **json_serializable / freezed** | 生成 `fromJson/toJson`,freezed 提供不可变 + copyWith + union(适配三态) |
| 本地存储 | **shared_preferences**(轻量 KV) + **flutter_secure_storage**(Token/敏感) | secure 存凭据,prefs 存非敏感设置 |
| 依赖注入 | **Riverpod providers(无需额外 DI)** | 综合在状态管理中 |
| DI 手动装配 | Riverpod 全家桶即可 | — |
| 开发便利 | **Riverpod Codegen**(codegen 或 watch) | 减少样板 |
| 构建配置 | **flutter_dotenv**(dev/prod baseUrl) | 见 §7 |
| 多语言 | **flutter_localizations** + intl | 预留 |
| 日志 | **logger** + Dio LogInterceptor | trace_id 贯通 |
| 桌面适配 | **flutter_platform_widgets**(可选)、响应式组件 | 见 §8 |
| Codegen | **build_runner** | riverpod_generator + json_serializable + freezed |

### 3.1 状态管理:为什么选 Riverpod(而非 Bloc)

- **编译期安全**: 类型安全,`ConsumerWidget` / `ref.watch` 自带依赖图,避免 Provider 的多层重建问题。
- **可测试性好**: provider 可注入 mock 依赖(`overrideWith`),纯 Dart 单测无需 widget 环境。
- **async 一等公民**: `AsyncValue<T>` 内置 loading / data / error 三态,直接支撑 §6 的三态 UI,无需自造状态枚举。
- **组合优于继承**: 命令(controller)与数据(provider)分离,配合 `AsyncNotifier` 组织复杂业务流(如下单流水线)。
- **与免费zed 结合**: `AsyncValue.when(...)` 三态分支写起来最贴近本项目 UI。
- **对比 Bloc 的考量**: Bloc 事件驱动、结构严谨、团队大时好约束;但样板代码多(Event/State/Bloc 三件套)、跨端复用繁琐、Web 深链场景与 go_router 的集成不如 Riverpod 直接。本项目业务流偏"命令式流水线"(报价→下单→支付→回查)且需强类型 Web 适配,Blo c 的收益不匹配成本,故选 Riverpod。

### 3.2 路由:go_router

```dart
// 核心路由表(节选)
final appRouter = GoRouter(
  initialLocation: '/home',
  redirect: _guard,                    // 鉴权守卫,见代码
  routes: [
    GoRoute(path: '/login',  builder: (c, s) => const LoginPage()),
    GoRoute(path: '/home',   builder: (c, s) => const HomeShell(), routes: [
      GoRoute(path: 'products', builder: ...),          // 产品列表
      GoRoute(path: 'products/:id', builder: ...),      // 产品详情
      GoRoute(path: 'search',    builder: ...),         // 全局搜索
      GoRoute(path: 'orders',    builder: ...),
      GoRoute(path: 'policies',  builder: ...),
      GoRoute(path: 'contracts', builder: ...),
      GoRoute(path: 'profile',   builder: ...),
    ]),
    GoRoute(path: '/quote/:productId', builder: ...),   // 投保表单
    GoRoute(path: '/order/confirm/:quoteId', builder: ...),
    GoRoute(path: '/payment/:orderId', builder: ...),
    GoRoute(path: '/policy/:id', builder: ...),
    GoRoute(path: '/contract/sign/:contractId', builder: ...), // 电子签署 WebView
    GoRoute(path: '/payment/result/:orderId', builder: ...),   // 支付结果
  ],
);
```

**路由特性落地:**
- **redirect 守卫**: 未登录访问受保护路由 → 跳 `/login`。
- **Web 深链**: go_router 直接映射 URL 路径,Web 端可分享/刷新恢复(§8)。
- **状态传递**: 用 `state.uri.queryParameters` / `extra` 传递商品 ID、回跳信息。
- **进入/离开保护**: 表单未保存确认、支付中禁止返回。

### 3.3 网络层 Dio + 拦截器

```
┌──────────────────────────────────────────────────────────────┐
│ Dio(BaseOptions: baseUrl / connectTimeout / receiveTimeout)   │
│   ├── AuthInterceptor          → 注入 Authorization: Bearer token
│   ├── TokenRefreshInterceptor  → 401 时静默刷新 + 队列重试原请求
│   ├── ErrorInterceptor         → 把 HTTP/业务错误归一为 ApiException
│   └── LoggingInterceptor       → 开发日志,打印 trace_id
└──────────────────────────────────────────────────────────────┘
```

```dart
final dio = Dio(
  BaseOptions(
    baseUrl: AppConfig.instance.apiBaseUrl,
    connectTimeout: const Duration(seconds: 15),
    receiveTimeout: const Duration(seconds: 15),
    headers: {'Content-Type': 'application/json'},
  ),
)..interceptors.addAll([authInterceptor, tokenRefreshInterceptor, errorInterceptor, loggingInterceptor]);
```

**拦截器职责(详 §6):**

1. **AuthInterceptor**: 从 `TokenStorage` 取 accessToken,`request` 阶段注入 `Authorization: Bearer <token>`。
2. **TokenRefreshInterceptor**: `onError` 捕获 401/业务码 401(unauthenticated),加锁并发去重刷新(RefreshToken换新AccessToken),失败则登出跳登录。
3. **ErrorInterceptor**: 解析 `ResponseEnvelope`,`code != 0` 时抛出 `BizException(code, message, trace_id)`;网络/超时抛 `NetworkException`。
4. **LoggingInterceptor**: 打印方法/路径/耗时/状态码/`trace_id`,便于与后端错误链路对齐。

**泛型请求封装(ApiClient):**

```dart
class ApiClient {
  final Dio _dio;
  Future<T> get<T>(String path, {Map<String,dynamic>? query, T Function(dynamic)? decode});
  Future<T> post<T>(String path, {Object? data, T Function(dynamic)? decode});
  // put / delete ...
}
```

### 3.4 本地存储

| 用途 | 方案 | key 示例 |
|---|---|---|
| accessToken / refreshToken / user 敏感信息 | `flutter_secure_storage`(Keychain/Keystore) | `token_access`, `token_refresh` |
| 非敏感设置(主题、搜索历史、地区) | `shared_preferences` | `theme_mode`, `recent_search` |
| 登录态内存镜像 | `StateNotifier<AuthState>`(Riverpod) | — |

`TokenStorage` 抽象接口便于 mock;secure 与 prefs 实现背后注入。

---

## 4. 页面 / 功能规划(features)

底部 Tab 导航:`home`(首页) / `product`(产品/搜索) / `order`(订单) / `policy`(保单) / `profile`(我的)。合同签署入口挂在保单与订单详情。

### 4.1 features/auth — 登录 / 注册

- `LoginPage`:手机号/密码或验证码登录,调用 `POST /api/v1/auth/login`,存入 token,`ref.read(authController.notifier).login()`。
- `RegisterPage`:注册表单(手机号验证码 / 密码确认),`POST /api/v1/auth/register`。
- `ForgotPasswordPage`(可选)。
- 控制器: `AuthController extends AsyncNotifier<AuthState>`;token 刷新/登出/本地恢复会话。
- 守卫: 通过 `appRouter` 的 `redirect` + `authController` 实现全局登录态。

### 4.2 features/home — 首页

- `HomePage`:欢迎区(登录态)、产品分类入口、运营 Banner、推荐产品列表、快捷入口(我的保单/合同签署)。
- 数据: `GET /api/v1/products/featured`（首页推荐）。
- 卡片点击 → 产品详情/搜索。

### 4.3 features/product — 产品列表 / 详情 / 搜索

- `ProductListPage`:按分类分页列表(加载更多),`GET /api/v1/products?category=&page=&size=`。
- `ProductDetailPage`:产品资料(保障责任、条款、费率试算入口),`GET /api/v1/products/:id`;CTA「立即投保」→ `/quote/:id`。
- `SearchPage`:全局搜索,`GET /api/v1/search?keyword=&type=`,含历史记录(shared_preferences)、联想、空态。
- 控制器: `ProductListController`(分页 + 筛选)、`KeywordSearchController`(防抖 + 取消旧请求)。

### 4.4 features/quote — 投保报价表单

- `QuoteFormPage(productId)`:多步骤表单(被保人信息、保障期/保额选择、健康告知、受益人)。
- 校验: `validators.dart`(身份证 18 位、手机号、金额范围、必填);表单错误 inline 展示。
- 试算: 选择档位后本地/接口试算保费。
- 提交: `POST /api/v1/quotes` → 返回 `Quote`(含 quoteId / premium / 报价有效期)。
- 定价展示: 报价确认页展示保费明细、条款链接 →「下一步」→ `/order/confirm/:quoteId`。

### 4.5 features/order — 订单

- `OrderConfirmPage(quoteId)`:确认投保方案、被保人摘要、保费、优惠券 → 生成订单 `POST /api/v1/orders`。
- `OrderListPage`:全部/待支付/已支付订单,`GET /api/v1/orders?status=`。
- `OrderDetailPage`:订单详情、状态流转、去支付 CTA。
- 控制器: `OrderController`(下单流水线,见 §9 Step 2)。

### 4.6 features/payment — 支付

- `PaymentPage(orderId)`:选择支付渠道(微信/支付宝/银行卡/余额)、金额确认、发起支付 `POST /api/v1/payments`。
- `PaymentWebView`:Web/部分渠道走 H5 收银台 + JS bridge 回跳。
- `PaymentResultPage`:支付结果轮询 `GET /api/v1/payments/:orderId/status`,成功 → 保单生成 → 跳转保单/签署。
- 多端差异见 §8(移动端 SDK 拉起 vs Web 收银台 vs 桌面扫码)。

### 4.7 features/policy — 保单列表 / 详情

- `PolicyListPage`:我的保单,`GET /api/v1/policies`,状态筛选(有效/待生效/已到期)。
- `PolicyDetailPage`:保单摘要、保障内容、期限、缴费记录、下载 PDF(`GET /api/v1/policies/:id/pdf`)、去签署入口(若存在待签合同)。
- 控制器: `PolicyController`。

### 4.8 features/contract — 合同列表 / 电子签署

- `ContractListPage`:待签/已签合同,`GET /api/v1/contracts?status=unsigned`。
- `ContractSignPage(contractId)`:嵌入 `AppWebView`,加载 `POST /api/v1/contracts/:id/sign-url` 返回的签署页;通过 JS channel 监听签署完成事件。
- 签署动作: 签名落库 `POST /api/v1/contracts/:id/sign`(或交由 H5 签署平台回调),完成后刷新合同状态引导查看。
- 差异:移动端原生 + WebView 双通道;Web/桌面见 §8。

### 4.9 features/profile — 我的

- 个人信息、实名认证状态、我的订单/保单/合同入口、联系客服、设置(主题/退出登录)、关于。
- 头像/昵称维护 `PATCH /api/v1/auth/me`(预留 domain 扩展)。

---

## 5. 关键数据模型 Dart 类

放在各 feature `domain/` 下,配 `freezed` + `json_serializable`。这里给出字段与语义(与后端实体对齐)。

```dart
// ============ auth ============
@freezed
class User with _$User {
  const factory User({
    required String id,
    required String mobile,          // 手机号
    String? nickname,
    String? avatarUrl,
    int? realNameVerified,           // 实名认证状态 0未 1通过
    String? idCardNo,                // 脱敏
    DateTime? createdAt,
  }) = _User;
  factory User.fromJson(Map<String, dynamic> json) => _$UserFromJson(json);
}

@freezed
class AuthSession with _$AuthSession {
  const factory AuthSession({
    required String accessToken,
    required String refreshToken,
    required User user,
  }) = _AuthSession;
}
```

```dart
// ============ product ============
@freezed
class Product with _$Product {
  const factory Product({
    required String id,
    required String name,
    required String category,        // 如 寿险/健康险/意外险/车险
    required String type,            // 险种类型码
    required String description,
    required List<String> coverages, // 保障责任
    required String insurer,         // 承保公司
    required num premiumFrom,        // 起保费
    required String currency,        // CNY
    required int terms,              // 保障期选项
    required int soldCount,
    required double rating,
    required bool isFeatured,
    required bool isHot,
    List<ProductPlan>? plans,        // 可选保障方案档位
    DateTime? createdAt,
  }) = _Product;
  factory Product.fromJson(Map<String, dynamic> json) => _$ProductFromJson(json);
}

@freezed
class ProductPlan with _$ProductPlan {
  const factory ProductPlan({
    required String id,
    required String name,            // 档位名
    required num premium,
    required int coverageAmount,     // 保额
    required int term,
    required Map<String, dynamic> benefits, // 权益
  }) = _ProductPlan;
}
```

```dart
// ============ quote ============
@freezed
class Quote with _$Quote {
  const factory Quote({
    required String id,
    required String productId,
    required String insuredName,     // 被保人
    String? insuredIdCard,
    required String beneficiary,
    required num premium,
    required String currency,
    required int term,               // 保障期
    required int coverageAmount,
    required DateTime expiresAt,     // 报价有效期
    required String status,          // draft / confirmed / expired
    List<HealthDeclarationItem>? healthDeclarations,
    DateTime? createdAt,
  }) = _Quote;
  factory Quote.fromJson(Map<String, dynamic> json) => _$QuoteFromJson(json);
}
```

```dart
// ============ order ============
@freezed
class Order with _$Order {
  const factory Order({
    required String id,
    required String orderNo,         // 订单号
    required String quoteId,
    required String productId,
    required String productName,
    required num amount,             // 应付金额
    required String currency,
    required String status,          // created / paying / paid / cancelled
    String? paymentId,
    required DateTime createdAt,
    DateTime? paidAt,
  }) = _Order;
  factory Order.fromJson(Map<String, dynamic> json) => _$OrderFromJson(json);
}

@freezed
class Payment with _$Payment {
  const factory Payment({
    required String id,
    required String orderId,
    required num amount,
    required String channel,         // wechat / alipay / unionpay / balance
    required String status,          // pending / success / failed / refunding
    required String payUrl,          // H5/收银台 url(Web 用)
    String? tradeNo,
    DateTime? paidAt,
  }) = _Payment;
  factory Payment.fromJson(Map<String, dynamic> json) => _$PaymentFromJson(json);
}
```

```dart
// ============ policy ============
@freezed
class Policy with _$Policy {
  const factory Policy({
    required String id,
    required String policyNo,        // 保单号
    required String orderId,
    required String productId,
    required String productName,
    required String insuredName,
    required String holderName,      // 投保人
    required num premium,
    required String status,          // active / awaiting / expired / cancelled
    required DateTime startDate,
    required DateTime endDate,
    String? pdfUrl,
    List<String>? benefits,
    DateTime? createdAt,
  }) = _Policy;
  factory Policy.fromJson(Map<String, dynamic> json) => _$PolicyFromJson(json);
}
```

```dart
// ============ contract ============
@freezed
class Contract with _$Contract {
  const factory Contract({
    required String id,
    required String contractNo,
    required String policyId,
    required String title,
    required String status,          // unsigned / signed / invalid
    required String signUrl,         // 电子签署页 url
    DateTime? signedAt,
    required DateTime createdAt,
  }) = _Contract;
  factory Contract.fromJson(Map<String, dynamic> json) => _$ContractFromJson(json);
}
```

> 说明:模型字段统一加 `required` 强约束 + `fromJson` 容错(缺省给默认),避免后端字段调整导致崩溃。金额统一用 `num`(或转 `int` 分),前端展示 `formatters.dart` 转字符串。

---

## 6. 统一响应处理

### 6.1 ResponseEnvelope 模型

```dart
@freezed
class BaseResponse<T> with _$BaseResponse<T> {
  const factory BaseResponse({
    required int code,               // 0 成功,非 0 业务错误
    required String message,
    T? data,
    String? traceId,                 // 链路追踪,错误排查必备
  }) = _BaseResponse;
}
```

### 6.2 错误映射

| 条件 | 抛出的异常 | UI 表现 |
|---|---|---|
| `code == 0` | 无,正常返回 `data` | — |
| HTTP 4xx / `code != 0`(业务) | `BizException(code, message, traceId)` | 顶部 SnackBar 或表单内联展示 message |
| HTTP 401 / 业务码 401 | 由 TokenRefresh 处理 | 静默刷新,失败 → 登出引导登录 |
| 超时 / 无网 | `NetworkException` | 全屏 error 态 + 重试 |
| 解析异常 | `ParseException` / `ApiException` | error 态 |
| 5xx | `ServerException` | 提示稍后重试,带 traceId |

```dart
// ErrorInterceptor 核心逻辑
Future<void> onError(DioException e, handler) async {
  if (e.type == DioExceptionType.connectionTimeout || e.type == DioExceptionType.connectionError) {
    throw NetworkException('网络连接失败,请检查网络', cause: e);
  }
  final res = e.response;
  final envelope = res != null ? BaseResponse.fromEnvelope(res.data) : null;
  if (envelope != null && envelope.code != 0) {
    throw BizException(envelope.code, envelope.message, envelope.traceId);
  }
  // 5xx 等
  throw ServerException('服务异常(${res?.statusCode}) ,traceId: ${res?.headers['x-trace-id']}');
}
```

### 6.3 loading / empty / error 三态

用 Riverpod `AsyncValue<T>` 直接驱动三态组件:

```dart
class BaseStateView<T> extends ConsumerWidget {
  const BaseStateView({super.key, required this.value, required this.builder, this.onRetry});
  final AsyncValue<T> value;
  final Widget Function(T data) builder;
  final VoidCallback? onRetry;

  @override
  Widget build(context, ref) => switch (value) {
    AsyncData(:final value) => value == null ? const EmptyView() : builder(value),
    AsyncError(:final error) => ErrorView(reason: error.toString(), onRetry: onRetry),
    _ => const LoadingView(),
  };
}
```

- `LoadingView`:居中转圈(骨架屏可选)。
- `EmptyView`:图标 + 文案 + 可选操作(如「去逛逛」)。
- `ErrorView`:错误文案 + 重试按钮;提供 `traceId` 展示便于反馈。
- 列表分页用 `AsyncNotifier` 维护 `items / hasMore / loadingMore / error`,加载更多与首屏错误分离处理。

---

## 7. 环境配置

### 7.1 baseUrl 策略:编译期 + 运行期

多端(含 Web 部署)建议**编译期注入**,避免 Web 打包把 prod 地址泄漏成明文常量。

| 方案 | 方式 | 适用 |
|---|---|---|
| `--dart-define`(推荐) | `API_BASE_URL=https://api.ins.com`, `APP_ENV=prod` | 构建时分环境 |
| `flutter_dotenv` | `.env.dev` / `.env.prod`,运行时读取 | 需打包期处理,dart-define 更硬 |
| 二者结合 | dart-define 覆盖,dotenv 兜底 | 灵活 |

```yaml
# pubspec.yaml
# .env.dev / .env.prod 用 dart-define 注入,运行时 AppConfig 读取
```

```dart
class AppConfig {
  static const _definedBase = String.fromEnvironment('API_BASE_URL');
  static const appEnv = String.fromEnvironment('APP_ENV', defaultValue: 'dev');
  static String get apiBaseUrl {
    if (_definedBase.isNotEmpty) return _definedBase;
    return appEnv == 'prod' ? _prodBase : _devBase;
  }
  static const _devBase  = 'https://dev-api.ins-service.com/api/v1';
  static const _prodBase = 'https://api.ins-service.com/api/v1';
  static const apiTimeout = Duration(seconds: 15);
}
```

构建命令示例:

```bash
# dev
flutter run --dart-define=APP_ENV=dev --dart-define=API_BASE_URL=https://dev-api.ins-service.com/api/v1
# prod
flutter build apk --release --dart-define=APP_ENV=prod --dart-define=API_BASE_URL=https://api.ins-service.com/api/v1
# web
flutter build web --release --dart-define=APP_ENV=prod
```

> 注意:Web 端 baseUrl 需含 `/api/v1` 前缀,且本地开发接入后端需配 CORS(或 dev 起 proxy)。

---

## 8. 多端差异处理

统一用 `shared/platform/platform_info.dart` 探测 `kIsWeb` + `Platform.isXXX`,逻辑分支集中在**平台能力层**,业务层不散落 `if(kIsWeb)`。

### 8.1 支付差异

| 端 | 方案 |
|---|---|
| iOS / Android | `payment_launcher.dart` 拉起原生 SDK(微信/支付宝)或 App 间跳转;结束回调 + 服务端确认 | 
| Web | 走 `payUrl` H5 收银台,新窗口/iframe 打开,支付后 JS 回调 + 轮询状态兜底 |
| 桌面(Windows/macOS) | 展示**付款二维码**扫码支付(微信/支付宝),轮询 `GET /payments/:id/status` |

统一出口:定于 `PaymentLauncher` 抽象接口,按平台实现(SDK / 网页 / 二维码),`PaymentPage` 不关心平台。支付结果一律以**服务端轮询确认**为准(防客户端伪造)。

### 8.2 电子签署差异

| 端 | 方案 |
|---|---|
| iOS / Android | 原生 WebView 加载 `signUrl`,JS 回传签署完成事件,再调 `POST /contracts/:id/sign` 落库 |
| Web | 直接新标签页/iframe 打开签署页,H5 平台回调通知 + 本地轮询刷新合同状态 |
| 桌面 | 内嵌 `AppWebView`(webview_flutter 桌面支持)或打开系统浏览器 |

签名合规:签署结果以服务端回调/主动查询为准;客户端只负责展示与刷新。

### 8.3 布局差异(响应式)

- `responsive_widget.dart` 提供 `isMobile / isTablet / isDesktopMaxWidth` 断点(如 `<=600 / <=1000 / >1000`)。
- 移动端:底部 Tab + 全屏页面导航。
- 桌面/平板:`desktop_scaffold.dart` 左侧抽屉/侧边栏导航(首页/产品/订单/保单/合同/我的),内容区自适应;列表页桌面可两列卡片。

### 8.4 其余差异

- **Web 刷新/深链**: go_router URL 即状态,刷新恢复当前页;登录态从 secure_storage(Web 用 localStorage 封装)读取。
- **Web 导航**: 禁用原生返回手势影响,统一走路由。
- **PDF 查看**: 移动端 `printing` 预览/分享,Web 直接新标签打开 `pdfUrl`,桌面 `printing`。
- **分享**: 移动/桌面系统能力差异,封装 `share_service` 抽象。

---

## 9. 核心业务流(投保→支付→保单→签署)

```
产品浏览/搜索 → 投保表单(报价) → 订单确认 → 支付 → 保单生成 → 合同签署
```

### Step 1 投保报价

```
ProductDetailPage --点击投保--> QuoteFormPage
  多步骤表单校验(validators.dart)
  POST /api/v1/quotes { productId, insuredInfo, planId, beneficiaries, healthDeclarations }
  → Quote{ id, premium, expiresAt }
  --确认--> /order/confirm/:quoteId
```

### Step 2 订单生成

```
OrderConfirmPage
  POST /api/v1/orders { quoteId, ... }
  → Order{ id, orderNo, amount, status: created }
  --去支付--> /payment/:orderId
```

### Step 3 支付

```
PaymentPage
  选择渠道 → POST /api/v1/payments { orderId, channel } → Payment{ payUrl/签名参数 }
  移动:拉起SDK / Web:收银台 / 桌面:二维码 → 用户完成
  轮询 GET /api/v1/payments/:orderId/status → success
      → 服务端后台触发生成 Policy
  --结果--> PaymentResultPage → /policy/:id
```

### Step 4 保单生成与查看

```
支付成功 → 服务端生成 Policy(active)
PolicyListPage  → PolicyDetailPage(下载PDF、缴费记录)
   若存在待签合同 → 引导 /contract/sign/:contractId
```

### Step 5 合同电子签署

```
ContractSignPage(contractId)
  加载 signUrl 到 AppWebView
  用户完成 H5 签署 → JS 回调 / 服务端通知
      → 刷新合同状态= signed,引导查看已签合同 PDF
```

> **状态一致性原则**: 每一步都以服务端最终确认为准;支付/签署这类异步操作一律「客户端展示 + 服务端轮询确认」,配合页面级的 loading/错误重试。

---

## 10. 脚手架落地清单

按依赖序落地,每项对应脚手架命令/步骤:

1. **工程初始化**
   `flutter create --org com.insco --platforms=android,ios,web,macos,windows,linux app`
   配 `pubspec.yaml`,引入: `flutter_riverpod`, `riverpod_annotation`, `go_router`, `dio`, `freezed_annotation`, `json_annotation`, `flutter_secure_storage`, `shared_preferences`, `webview_flutter`, `flutter_dotenv`, `logger`, `intl` / `flutter_localizations`, `printing`。dev 依赖:`build_runner`, `freezed`, `json_serializable`, `riverpod_generator`, `mocktail`, `integration_test`。
   `flutter pub add` / 手写。

2. **core 骨架**: `config/`(AppConfig)、`network/`(dio + 拦截器 + api_client + exceptions + base_response)、`router/`、`theme/`、`utils/`(三态 + validators + formatters)。

3. **storage**: `TokenStorage` 抽象 + secure/prefs 实现,注入 Riverpod。

4. **auth feature 先行**: 登录/注册打通,验证拦截器 + 刷新 + 路由守卫闭环。

5. **按 §9 流程逐 feature 落地**: product → quote → order → payment → policy → contract → home/profile。

6. **响应式层**: `responsive_widget.dart` + `desktop_scaffold.dart`,补 Web/桌面细节。

7. **测试**: 拦截器单测、模型 fromJson 单测、三态组件 widget 测试、登录/投保/支付关键流 integration_test。

8. **环境与 CI**: `--dart-define` 构建脚本;GitHub Action 跑 analyze + test + 多端 build。

---

## 附:A 端基础组件与命名约定

- 通用组件放 `shared/widgets/`:`LoadingView` / `EmptyView` / `ErrorView` / `AppWebView` / `AppButton` / `SectionHeader`。
- 命名:`XxxPage`(路由页)、`XxxController`(Riverpod AsyncNotifier)、`XxxProvider`、`XxxRepository`(抽象在 domain,`XxxRepositoryImpl` 在 data)。
- 路由常量集中 `app_router.dart`;API 路径常量在 `core/constants/api_paths.dart`(`/auth/login` / `/products` / `/quotes` / `/orders` / `/payments` / `/policies` / `/contracts` / `/search`)。
- 错误处理统一在 `error_interceptor.dart`,UI 层不 catch DioException,只 catch `ApiException` 系。
