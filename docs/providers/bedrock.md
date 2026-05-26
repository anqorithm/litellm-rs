# AWS Bedrock Provider

LiteLLM-RS routes `provider_type: "bedrock"` through the native AWS Bedrock
runtime provider. It signs requests with AWS SigV4 and sends Bedrock model IDs
directly to AWS.

Use this provider for Bedrock Runtime `Converse`, `ConverseStream`,
`InvokeModel`, and `InvokeModelWithResponseStream` access. Use
[`openai-compatible-bedrock-proxy.md`](./openai-compatible-bedrock-proxy.md)
instead when you are calling Bedrock Access Gateway or another
OpenAI-compatible proxy in front of Bedrock.

## Environment

```bash
export AWS_ACCESS_KEY_ID="AKIA..."
export AWS_SECRET_ACCESS_KEY="..."
export AWS_SESSION_TOKEN="..."        # optional
export AWS_REGION="us-east-1"         # optional, defaults to us-east-1
```

`AWS_DEFAULT_REGION` is also accepted when `AWS_REGION` is unset.

## Gateway Configuration

`ProviderConfig` still has a generic `api_key` field, but native Bedrock does
not use it. Leave it empty and put AWS-specific fields under `settings`, or let
the provider read AWS credentials from the environment.

```yaml
providers:
  - name: "bedrock-native"
    provider_type: "bedrock"
    api_key: ""
    timeout: 60
    max_retries: 3
    settings:
      aws_region: "us-east-1"
      aws_access_key_id: "${AWS_ACCESS_KEY_ID}"
      aws_secret_access_key: "${AWS_SECRET_ACCESS_KEY}"
      aws_session_token: "${AWS_SESSION_TOKEN}"
    models:
      - "us.anthropic.claude-3-5-sonnet-20241022-v2:0"
    enabled: true
```

The factory also accepts these aliases in `settings`:

- `aws_region`, `aws_region_name`, or `region`
- `aws_access_key_id`, `aws_access_key`, or `access_key`
- `aws_secret_access_key`, `aws_secret_key`, or `secret_key`
- `aws_session_token` or `session_token`

## Model IDs

The native provider preserves the AWS execution `modelId`.

```rust
completion(
    "bedrock/us.anthropic.claude-3-5-sonnet-20241022-v2:0",
    messages,
    None,
).await?;
```

The `bedrock/` prefix is only a LiteLLM-RS provider selector. The AWS request
uses `us.anthropic.claude-3-5-sonnet-20241022-v2:0`.

These IDs are also preserved for AWS execution:

- `global.anthropic.claude-sonnet-4-v1:0`
- `us-east-1.anthropic.claude-3-haiku-20240307`
- `arn:aws:bedrock:us-east-1:123456789012:inference-profile/us.anthropic...`

Metadata lookup may fall back to the canonical foundation model ID, but the
fallback ID is not sent to AWS.

## Native vs Proxy

Choose native Bedrock when you want AWS credentials, AWS model IDs, inference
profiles, and Bedrock runtime semantics.

Choose an OpenAI-compatible proxy when you have a proxy base URL and proxy API
key. Do not configure that proxy as `provider_type: "bedrock"`.

## Current Limits

Native runtime wiring and safe model ID handling are implemented. Catalog
convergence, stricter model-specific parameter policy, Mantle-specific endpoint
mode, and optional live AWS smoke tests are tracked separately from this page.
