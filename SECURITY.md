# Security Policy

## Supported Versions

We provide security updates for the current active release line:

| Version | Supported          |
| :---    | :---               |
| 0.3.x   | :white_check_mark: |
| < 0.3.0 | :x:                |

We recommend always staying on the latest version available via Homebrew or Cargo.

---

## Reporting a Vulnerability

If you discover a security vulnerability in `run-cli`, please report it responsibly. **Do not open a public GitHub issue for security vulnerabilities.**

Instead, please report it through one of the following methods:

1. **GitHub Private Vulnerability Reporting**:
   Navigate to the [Security tab](https://github.com/naenmad/run-cli/security) on the repository and click **Report a vulnerability**.

2. **Direct Email**:
   Send an email to [2310631170064@student.unsika.ac.id](mailto:2310631170064@student.unsika.ac.id) with:
   - A description of the vulnerability.
   - Steps to reproduce or proof-of-concept code.
   - Any potential impact on users or systems.

---

## What to Expect

* **Initial Response**: We will acknowledge receipt of your report within 48 hours.
* **Assessment**: We will evaluate the report and keep you informed of our progress.
* **Resolution**: Once a fix is verified, we will release a patch release and credit the reporter (unless anonymity is requested).

---

## Scope & Security Considerations

`run-cli` interacts directly with macOS system utilities and files:

* **File Encryption**: The `run encrypt` and `run decrypt` commands use authenticated AES-256-GCM with key derivation. Flaws in cryptographic implementation or password handling are treated as critical.
* **Process Execution**: Command helpers sanitize input arguments before passing them to system subprocesses. Any bypass leading to arbitrary command injection is treated with high priority.
* **System Settings**: Commands modifying system configurations (Gatekeeper xattr, DNS flushes, network settings) require deliberate user intent and appropriate confirmations.
