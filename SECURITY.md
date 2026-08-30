# Security Policy

Hunty contains Soroban smart contracts that can custody XLM and mint NFTs. Please report suspected vulnerabilities privately so they can be investigated without putting users or funds at risk.

## Reporting a Vulnerability

Please use GitHub's private vulnerability reporting channel:

[Report a vulnerability privately](https://github.com/Samuel1-ona/Hunty-contract/security/advisories/new)

Do not disclose exploitable details in a public issue, pull request, or discussion. Include enough information to reproduce the issue, including affected contract or package, relevant transaction or input data, impact, and any proposed mitigation. Please remove private keys, credentials, and other secrets from reports.

If private vulnerability reporting is unavailable, contact the repository maintainers through the GitHub organization account and request a private security contact. Do not post exploit details publicly while waiting for a response.

## Response Targets

- We aim to acknowledge a report within 5 business days.
- We aim to provide an initial severity assessment and mitigation plan within 10 business days.
- Timing may vary for complex reports or issues requiring coordination with Stellar, dependency maintainers, exchanges, or affected integrators.

## Disclosure Policy

We follow coordinated disclosure. We will work with the reporter to understand the issue, prepare a fix, and coordinate publication of a security advisory.

Unless the reporter and maintainers agree to a different schedule, we target disclosure within 90 days of the initial report. We may delay disclosure when a fix is not yet available or when disclosure could put users or funds at additional risk. We will credit reporters who request attribution and will not disclose their identity without permission.

Reports that are already public, made in bad faith, or unsupported by sufficient technical detail may not be eligible for a coordinated disclosure timeline.

## Scope

Reports involving the following are in scope:

- XLM custody, reward distribution, or unauthorized withdrawals
- NFT minting, ownership, supply, or authorization bypasses
- Contract access control, reentrancy, replay, accounting, or input-validation flaws
- Sensitive data exposure or vulnerabilities in the TypeScript service
- CI, deployment, or dependency issues that could compromise releases or user funds

Please do not report ordinary bugs, feature requests, or questions through the private vulnerability channel. Use GitHub Issues for those topics when they do not expose a security risk.

## Supported Versions

Only the latest version on the `main` branch is actively maintained for security fixes. Deployed releases may require a coordinated migration or redeployment; reports should identify the affected release, commit, or deployed contract where possible.
