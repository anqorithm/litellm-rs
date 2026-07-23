# Tech Spec

## Linked Issue

GH-1112 / #1112

## Product Spec

见 `specs/GH1112/product.md`。

## Codebase Context

以下锚点已在 `origin/main@671282f265fdf7ba4a5b1c8d0646e175903faabb` 核验。

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| Gemini registry/types | `src/core/providers/gemini/models/mod.rs:129-206` | `ModelSpec` 同时承载模型信息、能力、价格和限制；registry 以 `HashMap` 存储且 `list_models()` 直接返回 values。 | 当前第一套事实源，列表顺序不稳定。 |
| Gemini fuzzy family detection | `src/core/providers/gemini/models/mod.rs:210-278` | `from_model_name` 通过 lowercase + `contains` 推断 family。 | B-001/B-014 要把 executable model lookup 与 family 猜测解耦。 |
| Gemini provider | `src/core/providers/gemini/provider.rs:47-116,141-181` | provider 从 registry 构造模型列表；validation、supported params 与 mapping 分散。 | 共享 contract 的第一个 consumer。 |
| Gemini final request body | `src/core/providers/gemini/client.rs:93-164,273-328` | public client entry points call a local `transform_chat_request` that serializes generation fields independently of provider validation. | B-008/B-010 must bind the actual Developer `generationConfig`, not only provider-level preflight. |
| Gemini catalog data | `src/core/providers/gemini/models/catalog/{mod.rs,gemini25.rs,gemini3.rs,gemini31.rs,gemini35.rs,legacy.rs}` | 模型记录按 family 分文件注册到 Gemini-owned registry。 | 数据需迁移到 provider-neutral Google owner，避免 alias wrapper。 |
| Vertex model enum/capabilities | `src/core/providers/vertex_ai/mod.rs:138-203,205-480` | Gemini、partner 与 `Custom(String)` 混在一个 enum；capability/limit 由独立 match 表维护。 | 当前第二套事实源与 Custom 默认语义。 |
| Vertex parser | `src/core/providers/vertex_ai/mod.rs:482-639` | 大量 lowercase substring 分支，未知值返回 `Custom`。 | B-001/B-006/B-007 的直接根因。 |
| Vertex advertised models | `src/core/providers/vertex_ai/client.rs:285-335` | `models()` 另建只含 Gemini 1.5 的静态 `ModelInfo` 表。 | 当前第三套事实源。 |
| Vertex request dispatch | `src/core/providers/vertex_ai/client.rs:120-153,431-507` | chat 先 fuzzy parse，再按 enum 选择 transformer；supported params 又按字符串 `contains("gemini")`。 | exact gate 与 shared request contract 的执行点。 |
| Vertex health probe | `src/core/providers/vertex_ai/client/health.rs:5-16` | health check hard-codes `gemini-1.5-flash` and performs auth/network without consulting Vertex availability. | A retired or unavailable probe must not bypass the exact catalog gate. |
| Vertex Gemini body contract | `src/core/providers/vertex_ai/transformers.rs:29-100,541-762`、`src/core/providers/vertex_ai/common_utils.rs:81-166,246-281` | `GeminiTransformer` 直接从 `ChatRequest` 构造 `GenerationConfig`；另有独立 `validate_parameters`，未消费共享模型请求契约。 | B-008/B-010 必须覆盖实际 request body，而不只覆盖 client 参数 map。 |
| Vertex batch parser consumer | `src/core/providers/vertex_ai/batches/mod.rs:214-285` | batch request/response path 调用 `parse_vertex_model`，并以 model string 判断 Gemini。 | parser 改为 `Result` 后必须机械迁移该 consumer，保持 batch 产品行为不变。 |
| Vertex URL | `src/core/providers/vertex_ai/client/url.rs:14-61` | URL 根据 enum 类别和 model ID 选择 Google/partner/custom 路径。 | Custom fallback 必须不可达，endpoint ownership 保留。 |
| Developer auth | `src/core/providers/mod.rs:141-179`、`src/core/providers/gemini/client.rs:70-91`、`src/core/providers/gemini/config.rs:14-88` | native Developer URL 校验 route segment 后把 API key 放入 query；`GeminiConfig` derives raw `Debug`. | B-011/B-013 的 transport 边界必须保留，同时 production Debug 必须 redact key。 |
| Vertex auth | `src/core/providers/vertex_ai/client.rs:81-94`、`src/core/providers/vertex_ai/auth.rs:17-86,197-224` | `VertexAuth` 获取 access token，client 设置 Bearer header；credential structs derive raw `Debug`. | B-012/B-013 的 transport 边界必须保留，同时 production credential Debug 必须 redact secrets。 |
| Compatibility consumers | `src/utils/ai/models/pricing.rs:360-381`、`src/utils/ai/models/utils.rs:210-505`、`src/utils/ai/models/utils_tests.rs:1-176`、`src/core/providers/shared.rs:26-63`、`src/core/providers/gemini/mod.rs:32-67` | utility capability logic and shared context-window helper carry Gemini substring/match tables in addition to direct registry users. | 迁移时必须改为 canonical Google query，不能保留第二份 capability/limit wrapper registry。 |

## Planned Changes

```specrail-planned-changes
{
  "issue": 1112,
  "complete": true,
  "paths": [
    "src/core/providers/mod.rs",
    "src/core/providers/google/mod.rs",
    "src/core/providers/google/models/mod.rs",
    "src/core/providers/google/models/registry.rs",
    "src/core/providers/google/models/request_contract.rs",
    "src/core/providers/google/models/catalog/mod.rs",
    "src/core/providers/google/models/catalog/gemini25.rs",
    "src/core/providers/google/models/catalog/gemini3.rs",
    "src/core/providers/google/models/catalog/gemini31.rs",
    "src/core/providers/google/models/catalog/gemini35.rs",
    "src/core/providers/google/models/catalog/legacy.rs",
    "src/core/providers/google/models/tests.rs",
    "src/core/providers/gemini/mod.rs",
    "src/core/providers/gemini/client.rs",
    "src/core/providers/gemini/config.rs",
    "src/core/providers/gemini/models/mod.rs",
    "src/core/providers/gemini/models/catalog/mod.rs",
    "src/core/providers/gemini/models/catalog/gemini25.rs",
    "src/core/providers/gemini/models/catalog/gemini3.rs",
    "src/core/providers/gemini/models/catalog/gemini31.rs",
    "src/core/providers/gemini/models/catalog/gemini35.rs",
    "src/core/providers/gemini/models/catalog/legacy.rs",
    "src/core/providers/gemini/provider.rs",
    "src/core/providers/gemini/provider_tests.rs",
    "src/core/providers/vertex_ai/mod.rs",
    "src/core/providers/vertex_ai/batches/mod.rs",
    "src/core/providers/vertex_ai/client.rs",
    "src/core/providers/vertex_ai/client/health.rs",
    "src/core/providers/vertex_ai/client/url.rs",
    "src/core/providers/vertex_ai/client_tests.rs",
    "src/core/providers/vertex_ai/common_utils.rs",
    "src/core/providers/vertex_ai/tests.rs",
    "src/core/providers/vertex_ai/transformers.rs",
    "src/core/providers/vertex_ai/transformers/split_tests.rs",
    "src/core/providers/vertex_ai/auth.rs",
    "src/core/providers/shared.rs",
    "src/utils/ai/models/pricing.rs",
    "src/utils/ai/models/utils.rs",
    "src/utils/ai/models/utils_tests.rs"
  ],
  "spec_refs": [
    "B-001", "B-002", "B-003", "B-004", "B-005", "B-006",
    "B-007", "B-008", "B-009", "B-010", "B-011", "B-012",
    "B-013", "B-014", "B-015", "B-016", "B-017", "B-018"
  ]
}
```

上述清单包含新 neutral owner、被删除的旧 Gemini-owned catalog 路径以及全部已知
consumer。实施发现必须改清单外 production path 时，先提交 spec amendment；测试 helper
仅可在对应 production owner 的既有 test path 内增加。

## 设计方案

### 1. Provider-neutral Google catalog

新增 crate-private `core::providers::google::models`，拆为：

- `registry.rs`：`GoogleModelId`、`GoogleModelSpec`、lifecycle、capabilities、limits、
  `GoogleAvailability` 和 immutable registry；
- `request_contract.rs`：允许参数闭集、数值边界和模型特定 illegal-state policy；
- `catalog/*`：从 Gemini-owned 目录移动的静态数据；
- `tests.rs`：duplicate、missing evidence、overlay、ordering 与 concurrent-read fixtures。

每条记录以 exact ID 为 key，并分别携带 Developer/Vertex availability：

```text
GoogleModelSpec
  canonical_id
  lifecycle + source_ref
  capabilities + limits
  request_contract
  availability:
    developer_api: unavailable | available(source_ref)
    vertex_ai: unavailable | available(source_ref)
```

registry 初始化先验证全部记录，再一次性发布不可变快照。`models_for(surface)` 过滤
availability 后按 ID 排序；exact lookup 不做 lowercase 或 contains。显式 alias 使用单独
映射并在初始化时做 collision 检查，本 tranche 不从现有 fuzzy parser 生成 alias。

价格字段为兼容现有 consumer 可以暂留记录中，但本 issue 不改变 pricing lookup 的未知
语义、单位或 authority；#1113 后续可以用相同 canonical key 收敛 pricing。目录不得依赖
auth、HTTP client 或 provider config。

### 2. 删除 Gemini-owned duplicate catalog

将 `gemini/models/catalog/*` 的数据移动到 neutral owner，并把原 `models/mod.rs` 拆为
`registry.rs` / `request_contract.rs` / `tests.rs`。旧 `GeminiModelRegistry` 和旧 catalog
路径删除，不保留 type alias、wrapper registry 或双写兼容层。

`gemini/mod.rs`、pricing utility、model utility implementation/tests 与
`core::providers::shared::gemini_context_window` 直接消费 canonical Google API。
外部可观察 helper 名若是 public compatibility surface，可保留函数名，但实现必须直接
查询 single registry，且不得暴露第二种 registry/type identity。

### 3. Gemini consumer 与 request contract

`GeminiProvider::new` 只公开 Developer availability 的 `ModelInfo`，排序由 registry
保证。`validate_request` 先 exact lookup + Developer overlay，再依次执行 common validation
和 shared request contract；`get_supported_openai_params` 从同一 contract 返回稳定闭集，
`map_openai_params` 只映射 contract 已允许字段。

`GeminiClient::transform_chat_request` 接收 exact Developer model lookup 得到的
`GoogleRequestContract` decision，只把已允许且验证通过的字段写入最终
`generationConfig`。`chat`/`chat_stream` direct client entry points 也必须先执行同一
preflight；不得因为绕过 `GeminiProvider` facade 而发送 contract-disallowed field。

不在本 issue 添加 #1108 新模型/新 lifecycle 数据；当前记录只做 ownership-preserving
迁移。若现有记录缺少 Developer/Vertex 来源证据，默认 unavailable 并由 B-017 diff fixture
明确记录，不得猜测补 available。

### 4. Vertex exact classification

把 Google Gemini chat model 从 `VertexAIModel` 的重复 variant/match 表中移除，改为
`VertexModel::Google(GoogleModelId)`；partner model 保留独立 exact catalog/enum。
`parse_vertex_model` 改为返回 `Result<VertexModel, ProviderError>`：先 exact Google+Vertex
overlay lookup，再 exact partner lookup，否则 typed model-not-found。

`Custom(String)` 不再是 chat parse fallback；原 custom URL 分支对 chat 不可达。custom
`api_base` 只替换 transport base，不放宽 model gate。Embedding/image/model-garden 的独立
custom model enum 不在本次修改范围。

`batches/mod.rs` 是 `parse_vertex_model` 的现有 consumer，随 parser signature 做机械迁移：
使用相同 exact classification 并传播 typed error，不新增 batch capability、wire 字段或
生命周期声明。这样每个 task head 都能编译，也不会用 compatibility wrapper 保留 fuzzy
语义。

`VertexAIProvider::models()` 由 `models_for(VertexAi)` 生成 Google `ModelInfo`，与 partner
models 以 exact ID 合并、去重和排序；删除当前 Gemini 1.5 静态表。Vertex request
dispatch、capability 和 limits 对 Google variant 读取共享 spec，对 partner variant 读取
partner owner。

`client/health.rs` 从 Vertex-available catalog 的稳定、明确 health-capable fixture 选择
probe model；若没有合法 probe，则在 credential/token/network 前返回 typed unhealthy
evidence。不得硬编码可能 retired/unavailable 的 Google model，也不得从 Developer overlay
推断 Vertex probe availability。

### 5. Shared request contract without shared transport

`GoogleRequestContract` 提供纯函数：

- `supported_openai_params()`；
- `validate_chat_request(&ChatRequest)`；
- `map_generation_fields(...)` 所需的 provider-neutral决策结果。

Gemini 与 Vertex transformer 可以保留 wire 命名差异，但只能消费该决策；禁止各自再用
model substring 建参数 allowlist。任何不允许字段、越界值或非法 turn state 产生
non-retryable invalid-request，发生在 URL 构造、token 获取、预算副作用和 HTTP send 前。

Vertex 的 `common_utils::GenerationConfig` 继续只是 Vertex wire DTO，不成为第二份行为
contract；`validate_parameters` 必须删除或改为直接委托 `GoogleRequestContract`。
`GeminiTransformer::transform_chat_request` 接收已经通过 exact model/overlay lookup 的
contract decision，并只序列化被允许的 generation fields。对应 inline 与 split tests 同时
覆盖 supported-params 声明、validation verdict 和最终 `generationConfig` 三者一致。

### 6. Authentication and endpoint isolation

catalog 与 request contract 不接收 credential、header、query、base URL、project、region
或 API version。Gemini Developer 请求继续由现有 native URL helper写 query key；Vertex
继续通过 `VertexAuth` 生成 Bearer header。测试使用互不相同的 sentinel credential，捕获
请求并断言：

- Developer URL 只有 query sentinel，无 Vertex Bearer；
- Vertex header 只有 Bearer sentinel，无 Developer query key；
- model-not-found/validation errors、Debug/Display 和 catalog snapshot 都无 sentinel；
- rejected request 的 auth/token/network counters 为零。

为使该 gate 可执行，`GeminiConfig`、`VertexCredentials`、`ServiceAccountKey` 与
`AuthorizedUserCredentials` 使用显式 redacted `Debug`（并约束任何 Display/log adapter）；
可以显示非敏感类型/状态，但 API key、private key、client secret、refresh token 与 access
token 必须统一替换为固定 redaction marker。Gemini 与 Vertex credential 类型仍各自所有，
不得借 redaction 合并 config 或认证路径。

本 issue 不把 legacy `GeminiConfig::new_vertex_ai` 变成认证桥接层；Vertex transport 仍由
`VertexAIProvider` 所有。若后续移除/迁移该 public constructor，需要单独兼容性 spec。

### 7. Migration and fail-closed verification

在迁移前生成现有 exact advertised-ID snapshot；迁移后逐项标注：保留、因无对应
availability 证据停止广告、或属于 #1108 后续刷新。禁止 golden snapshot 自动更新却无
解释。negative fixtures 必须构造类型合法但业务非法的 model/contract，不以 serde/type
失败代替 gate 验证。

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| B-001 | Google exact registry + Vertex parser | unit table：exact 成功，prefix/suffix/case/substring 全部 typed reject。 |
| B-002 | `GoogleModelSpec` + deleted old owners | `cargo check --locked` 证明 provider、utility 与 shared-limit consumer 只依赖唯一 registry type；双 provider parity fixture 读取同一 spec 并得到一致核心 metadata。 |
| B-003 | availability overlay | Developer-only、Vertex-only、both、neither fixture 分别比较两个 provider list。 |
| B-004 | registry/list builders | 重复构建与并发读 100 次均得到同一排序、无重复 ID。 |
| B-005 | lifecycle gate | retired/unverified fixture 不公开且 upstream counter=0。 |
| B-006 | Vertex exact parser | `cargo test --locked vertex_ai_model_exact` 覆盖空、case、prefix、suffix 和未知。 |
| B-007 | Vertex chat dispatch | custom base + unknown model 仍 model-not-found；URL/auth/network counters=0。 |
| B-008 | shared request contract consumers + Gemini client + Vertex transformer | facade/direct-client supported params、validation、最终 body projection 的 table-driven parity test。 |
| B-009 | request preflight | missing contract、unsupported key、range、illegal state 均 invalid-request 且 network=0。 |
| B-010 | overlapping model parity | 同一 `ChatRequest` matrix 对 Developer/Vertex 得到相同 provider-neutral verdict。 |
| B-011 | Developer auth boundary + `GeminiConfig` redacted Debug | loopback capture：query key 存在、无 Bearer；adversarial Debug/Display/error/log/catalog 无 sentinel。 |
| B-012 | Vertex auth boundary + credential redacted Debug | loopback capture：Bearer 存在、无 query key；credential Debug/Display/log 不泄露 private/client/refresh/access secret，且不读取 Gemini API-key field。 |
| B-013 | crate-private catalog API + dependency boundary | API 只接收静态 model records、不接收 auth/config/client；目录查询 fixture 的 auth/network counters=0；独立 reviewer 核对依赖边界。 |
| B-014 | alias map | empty alias set + collision/undeclared alias negative fixtures；fuzzy input 不命中。 |
| B-015 | registry validation | duplicate/missing lifecycle/contract/evidence fixtures 使初始化返回 error，不发布部分列表。 |
| B-016 | immutable snapshot | concurrent read test 证明所有 reader 共享同一 snapshot；crate-private API 只暴露 read methods。 |
| B-017 | migration snapshot | before/after exact ID fixture，所有删除项带 lifecycle/availability disposition。 |
| B-018 | aggregate regression | positive/negative fixture count guard、production credential redaction、health/direct-client upstream=0 与 full provider tests。 |

## 数据流

```text
canonical Google catalog (immutable, exact IDs)
  ├─ filter Developer availability
  │    -> Gemini models()
  │    -> shared request-contract preflight
  │    -> Gemini transformer
  │    -> Developer endpoint + query API key
  └─ filter Vertex availability
       -> Vertex models()
       -> exact Google/partner classification
       -> shared request-contract preflight
       -> Vertex transformer
       -> Vertex endpoint + VertexAuth Bearer
```

catalog 查询是纯内存操作，不触发认证、HTTP 或持久化。provider-specific transport 在
preflight 成功后才执行。

## 备选方案

1. **Vertex 直接 import `gemini::models`**：拒绝。虽然减少一份表，但把 shared truth
   放在单 provider owner 下，继续造成依赖方向和认证概念混淆。
2. **保留三份表并加一致性测试**：拒绝。测试只能发现漂移，不能消除多写 authority。
3. **继续 substring parser，增加更多顺序分支**：拒绝。新 family 会改变匹配优先级，
   任意前后缀仍可误分类。
4. **把 unknown 全部当 Custom endpoint model**：拒绝。custom base 是 transport 配置，
   不是 capability/lifecycle/contract 证据。
5. **顺带合并 pricing 与 tool wire**：拒绝。分别属于 #1113 与 #1111，会扩大风险与
   acceptance surface。

## 风险

- **Security**：认证边界改动属于 SEC-11 高风险面；禁止 catalog 接触 secret，auth tests
  和人工 exact-head review 必须覆盖 query/header/error/log。
- **Compatibility**：fuzzy/unknown 输入会从可能误路由变为确定性拒绝；这是预期收紧，
  但必须列出 migration snapshot。
- **Data correctness**：现有 catalog 可能缺 Vertex availability 证据；默认 unavailable，
  不用 Developer 证据补齐。
- **Performance**：初始化校验 O(models + aliases)，查询 O(1)，排序只在 provider 初始化
  构造列表时发生；请求热路径不克隆完整 catalog。
- **Maintenance**：移动大 catalog 时容易保留旧 wrapper；删除旧类型后由 Rust 编译器迫使
  consumer 迁移，provider parity fixtures 与独立 review 阻止双 authority 回流。

## 测试计划

- [ ] Focused registry: `cargo test --locked google_model_catalog`
- [ ] Focused Gemini: `cargo test --locked gemini_provider`
- [ ] Focused Vertex: `cargo test --locked vertex_ai_model_exact`、`cargo test --locked vertex_ai_transformer`
- [ ] Auth isolation: `cargo test --locked google_auth_isolation`
- [ ] Utility compatibility: `cargo test --locked model_utils`
- [ ] Format/build: `cargo fmt --all -- --check && cargo check --locked`
- [ ] Strict lint: `cargo clippy --locked --all-targets -- -D warnings`
- [ ] Full suite: `cargo test --locked`
- [ ] Coverage artifact: `cargo llvm-cov --locked --all-features --workspace --branch --lcov --output-path artifacts/coverage/GH1112/lcov.info`；随后原样执行下方 exact-head gate。它以 `COVERAGE_BASE_SHA...HEAD` 的 Rust diff 为分母，要求新增可执行行 line coverage ≥80%，并要求新增 catalog/validation/exact-rejection/request-contract branch record 100% 命中；missing base、missing LCOV file、无新增可执行行、无关键分支、malformed record 或任一未命中都以非零退出。auth isolation 继续由 B-011/B-012/B-013 的行为 fixture 证明，不用普通 line report 冒充认证边界证明。
- [ ] SpecRail: `python3 checks/check_workflow.py --repo . --spec-dir specs/GH1112 && python3 checks/check_workflow.py --repo .`
- [ ] Diff integrity: `git diff --check`

full suite、strict Clippy 与 SpecRail gates 由 coordinator 在 exact implementation head 各执行
一次；reviewer 默认只做 inspection/focused checks，避免重复 target-dir 锁等待。

### Exact-head coverage gate

`COVERAGE_BASE_SHA` 必须是本 implementation tranche 从最新 `origin/main` 创建时记录的
40 位 commit，`COVERAGE_HEAD_SHA` 必须是 reviewer 将要审查的 exact head；禁止用会移动
的 branch name 或自动更新 golden。`cargo llvm-cov --branch` 必须与 gate 串行运行，且
两条命令绑定同一个 tracked-clean `HEAD`。gate artifact 保存 immutable base/head、
changed-source/line manifest、LCOV digest 和逐类别 branch 结果：

```bash
set -euo pipefail
test "${COVERAGE_BASE_SHA:-}" != "" &&
test "${COVERAGE_HEAD_SHA:-}" != "" &&
test "$(git rev-parse "$COVERAGE_BASE_SHA^{commit}")" = "$COVERAGE_BASE_SHA" &&
test "$(git rev-parse "$COVERAGE_HEAD_SHA^{commit}")" = "$COVERAGE_HEAD_SHA" &&
test "$(git rev-parse HEAD)" = "$COVERAGE_HEAD_SHA" &&
test -z "$(git status --porcelain --untracked-files=no)" &&
test -f artifacts/coverage/GH1112/lcov.info &&
python3 - "$COVERAGE_BASE_SHA" "$COVERAGE_HEAD_SHA" artifacts/coverage/GH1112/lcov.info <<'PY' | tee artifacts/coverage/GH1112/gate.json
from fnmatch import fnmatch
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

base, head, lcov_path = sys.argv[1:4]
if any(re.fullmatch(r"[0-9a-f]{40}", value) is None for value in (base, head)):
    raise SystemExit("coverage base/head must be full commit SHAs")

diff = subprocess.run(
    ["git", "diff", "--unified=0", "--no-color", f"{base}...{head}", "--", "*.rs"],
    check=True,
    text=True,
    capture_output=True,
).stdout
changed: dict[str, set[int]] = {}
current_path: str | None = None
for raw in diff.splitlines():
    if raw.startswith("+++ b/"):
        current_path = raw[6:]
        changed.setdefault(current_path, set())
        continue
    if raw.startswith("@@ ") and current_path is not None:
        match = re.search(r"\+(\d+)(?:,(\d+))?", raw)
        if match is None:
            raise SystemExit(f"malformed diff hunk: {raw}")
        start = int(match.group(1))
        count = int(match.group(2) or "1")
        changed[current_path].update(range(start, start + count))

root = Path.cwd().resolve()
line_hits: dict[tuple[str, int], int] = {}
branches: list[tuple[str, int, int]] = []
lcov_sources: set[str] = set()
source: str | None = None
for raw in Path(lcov_path).read_text(encoding="utf-8").splitlines():
    if raw.startswith("SF:"):
        value = Path(raw[3:])
        try:
            source = value.resolve().relative_to(root).as_posix() if value.is_absolute() else value.as_posix()
        except ValueError:
            source = None
        if source is not None:
            lcov_sources.add(source)
    elif raw.startswith("DA:") and source is not None:
        fields = raw[3:].split(",")
        if len(fields) < 2:
            raise SystemExit(f"malformed DA record: {raw}")
        line_hits[(source, int(fields[0]))] = int(fields[1])
    elif raw.startswith("BRDA:") and source is not None:
        fields = raw[5:].split(",")
        if len(fields) != 4:
            raise SystemExit(f"malformed BRDA record: {raw}")
        branches.append((source, int(fields[0]), 0 if fields[3] == "-" else int(fields[3])))

def is_test_source(path: str) -> bool:
    return (
        "/tests/" in path
        or path.endswith("/tests.rs")
        or path.endswith("_test.rs")
        or path.endswith("_tests.rs")
    )

changed_production_sources = {
    path for path, lines in changed.items()
    if lines
    and path.startswith("src/")
    and path.endswith(".rs")
    and not is_test_source(path)
}
missing_sources = sorted(changed_production_sources - lcov_sources)
if missing_sources:
    raise SystemExit(f"changed production sources missing from LCOV: {missing_sources}")

changed_lines = {
    key: hits for key, hits in line_hits.items()
    if key[0] in changed_production_sources
    and key[1] in changed.get(key[0], set())
}
if not changed_lines:
    raise SystemExit("no changed executable Rust lines found in LCOV")
covered_lines = sum(hits > 0 for hits in changed_lines.values())
line_percent = covered_lines * 100.0 / len(changed_lines)
if line_percent < 80.0:
    raise SystemExit(f"changed-line coverage {line_percent:.2f}% is below 80%")

critical_categories = {
    "catalog_validation": (
        "src/core/providers/google/models/registry.rs",
    ),
    "gemini_exact_rejection": (
        "src/core/providers/gemini/provider.rs",
    ),
    "vertex_exact_rejection": (
        "src/core/providers/vertex_ai/mod.rs",
        "src/core/providers/vertex_ai/batches/mod.rs",
        "src/core/providers/vertex_ai/client.rs",
    ),
    "request_contract": (
        "src/core/providers/google/models/request_contract.rs",
    ),
}
category_results: dict[str, dict[str, object]] = {}
for category, patterns in critical_categories.items():
    records = [
        record for record in branches
        if record[1] in changed.get(record[0], set())
        and any(fnmatch(record[0], pattern) for pattern in patterns)
    ]
    if not records:
        raise SystemExit(f"no changed branch records for critical category: {category}")
    uncovered = sorted({(path, line) for path, line, hits in records if hits <= 0})
    if uncovered:
        raise SystemExit(f"uncovered {category} branches: {uncovered}")
    category_results[category] = {"branches": len(records), "covered_percent": 100}

manifest = {
    path: sorted(lines)
    for path, lines in sorted(changed.items())
    if lines and path.endswith(".rs")
}
result = {
    "base_sha": base,
    "head_sha": head,
    "changed_manifest": manifest,
    "changed_line_coverage": round(line_percent, 2),
    "critical_categories": category_results,
    "lcov_sha256": hashlib.sha256(Path(lcov_path).read_bytes()).hexdigest(),
}
print(json.dumps(result, sort_keys=True))
PY
```

## 回滚方案

以 implementation PR 为单位回滚 neutral catalog 和两个 consumer 的迁移，恢复原三套表；
不得通过重新启用 fuzzy parser/Custom fallback 或 silent contract drop 做部分回滚。若只
有一个 consumer 失败，应整体回滚同一 atomic migration，避免短期双 authority。回滚不
修改 credential、endpoint 或用户配置，也不自动恢复已被证据判定 unavailable 的模型；
该决定需另行审查。
