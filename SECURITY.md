# Security Policy

## Supported Versions

Only the latest released version is actively supported with security updates.

## Reporting a Vulnerability

If you discover a security vulnerability, please report it privately.

**Do NOT** open a public issue.

### How to Report

Send an email to: security@griid-trace.xyz

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if known)

### Response Timeline

- Initial response: Within 48 hours
- Fix timeline: Based on severity
- Public disclosure: After fix is released

## Security Principles

### Local-First Security

griid-trace is designed with security by default:

1. **No Outbound Network**: Zero network calls by default
2. **No Data Collection**: Zero telemetry, no phone-home
3. **File-Based**: All data stays on your machine
4. **Auditable**: Open source Rust code, fully inspectable
5. **Reproducible Builds**: Binary verification available

### Threat Model

We protect against:
- Remote code execution via trace files
- Data exfiltration (no network by default)
- Supply chain attacks (minimal dependencies)
- Privilege escalation (runs as user, no sudo)

### File Handling

- trace.jsonl files are parsed with strict validation
- No arbitrary code execution from trace files
- File size limits enforced
- Path traversal protection

## Dependency Security

### Minimal Dependencies

We use minimal, well-audited dependencies:
- Rust std library
- serde/serde_json (parsing)
- ratatui/crossterm (TUI)
- tokio (async runtime)

### Dependency Updates

- Regular security audits
- Automated dependency scanning
- Prompt updates for CVEs

## Binary Verification

### Checksums

All releases include SHA256 checksums for verification.

### Signatures

Desktop binaries are signed for platform verification.

### Verification Steps

```bash
# Download checksums
curl -O https://github.com/griid-trace/griid-trace/releases/download/v1.0.0/SHA256SUMS

# Verify
sha256sum -c SHA256SUMS
```

## Past Audits

See [AUDIT.md](AUDIT.md) for the full security audit log, automated CI checks, and verification procedures.

| Date | Type | Result |
|------|------|--------|
| 2026-07-05 | Automated CI (cargo-audit, clippy) | Ongoing on every PR |
| 2026-07-05 | Internal design review | Pass — local-first architecture |
| TBD | Independent third-party audit | Planned |

## Best Practices for Users

1. **Verify Downloads**: Always verify checksums
2. **Keep Updated**: Use latest version
3. **Review Code**: It's open source, review it
4. **Run as User**: No root/admin required
5. **Air-Gapped Safe**: Works without internet

## Disclosure Policy

- Coordinated disclosure for vulnerabilities
- Credit to researchers
- Transparent communication
- No bounty program (community-driven)

## Contact

- Security: security@griid-trace.xyz
- General: hello@griid-trace.xyz
- GitHub: https://github.com/griid-trace/griid-trace
