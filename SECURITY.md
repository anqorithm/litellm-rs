# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.3.x   | :white_check_mark: |
| < 0.3   | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability, please report it responsibly:

1. **Do NOT** open a public issue
2. Email security concerns to: [Create a private security advisory](https://github.com/majiayu000/litellm-rs/security/advisories/new)
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

## Response Timeline

- **Initial Response**: Within 48 hours
- **Status Update**: Within 7 days
- **Fix Timeline**: Depends on severity

## Security Best Practices

When using litellm-rs:

### API Keys

- Never commit API keys to version control
- Use environment variables for sensitive data
- Rotate keys regularly

### Configuration

- Use `.env` files for local development (add to `.gitignore`)
- Use secret managers in production (AWS Secrets Manager, HashiCorp Vault, etc.)
- Never use default passwords in production

### Network

- Enable TLS/HTTPS in production
- Use firewalls to restrict access
- Monitor for unusual traffic patterns

## Known Security Considerations

### Sensitive Data Logging

The library includes automatic sanitization of sensitive data in logs:
- API keys are masked
- Passwords are redacted
- Tokens are hidden

To enable verbose logging safely:
```bash
LITELLM_VERBOSE=true  # Logs are sanitized
```

### Dependencies

We regularly update dependencies to address security vulnerabilities. Run:
```bash
cargo audit
```

## Acknowledgments

We appreciate responsible disclosure and will acknowledge security researchers who report valid vulnerabilities.
