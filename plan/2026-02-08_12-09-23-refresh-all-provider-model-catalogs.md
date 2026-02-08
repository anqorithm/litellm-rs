---
mode: plan
cwd: /Users/lifcc/Desktop/code/AI/gateway/litellm-rs
task: Refresh all provider model catalogs and pricing metadata to latest official versions
complexity: complex
planning_method: builtin
created_at: 2026-02-08T04:09:23Z
---

# Plan: Refresh All Provider Model Catalogs

🎯 任务概述
在不引入设计分叉、不删除兼容别名的前提下，分批更新所有模型厂商的模型清单、能力与定价映射，并保证每批可独立验证和提交。目标是把当前仓库从“局部更新（OpenAI/Bedrock）”推进到“全厂商覆盖且可持续维护”。

📋 执行计划
1. 建立厂商盘点与分层优先级（Tier 1/2/3），冻结本轮范围与非目标。
2. 为每个厂商建立“官方来源映射表”（文档/API/发布日志）并标记抓取方式（可脚本化/需人工核对）。
3. 统一模型元数据更新规范：模型 ID、别名、上下文窗口、功能能力、默认推荐模型、定价字段。
4. 实施 Wave 1（核心厂商）：Anthropic、Gemini/Vertex、Azure OpenAI、xAI、DeepSeek、Mistral、Cohere、Qwen、Meta Llama。
5. 实施 Wave 2（平台与聚合）：OpenRouter、VLLM/Hosted VLLM、Ollama、Github/Copilot、Snowflake/Watsonx 等高使用入口。
6. 实施 Wave 3（长尾厂商）：其余 providers 逐个补齐模型与别名，保持向后兼容，不移除现有旧 ID。
7. 每个厂商完成后执行同一验证矩阵：编译、目标测试、定价回归、模型识别回归，并单厂商单提交。
8. 完成全量回归：全局 `cargo check`、关键 e2e、跨 provider 路由场景、成本计算一致性检查。
9. 输出最终对账与迁移说明：新增/变更/弃用模型清单、风险项、后续自动同步建议。

⚠️ 风险与注意事项
- 官方文档存在动态页面、Cloudflare 或权限限制，可能导致自动抓取不稳定。
- 部分厂商只提供别名或 region-specific ID，易出现重复/冲突映射。
- 定价发布时间与模型发布时间不同步，可能出现“模型已更新但价格未定”的窗口期。
- 一次性大改会降低可审查性，因此必须按厂商分批提交并保持每批可回滚。

📎 参考
- `src/core/providers`
- `src/core/providers/openai/models.rs`
- `src/core/providers/bedrock/model_config.rs`
- `src/core/providers/bedrock/utils/cost.rs`
- `src/core/cost/calculator.rs`
- `src/core/cost/utils.rs`
- `src/utils/ai/models/pricing.rs`
- `src/utils/ai/models/utils.rs`
- `src/utils/ai/tokens.rs`
