# Security

This project will always remain local first unless you choose to outsource your llms to external API's. Deterministic internal tools (calculator, system time) run locally; probabilistic reasoning uses the Maker voting tool described in the MAD/Maker docs. With that being said please always let me know of any insecurities and I'll be happy to resolve them!

Current hardening baseline in the daemon:
- Ingress scan + tainting for prompt-injection indicators before orchestration.
- Safety sidecar risk engine (`green/amber/red`) in front of tool execution.
- Raw command execution blocked by policy; only typed tools are routed.
- API keys stored in OS keychain via `keyring` and excluded from persisted JSON config.

<img src="docs/assets/boomaiSand.png" alt="Boomai" width="80%">

If you find a vulnerability, please reach out:
- Email: security@boomai.dev
- Or open a private GitHub Security Advisory (preferred for sensitive issues).

Include:
- What’s the issue and impact?
- Steps to reproduce and affected configs?
- Any mitigation ideas?

I’ll acknowledge within 3 business days and share a plan or status update within 10 business days. Please keep it private until there’s a fix.
