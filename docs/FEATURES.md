# OpenLeash - Canonical Feature Tracker

This document tracks the implementation progress of all features defined in the PRD and the overall project roadmap.

## Status Legend
- **Complete**: Fully implemented and tested.
- **In Progress**: Work started, basic functionality may exist.
- **Pending**: Not yet started.
- **Blocked**: Waiting on another feature.

---

## 1. Core Infrastructure (P0)
| Feature | Status | Description |
| :--- | :---: | :--- |
| Cargo Workspace Scaffolding | Complete | Root workspace with 12+ modular crates. |
| gRPC API (Protobuf) | Complete | Defined Request, Task, Approval, and Audit services. |
| SQLite Persistence Layer | Complete | Async storage using SQLx with automatic migrations. |
| Unix Domain Socket (UDS) Support | Complete | Secure local IPC for agent-to-daemon communication. |
| YAML Configuration Schema | Complete | Unified configuration for server, storage, and backends. |
| Onboarding Wizard (`openopenleash init`) | Complete | Automatic environment scaffolding and profile generation. |

## 2. Resource Backends (P0)
| Feature | Status | Description |
| :--- | :---: | :--- |
| **Pip Backend** | Complete | Scoped Python package installation into task venvs. |
| **NPM Backend** | Complete | Scoped Node.js package installation via `NPM_CONFIG_PREFIX`. |
| **Brew Backend** | Complete | Portable Homebrew instances cloned into task scopes. |
| **Keychain Backend** | Complete | Brokered access to macOS Keychain secrets. |
| **Resource Broker Model** | Complete | Daemon brokers resources (packages, secrets, PATH); agents execute commands directly in sandbox. |

## 3. Sandboxing & Isolation (P0)
| Feature | Status | Description |
| :--- | :---: | :--- |
| macOS Seatbelt Profiles | Complete | Tiered security profiles (Permissive/Restrictive). |
| Feature-Aware Sandbox Generation | Complete | Profiles dynamically adapt to enabled backends. |
| Task Scope Management | Complete | Automatic creation and cleanup of task-specific venvs. |
| Task Environment API | Complete | `GetTaskEnvironment` provides PATH/bin directory for direct agent execution. |

## 4. Policy Engine (P0)
| Feature | Status | Description |
| :--- | :---: | :--- |
| Regex Pattern Matching | Complete | Match resource IDs against policy patterns. |
| Priority-Based Evaluation | Complete | Conflict resolution via policy priority weights. |
| Deny-by-Default Logic | Complete | Fail-secure evaluation if no policies match. |
| Time-to-Live (TTL) Enforcement | Complete | Mandatory expiry for tasks and leases. |
| **Approval Scopes** | In Progress | Support for Once, Task, and Permanent approvals. |

## 5. Human-in-the-Loop (P0)
| Feature | Status | Description |
| :--- | :---: | :--- |
| **CLI Approval Management** | Complete | `openopenleash approve {list, grant, deny}` commands. |
| **Telegram Integration** | Complete | Real-time mobile notifications and approval buttons. |
| Slack Integration | Pending | Pending. |
| Web UI Dashboard | Pending | Future Milestone. |

## 6. Audit & Compliance (P0)
| Feature | Status | Description |
| :--- | :---: | :--- |
| **Hash-Chained Audit Ledger** | Complete | Immutable SHA-256 ledger of all daemon actions. |
| Audit Query CLI (`openopenleash audit`) | Complete | Visibility into the action history. |
| Integrity Verification | Complete | Locally verify the structural integrity of the chain. |
| Export (JSON/CSV) | In Progress | JSON output implemented for `openopenleash audit list`. |

## 7. Lifecycle & Automation (P1)
| Feature | Status | Description |
| :--- | :---: | :--- |
| Automatic Task Reaping | Complete | Background worker to clean up expired environments. |
| Standalone Lease Reaping | Complete | Background worker to uninstall temporary packages. |
| Service Installation (`openleash install`) | In Progress | Generates LaunchAgent/Systemd plists. |
| Health Checks | Pending | Pending. |

---

## 8. Improvements (Documentation & Quality)

Tracked items from project health and open-source readiness reviews.

| Improvement | Status | Description |
| :--- | :---: | :--- |
| **ARCHITECTURE.md roadmap** | Complete | Updated to reflect resource broker model (GetTaskEnvironment instead of ExecuteCommand). |
| **SECURITY.md contact** | Pending | Replace "add email here" placeholder with a real contact or GitHub Private Vulnerability Reporting only. |
| **Test scope & robustness** | Complete | Tests updated for GetTaskEnvironment API. ExecuteCommand tests removed. |
| **OPENCLAW_INTEGRATION.md** | Complete | Updated to reflect new execution model (direct execution with PATH). |
| **Default policy documentation** | Pending | Document where policies are loaded from (e.g. `LEASHD_POLICIES_PATH`, init) and behavior when no policy file or empty policies (deny-all vs allow-all). |
| **Linux secret backend** | Pending | Add SecretBackend for Linux Secret Service or Vault (ARCHITECTURE roadmap item). |
| **NPM isolation note** | Pending | Document in SECURITY_DESIGN or ARCHITECTURE that NPM uses system `npm` with scoped prefix; isolation is weaker than pip (venv) and brew (portable). |

---

## Next Priority Tasks:
1.  **OpenClaw Integration Examples**: Create `examples/` for direct SDK usage.
2.  **Full Approval Scope Logic**: Ensure `Task` and `Permanent` scopes are respected across all backends.
3.  **Audit Ledger Export**: Finalize CSV/JSON export for compliance reporting.
