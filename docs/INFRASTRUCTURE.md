# Infrastructure Architecture

This document describes the infrastructure architecture, deployment processes, and management procedures for the GCP Rust Blog project.

## Architecture Overview

The project implements a **cloud-native, serverless architecture** with full Infrastructure as Code management:

```mermaid
graph TB
    subgraph "Developer Workflow"
        DEV[Developer] --> GH[GitHub Repository]
        GH --> GHA[GitHub Actions]
    end

    subgraph "Google Cloud Platform"
        subgraph "Identity & Access"
            WIP[Workload Identity Pool]
            WIPROV[WIF Provider]
            GHSA[github-actions-deploy SA]
            ADMIN[infrastructure-admin SA]
        end

        subgraph "Compute & Storage"
            AR[Artifact Registry]
            CR[Cloud Run Service]
            GCS[GCS State Bucket]
        end

        subgraph "DNS & Networking"
            DNS[Cloud DNS Zone]
            A[A Records]
            LB[Global HTTPS Load Balancer]
        end

        subgraph "APIs"
            RUNAPI[Cloud Run API]
            ARAPI[Artifact Registry API]
            DNSAPI[DNS API]
            IAMAPI[IAM API]
        end
    end

    subgraph "External"
        SQ[Squarespace Domain]
        USERS[End Users]
    end

    %% Developer workflow
    GHA -->|OIDC Auth| WIP
    WIP --> WIPROV
    WIPROV -->|Impersonate| GHSA

    %% CI/CD Pipeline
    GHSA -->|Build & Push| AR
    GHSA -->|Deploy| CR
    AR -->|Pull Image| CR

    %% Infrastructure Management
    DEV -->|OpenTofu| GCS
    ADMIN -->|Manage Policies| DNS
    ADMIN -->|Configure| RUNAPI

    %% DNS Flow
    SQ -->|Nameservers| DNS
    DNS --> A
    A -->|Points to| LB
    LB -->|Routes via serverless NEG| CR

    %% User Traffic
    USERS -->|HTTPS| LB
    LB -->|Route| CR

    %% Styling
    classDef googleCloud fill:#4285f4,stroke:#1a73e8,stroke-width:2px,color:#fff
    classDef external fill:#ea4335,stroke:#d33b2c,stroke-width:2px,color:#fff
    classDef security fill:#34a853,stroke:#137333,stroke-width:2px,color:#fff

    class AR,CR,GCS,DNS,WIP,WIPROV,RUNAPI,ARAPI,DNSAPI,IAMAPI googleCloud
    class SQ,USERS external
    class GHSA,ADMIN,WIP,WIPROV security
```

## Core Components

### 1. Google Cloud Run
- **Service**: `blog`
- **Region**: `{YOUR_GCP_REGION}`
- **URL**: `https://blog-{SERVICE_HASH}-{REGION_CODE}.a.run.app`
- **Configuration**:
  - Port: `8080`
  - CPU: 1 vCPU
  - Memory: 512 MiB
  - Concurrency: 80 requests per instance
  - Min instances: 0 (scales to zero)
  - Max instances: 100

### 2. Artifact Registry
- **Repository**: `blog`
- **Location**: `{YOUR_GCP_REGION}`
- **Format**: Docker
- **Images**: Tagged with GitHub commit SHA

### 3. Google Cloud DNS
- **Zone**: Configured via `dns_zone_name` in `infra/prod.tfvars`
- **Domain**: Configured via `domain_name` in `infra/prod.tfvars`
- **Records**:
  - `www.<domain_name>` → A record → Global Load Balancer IP
  - `<domain_name>` → A record → Global Load Balancer IP

### 4. APIs Enabled
- Cloud Run API (`run.googleapis.com`)
- Artifact Registry API (`artifactregistry.googleapis.com`)
- Cloud DNS API (`dns.googleapis.com`)
- IAM Service Account Credentials API (`iamcredentials.googleapis.com`)
- IAM API (`iam.googleapis.com`)
- Cloud Resource Manager API (`cloudresourcemanager.googleapis.com`)

## Infrastructure as Code

### OpenTofu/Terraform Configuration

```bash
infra/
├── main.tf              # Core resources and service accounts
├── variables.tf         # Input variables
├── outputs.tf          # Resource outputs
├── versions.tf         # Provider versions
├── backend.tf          # GCS backend configuration
├── providers.tf        # GCP provider setup
├── prod.tfvars         # Production variable values
└── README.md          # Infrastructure-specific docs
```

### State Management
- **Backend**: Google Cloud Storage
- **Bucket**: Configured when bootstrapping the backend
- **Path**: Controlled by the `prefix` passed to `tofu init`
- **Locking**: Enabled
- **Versioning**: Enabled with 30-day retention

### Resource Dependencies

```mermaid
graph TD
    subgraph "Foundation"
        APIS[Google APIs]
        ORG[Organization Policies]
    end

    subgraph "Identity & Access"
        SA1[github-actions-deploy SA]
        SA2[infrastructure-admin SA]
        WIP[Workload Identity Pool]
        WIPROV[WIF Provider]
        IAM[IAM Bindings]
    end

    subgraph "Infrastructure"
        AR[Artifact Registry]
        DNS[DNS Zone]
        RECORDS[DNS Records]
        CR[Cloud Run Service]
    end

    %% Dependencies
    APIS --> SA1
    APIS --> SA2
    APIS --> AR
    APIS --> DNS
    APIS --> WIP

    SA1 --> WIP
    SA2 --> ORG
    WIP --> WIPROV
    WIPROV --> IAM

    DNS --> RECORDS
    RECORDS --> CR
    AR --> CR

    classDef foundation fill:#f9ab00,stroke:#e37400,stroke-width:2px,color:#fff
    classDef identity fill:#34a853,stroke:#137333,stroke-width:2px,color:#fff
    classDef infra fill:#4285f4,stroke:#1a73e8,stroke-width:2px,color:#fff

    class APIS,ORG foundation
    class SA1,SA2,WIP,WIPROV,IAM identity
    class AR,DNS,RECORDS,CR infra
```

## Deployment Pipeline

### GitHub Actions Workflow

```mermaid
sequenceDiagram
    participant Dev as Developer
    participant GH as GitHub
    participant GHA as GitHub Actions
    participant GCP as Google Cloud
    participant CR as Cloud Run

    Dev->>GH: git push main
    GH->>GHA: Trigger workflow
    GHA->>GCP: Authenticate (OIDC + WIF)
    GHA->>GHA: Detect changed paths
    alt content/** only + app-base exists
        GHA->>GHA: Build content overlay image
    else app/base or code changed
        GHA->>GHA: Build app-base image
        GHA->>GHA: Build full runtime image
    end
    GHA->>GCP: Push image(s) to Artifact Registry
    GHA->>CR: Deploy new revision
    CR->>CR: Health checks pass
    CR->>GHA: Deployment success
```

### Build Process
1. **Change scope detection** determines whether a push is content-only (`content/**`) or includes code/infrastructure.
2. **Full build path** (code/infrastructure changes, or missing base image):
   - Build `runtime-base` image (`:app-base`) with runtime dependencies and compiled binary.
   - Build full runtime image with `/app/content`.
3. **Content-only path** (only `content/**` changed and `:app-base` exists):
   - Build overlay image from `:app-base`.
   - Copy only `content/` into `/app/content`.
4. **Image push** to Artifact Registry with commit SHA tag (`:${GITHUB_SHA}`).
5. **Cloud Run deployment** with the new commit-tagged image.
6. **Bootstrap fallback**: if `:app-base` does not exist yet, workflow automatically runs the full build path and publishes it for future content-only deploys.

## DNS Configuration

### Domain Management Flow

```mermaid
graph LR
    subgraph "Domain Registration"
        SQ[Squarespace<br/>Domain Registrar]
    end

    subgraph "DNS Management"
        NS[Nameservers<br/>ns-cloud-e*.googledomains.com]
        ZONE[Google Cloud DNS<br/>boneleve-blog zone]
    end

    subgraph "DNS Records"
        APEX[boneleve.blog<br/>A Records]
        WWW[www.boneleve.blog<br/>A Record]
    end

    subgraph "Google Cloud"
        LBIP[Global Load Balancer IP]
        LB[Global HTTPS Load Balancer]
        CR[Cloud Run Service]
    end

    SQ -->|Points to| NS
    NS --> ZONE
    ZONE --> APEX
    ZONE --> WWW
    APEX --> LBIP
    WWW --> LBIP
    LBIP --> LB
    LB --> CR

    classDef external fill:#ea4335,stroke:#d33b2c,stroke-width:2px,color:#fff
    classDef dns fill:#f9ab00,stroke:#e37400,stroke-width:2px,color:#fff
    classDef gcp fill:#4285f4,stroke:#1a73e8,stroke-width:2px,color:#fff

    class SQ external
    class NS,ZONE,APEX,WWW dns
    class LBIP,LB,CR gcp
```

### Example DNS Setup
```bash
# Nameservers (set in Squarespace)
ns-cloud-e1.googledomains.com
ns-cloud-e2.googledomains.com
ns-cloud-e3.googledomains.com
ns-cloud-e4.googledomains.com

# DNS Records (managed by OpenTofu)
www.<domain_name>.  300  A      {LOAD_BALANCER_GLOBAL_IP}
<domain_name>.      300  A      {LOAD_BALANCER_GLOBAL_IP}
```

### SSL/TLS Configuration
- **Certificate Management**: Google-managed certificates
- **Provisioning**: Automatic upon DNS validation
- **Renewal**: Automatic (90-day lifecycle)
- **Protocol**: HTTPS enforced, HTTP redirects to HTTPS

## Environment Configuration

### GitHub Repository Secrets

Configure these secrets in **GitHub Repository Settings → Secrets and Variables → Actions**:

| Secret Name | Description | Example Value |
|-------------|-------------|---------------|
| `GCP_PROJECT_ID` | Your GCP Project ID | `my-blog-project-123` |
| `GCP_PROJECT_NUMBER` | Your GCP Project Number | `123456789012` |
| `GCP_REGION` | Deployment region | `us-central1` |
| `GCP_WORKLOAD_IDENTITY_POOL` | WIF Pool ID | `github-pool` |
| `GCP_WORKLOAD_IDENTITY_PROVIDER` | WIF Provider ID | `github-provider` |
| `GCP_SERVICE_ACCOUNT` | Deploy service account email | `github-actions-deploy@my-project.iam.gserviceaccount.com` |

**How to find these values:**
```bash
# Project ID and Number
gcloud projects list

# After running OpenTofu, get service account email
cd infra
tofu output impersonated_service_account

# Workload Identity Pool and Provider names
tofu output workload_identity_pool_name
tofu output workload_identity_provider_name
```

### OpenTofu Variables (prod.tfvars)
```hcl
project_id      = "your-gcp-project-id"
project_number  = "your-project-number"
region          = "your-preferred-region"  # e.g., us-central1, europe-west1
organization_id = "your-organization-id"   # Find with: gcloud organizations list
pool_id         = "github-pool"
provider_id     = "github-provider"
github_owner    = "your-github-username"
github_repo     = "your-repo-name"
# Note: service_account_email is automatically constructed from project_id
```

## Deployment Procedures

### Initial Infrastructure Setup

1. **Configure your environment**:
```bash
# Copy template and fill in your values
cp infra/prod.tfvars.template infra/prod.tfvars
# Edit infra/prod.tfvars with your project details
```

2. **Bootstrap GCS backend** (one-time):
```bash
PROJECT_ID=your-project-id BUCKET=your-tf-state-bucket ./scripts/bootstrap-tf-state.sh
```

3. **Initialize OpenTofu**:
```bash
cd infra
tofu init -backend-config="bucket=your-tf-state-bucket" -backend-config="prefix=your-project/infra"
```

4. **Apply infrastructure**:
```bash
tofu apply -var-file="prod.tfvars"
```

### Administrative Operations

Use the dedicated admin service account for organization-level tasks:

```bash
# Organization policy management
gcloud resource-manager org-policies set-policy policy.yaml \
  --organization={ORGANIZATION_ID} \
  --impersonate-service-account=infrastructure-admin@{PROJECT_ID}.iam.gserviceaccount.com

# DNS management
gcloud dns record-sets transaction start \
  --zone=<DNS_ZONE_NAME> \
  --project={PROJECT_ID} \
  --impersonate-service-account=infrastructure-admin@{PROJECT_ID}.iam.gserviceaccount.com
```

## Security Considerations

See [SECURITY.md](SECURITY.md) for detailed security architecture and best practices.

### Key Infrastructure Security Features
- **Least Privilege IAM**: Service accounts with minimal required permissions
- **Workload Identity Federation**: Keyless authentication from GitHub Actions
- **Organization Policies**: Control over IAM policy member domains
- **Container Security**: Non-root execution, minimal base image
- **DNS Security**: Managed zone with Google Cloud DNS
- **Network Security**: HTTPS-only with Google-managed certificates

## Monitoring and Cost Optimization

### Current Costs (Estimated)
- **Cloud Run**: ~$0-5/month (scales to zero)
- **Artifact Registry**: ~$0.10/month (storage costs)
- **Cloud DNS**: ~$0.40/month (managed zone)
- **Network Egress**: Minimal for blog traffic
- **Total**: <$10/month for low-traffic blog

### Resource Monitoring
```mermaid
graph TB
    subgraph "Observability"
        LOGS[Cloud Logging]
        METRICS[Cloud Monitoring]
        TRACE[Cloud Trace]
    end

    subgraph "Sources"
        GHA[GitHub Actions]
        CR[Cloud Run]
        DNS[Cloud DNS]
        AR[Artifact Registry]
    end

    GHA --> LOGS
    CR --> LOGS
    CR --> METRICS
    CR --> TRACE
    DNS --> METRICS
    AR --> METRICS

    LOGS --> ALERTS[Cloud Alerts]
    METRICS --> ALERTS
```

This infrastructure provides a robust, secure, and cost-effective foundation for the Rust blog application with full automation and monitoring capabilities.
