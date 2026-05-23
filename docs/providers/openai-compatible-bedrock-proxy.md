# OpenAI-Compatible Bedrock Proxy

Use `provider_type: "openai_compatible"` for Bedrock Access Gateway or any
other OpenAI-compatible service that proxies requests to Amazon Bedrock.

Do not use `provider_type: "bedrock"` for these deployments. Native Bedrock
expects AWS credentials and signs requests with SigV4; proxy deployments expect
a proxy base URL and proxy API key.

## Gateway Configuration

```yaml
providers:
  - name: "bedrock-access-gateway"
    provider_type: "openai_compatible"
    api_key: "${BEDROCK_ACCESS_GATEWAY_API_KEY}"
    base_url: "https://bedrock-access-gateway.example.com/api/v1"
    timeout: 60
    max_retries: 3
    settings:
      provider_name: "bedrock-access-gateway"
    models:
      - "us.anthropic.claude-3-5-sonnet-20241022-v2:0"
    enabled: true
```

## Model IDs

Model ID behavior is defined by the proxy, not by native Bedrock routing. If
the proxy expects OpenAI-style aliases, use those aliases. If it forwards AWS
model IDs, use the AWS IDs the proxy documents.

```rust
completion(
    "us.anthropic.claude-3-5-sonnet-20241022-v2:0",
    messages,
    None,
).await?;
```

## When to Use This Path

Use the proxy path when:

- You already run Bedrock Access Gateway.
- You want one OpenAI-compatible REST surface.
- Authentication is handled by the proxy API key.
- The proxy owns model aliases, request translation, and Bedrock account access.

Use [native Bedrock](./bedrock.md) when:

- The application should call AWS Bedrock Runtime directly.
- AWS SigV4 credentials and region are available to LiteLLM-RS.
- You need inference profile IDs and ARNs preserved as AWS `modelId` values.
