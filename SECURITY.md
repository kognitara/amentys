# Security Policy

## Scope

This policy covers the Amentys source tree and official releases maintained in this repository.
Third-party dependencies remain subject to their own security processes, but dependency issues
that affect Amentys should still be reported here.

## Reporting a vulnerability

Please do not disclose an unpatched vulnerability in a public issue, pull request, chat, or
social-media post.

Report security issues privately through the repository's private vulnerability reporting
feature, when enabled. If that feature is unavailable, contact the project maintainer through
the private contact method listed in the repository profile and include `Amentys security`
in the subject.

Please include:

- the affected version, commit, or component;
- steps to reproduce or a proof of concept;
- expected and observed behavior;
- security impact and possible mitigations.

Remove passwords, private keys, personal data, and other secrets from the report.

## Response process

The maintainer will acknowledge a valid private report when reasonably possible, investigate
its impact, coordinate a fix, and publish a security advisory or release note when disclosure
is appropriate. Reporters will be credited when they give permission.

Security fixes must not be used to introduce telemetry, data exfiltration, or closed binary
logic into the Amentys core. See LICENSE and the project's contribution rules.

## Supported versions

Unless a release note states otherwise, security fixes target the latest official release and
the current development branch. Unsupported or modified distributions are not official Amentys
releases.
