# Project Roadmap

This document outlines the development roadmap for Leash AI.

## Version 0.1.0 - Alpha Release ✅ COMPLETE

**Goal**: Core functionality for OpenClaw on macOS

### ✅ Completed

- [x] Abstract backend architecture (extensible design)
- [x] **macOS Keychain secret backend** (primary platform)
- [x] **Homebrew package backend** (primary package manager)
- [x] Policy engine with YAML configuration
- [x] Permission client SDK (Python async)
- [x] Basic CLI tools
- [x] **OpenClaw integration guide**
- [x] Comprehensive threat model
- [x] Security design documentation
- [x] Extensibility guide
- [x] Example policies
- [x] Product Requirements Document (PRD)

### Platform Support
- ✅ **macOS 12+**: Full support
- 🔄 Linux: Planned for v0.2.0
- 🔄 Windows: Planned for v0.4.0

### Current State
- **Status**: Alpha (v0.1.0)
- **Platform**: macOS only
- **Use Case**: OpenClaw AI agent integration
- **Ready For**: Early adopters, testing, feedback

## Version 0.2.0 - Beta Release (Q2 2025)

**Goal**: Production-ready core with Linux support

### Core Features

- [ ] **Linux Secret Service Backend**
  - GNOME Keyring support
  - KWallet support
  - Secret Service D-Bus API
  
- [ ] **APT Package Manager Backend**
  - Package installation/removal
  - Dependency handling
  - Version pinning

- [ ] **Enhanced Security**
  - JWT authentication
  - TLS 1.3 support
  - API key rotation
  - Rate limiting implementation

- [ ] **Audit Improvements**
  - Log rotation
  - Remote log shipping (syslog)
  - Audit log encryption
  - Compliance reporting

- [ ] **Testing**
  - Unit test coverage >90%
  - Integration test suite
  - Performance benchmarks
  - Security penetration testing

### Documentation

- [ ] Installation guides per OS
- [ ] Video tutorials
- [ ] Architecture deep dives
- [ ] Troubleshooting guide
- [ ] FAQ

## Version 0.3.0 - Enhanced Backends (Q3 2025)

**Goal**: Enterprise-grade secret management

### Secret Backends

- [ ] **HashiCorp Vault**
  - KV v2 engine support
  - Dynamic secrets
  - Token renewal
  - Namespaces

- [ ] **AWS Secrets Manager**
  - Secret retrieval
  - Automatic rotation
  - Cross-region replication
  - IAM integration

- [ ] **Azure Key Vault**
  - Secret management
  - Key management
  - Certificate management
  - Managed identity support

- [ ] **GCP Secret Manager**
  - Secret versions
  - Automatic rotation
  - Service account auth

### Package Managers

- [ ] **DNF/YUM Backend** (RedHat, Fedora, CentOS)
- [ ] **Snap Backend** (Universal Linux packages)
- [ ] **Windows Package Managers**
  - Chocolatey
  - winget

### Approval Workflows

- [ ] **Slack Integration**
  - Interactive approval buttons
  - Channel notifications
  - Thread-based conversations
  
- [ ] **Email Workflow**
  - Email notifications
  - Approval links
  - HTML templates

- [ ] **PagerDuty Integration**
  - Incident creation
  - On-call routing
  - Escalation policies

## Version 0.4.0 - Advanced Features (Q4 2025)

**Goal**: Enterprise features and integrations

### Core Enhancements

- [ ] **Web UI for Approvals**
  - Dashboard
  - Request management
  - Policy editor
  - Audit log viewer
  - Real-time notifications

- [ ] **Policy Testing Framework**
  - Dry-run mode
  - Policy simulation
  - Coverage analysis
  - Regression testing

- [ ] **Break-glass Access**
  - Emergency override
  - Elevated logging
  - Post-incident review
  - Automated revocation

- [ ] **Secrets Rotation**
  - Automatic rotation policies
  - Grace period handling
  - Notification before expiry
  - Integration with backends

### Monitoring & Observability

- [ ] **Metrics Dashboard**
  - Request rates
  - Approval times
  - Backend health
  - Policy violations

- [ ] **Anomaly Detection**
  - Unusual access patterns
  - Time-based anomalies
  - Volume anomalies
  - ML-based detection

- [ ] **Alerting**
  - Threshold-based alerts
  - Policy violation alerts
  - Backend failure alerts
  - Integration with PagerDuty, Slack, email

### Integrations

- [ ] **CI/CD Platforms**
  - GitHub Actions plugin
  - GitLab CI plugin
  - Jenkins plugin
  - CircleCI orb

- [ ] **SIEM Integration**
  - Splunk
  - Elasticsearch
  - Datadog
  - Sumo Logic

## Version 1.0.0 - Stable Release (Q1 2026)

**Goal**: Production-ready, battle-tested, stable API

### Stability

- [ ] **API Stability Guarantee**
  - Semantic versioning
  - Deprecation policy (2 versions notice)
  - Migration guides
  - Backward compatibility

- [ ] **Performance**
  - Benchmark suite
  - Performance regression tests
  - Optimization based on profiling
  - Scalability testing (1000+ instances)

- [ ] **Reliability**
  - Circuit breakers
  - Retry logic
  - Graceful degradation
  - Health checks

### Security Hardening

- [ ] **Third-party Security Audit**
  - Code review
  - Penetration testing
  - Dependency audit
  - Published report

- [ ] **Compliance**
  - SOC 2 Type II ready
  - HIPAA compliance documentation
  - GDPR compliance
  - ISO 27001 controls mapping

- [ ] **Bug Bounty Program**
  - HackerOne/Bugcrowd setup
  - Severity levels
  - Reward structure
  - Response SLAs

### Documentation

- [ ] **Certification Program**
  - Official training
  - Best practices guide
  - Case studies
  - Community certification

- [ ] **Production Deployment Guide**
  - High availability setup
  - Disaster recovery
  - Backup and restore
  - Scaling guide

## Future (Post-1.0)

### Advanced Security

- [ ] **Zero Trust Enhancements**
  - Continuous authentication
  - Device attestation
  - Behavioral biometrics
  - Risk-based access

- [ ] **Multi-tenancy**
  - Tenant isolation
  - Per-tenant policies
  - Shared secrets with ACLs
  - Tenant-level metrics

- [ ] **Federation**
  - Cross-instance trust
  - Federated approvals
  - Policy inheritance
  - Central management

### AI/ML Features

- [ ] **Smart Policies**
  - AI-suggested policies
  - Anomaly-based policy creation
  - Usage pattern analysis
  - Automatic policy optimization

- [ ] **Predictive Security**
  - Risk scoring
  - Threat prediction
  - Proactive alerts
  - Automated response

### Platform Expansion

- [ ] **Mobile Support**
  - iOS/Android apps for approval
  - Mobile notifications
  - Biometric approval
  - Offline capability

- [ ] **Browser Extension**
  - Quick approval from browser
  - Desktop notifications
  - Credential injection
  - SSO integration

## Community Goals

### Year 1

- [ ] 100+ GitHub stars
- [ ] 10+ contributors
- [ ] 5+ backend plugins
- [ ] Active discussion forum
- [ ] Monthly community calls

### Year 2

- [ ] 500+ GitHub stars
- [ ] 50+ contributors
- [ ] 20+ backend plugins
- [ ] Conference talks
- [ ] Academic papers

### Year 3

- [ ] 1000+ production deployments
- [ ] 100+ contributors
- [ ] Ecosystem of plugins
- [ ] Industry standard adoption
- [ ] Vendor support

## How to Influence the Roadmap

We welcome community input on priorities!

1. **Vote on Issues**: Use 👍 reactions on GitHub issues
2. **Propose Features**: Open feature requests with use cases
3. **Contribute**: PRs for roadmap items are welcome
4. **Sponsor**: Sponsorship can accelerate specific features
5. **Partner**: Enterprise partnerships can fund development

## Versioning Policy

- **Patch** (0.1.x): Bug fixes only
- **Minor** (0.x.0): New features, backward compatible
- **Major** (x.0.0): Breaking changes

**Deprecation**: Features marked deprecated in version N are removed in N+2

## Release Cadence

- **Patch releases**: As needed (bug fixes, security)
- **Minor releases**: Every 6-8 weeks
- **Major releases**: When breaking changes accumulate

## Support Policy

- **Current version**: Full support
- **Previous minor**: Security fixes only
- **Older versions**: Community support only

## Feedback

Questions or suggestions about the roadmap?

- Open a [GitHub Discussion](https://github.com/openclaw/leash-ai/discussions)
- Email: roadmap@openclaw.dev
- Community calls: First Tuesday of each month

---

**Last Updated**: 2025-02-07  
**Next Review**: Q2 2025
