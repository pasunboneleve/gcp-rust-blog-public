# Security Architecture

This document outlines the security posture and best practices implemented in the GCP Rust Blog project.

## Service Account Architecture

The project implements a **least-privilege, multi-service-account architecture** for different operational concerns:

### 1. GitHub Actions Deployment Service Account
- **Email**: `github-actions-deploy@{PROJECT_ID}.iam.gserviceaccount.com`
- **Purpose**: Automated deployment from GitHub Actions
- **Authentication**: Workload Identity Federation (keyless)
- **Permissions**:
  - `roles/run.admin` - Deploy and manage Cloud Run services
  - `roles/iam.serviceAccountUser` - Use service accounts
  - `roles/artifactregistry.writer` - Push container images
  - `roles/serviceusage.serviceUsageAdmin` - Enable required APIs
  - `roles/compute.loadBalancerAdmin` - Manage load balancer resources

### 2. Administrative Service Account
- **Email**: `infrastructure-admin@{PROJECT_ID}.iam.gserviceaccount.com`
- **Purpose**: Organization-level administrative tasks
- **Authentication**: User impersonation with `gcloud --impersonate-service-account`
- **Organization-level permissions**:
  - `roles/orgpolicy.policyAdmin` - Manage organization policies
  - `roles/securitycenter.adminViewer` - Security monitoring
  - `roles/serviceusage.serviceUsageAdmin` - API management

### 3. Cloud Run Runtime (Default)
- **Runtime**: Uses Cloud Run's configured runtime identity; this repo does not provision a dedicated runtime service account
- **Purpose**: Application execution only
- **Permissions**: None (application doesn't need GCP API access)

## Workload Identity Federation

**Keyless authentication** between GitHub Actions and Google Cloud:

```bash
# Authentication flow
GitHub OIDC Token → Google STS → Service Account Impersonation → GCP Resources
```

### Security Benefits
✅ **No stored secrets** - No service account keys in GitHub
✅ **Short-lived tokens** - GitHub OIDC tokens expire quickly
✅ **Repository-scoped** - Only specific GitHub repo can authenticate
✅ **Attribute conditions** - Restricted to a specific repository (`owner/repo`)

### Configuration
- **Workload Identity Pool**: `projects/{PROJECT_NUMBER}/locations/global/workloadIdentityPools/github-pool`
- **Provider**: GitHub OIDC (`https://token.actions.githubusercontent.com`)
- **Audience**: `sts.googleapis.com`
- **Condition**: `attribute.repository == '{github_owner}/{github_repo}'`

## Organization-Level Access

This repository provisions org-level IAM roles for the
`infrastructure-admin` service account so an operator can manage
organization policy and related admin tasks. It does not itself define
organization policy resources.

## Container Security

### Multi-stage Docker Build
```dockerfile
# Build stage - Full Rust toolchain
FROM rust:slim as builder
# ... build application

# Runtime stage - Minimal Debian image
FROM debian:bookworm-slim
# ... only binary and runtime dependencies
```

### Runtime Security
✅ **Non-root execution** - Container runs as `appuser`
✅ **Minimal attack surface** - Only application binary and dependencies
✅ **No build tools** - Compiler and build dependencies removed

## Network Security

### Cloud Run Configuration
- **Ingress**: `all` (public blog requires external traffic)
- **Authentication**: `allUsers` (public access with organization policy controls)
- **Port**: `8080` (non-privileged port)
- **Protocol**: HTTPS only with Google-managed TLS certificates

### DNS Security
- **Managed DNS Zone**: Google Cloud DNS with OpenTofu management
- **Domain Validation**: Required before DNS management transfer
- **SSL/TLS**: Automatic certificate provisioning and renewal
- **DNS Record Strategy**: `A` records for apex and `www` both point to the global HTTPS load balancer IP

## Secrets Management

### Current Implementation
- **No application secrets** - Blog is content-only, no API keys needed
- **Build-time secrets** - None required for static blog
- **Runtime configuration** - Only environment variables (`PORT`, `RUST_LOG`)

### Future Considerations
If secrets become necessary:
- Use **Google Secret Manager** with IAM-controlled access
- **Never** bake secrets into container images
- Use **service account impersonation** for secret access
- Implement **secret rotation** policies

## Administrative Security

### Infrastructure Management
```bash
# Use dedicated admin service account for org-level tasks
gcloud resource-manager org-policies set-policy policy.yaml \
  --organization={ORGANIZATION_ID} \
  --impersonate-service-account=infrastructure-admin@{PROJECT_ID}.iam.gserviceaccount.com
```

### Access Patterns
- **Owner account**: Your designated administrative user - High-level access, minimal day-to-day use
- **Admin service account**: Organization policy and security administration
- **Deploy service account**: CI/CD and application deployment only
- **Application runtime**: No GCP API access required

## Monitoring and Alerting

### Current Logging
- **Application logs**: Structured logging with `tracing` crate
- **Cloud Run logs**: Automatic ingestion to Cloud Logging
- **Build logs**: GitHub Actions and container build logging

### Recommended Enhancements
- **Security Command Center** integration for threat detection
- **Cloud Monitoring** alerts for deployment failures
- **Audit logging** for administrative actions
- **Error reporting** for application issues

## Security Checklist

### ✅ Implemented
- [x] Least-privilege service accounts
- [x] Workload Identity Federation (keyless CI/CD)
- [x] Organization-level IAM roles for the admin service account
- [x] Non-root container execution
- [x] HTTPS-only with managed certificates
- [x] Infrastructure as Code (immutable infrastructure)
- [x] Automated test and deploy checks via GitHub Actions

### 🔄 Recommended Future Enhancements
- [ ] Container image vulnerability scanning
- [ ] Cloud Armor WAF (if needed for DDoS protection)
- [ ] Security Command Center monitoring
- [ ] Automated security policy compliance checking
- [ ] Branch protection rules in GitHub

## Incident Response

### Administrative Access Recovery
1. **Primary**: Your designated administrative user account
2. **Secondary**: Google Workspace admin access (if applicable)
3. **Escalation**: Google Cloud support with domain verification

### Service Account Key Rotation
**Note**: This project uses keyless authentication, but for emergency access:
```bash
# Generate temporary key for admin service account
gcloud iam service-accounts keys create temp-key.json \
  --iam-account=infrastructure-admin@{PROJECT_ID}.iam.gserviceaccount.com

# Revoke immediately after use
gcloud iam service-accounts keys delete KEY_ID \
  --iam-account=infrastructure-admin@{PROJECT_ID}.iam.gserviceaccount.com
```

## Compliance Considerations

- **Data Privacy**: Blog is public content, no personal data collection
- **Data Residency**: Cloud Run in `{YOUR_GCP_REGION}` region
- **Encryption**: HTTPS in transit, Google-managed encryption at rest
- **Access Logging**: Cloud Audit Logs enabled by default
- **Backup**: Source code in GitHub, infrastructure in OpenTofu state
