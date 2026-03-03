# Infrastructure (OpenTofu/Terraform)

This folder provisions:
- Workload Identity Pool and Provider for GitHub OIDC
- IAM binding to let your GitHub repo impersonate the deploy service account
- Project roles for the deploy service account (Cloud Run, Artifact Registry, Cloud Build)

## Prereqs
- gcloud (authenticated to the target project)
- Terraform/OpenTofu 1.5+

## 1) Create a remote state bucket (one-time)
```bash
export PROJECT_ID=<your-project-id>
export GCS_BUCKET=<globally-unique-bucket-name>
./scripts/bootstrap-tf-state.sh
```

## 2) Init with GCS backend
```bash
cd infra
terraform init \
  -backend-config="bucket=$GCS_BUCKET" \
  -backend-config="prefix=gcp-rust-blog/infra"
```

## 3) Apply
Set values matching your environment:
```bash
terraform apply \
  -var="project_id=<PROJECT_ID>" \
  -var="project_number=<PROJECT_NUMBER>" \
  -var="organization_id=<ORGANIZATION_ID>" \
  -var="pool_id=github-pool" \
  -var="provider_id=github-provider" \
  -var="github_owner=<GITHUB_OWNER>" \
  -var="github_repo=<GITHUB_REPO>" \
  -var="service_name=blog" \
  -var="domain_name=<DOMAIN_NAME>" \
  -var="dns_zone_name=<DNS_ZONE_NAME>" \
  -var="admin_user_email=<ADMIN_USER_EMAIL>"
```

Notes:
- `domain_name` is the public domain (e.g., `boneleve.blog`).
- `dns_zone_name` is the GCP managed-zone identifier (e.g., `boneleve-blog`) and must not contain dots.
- The DNS managed zone is protected with `prevent_destroy` to avoid accidental production DNS deletion.

Outputs will include the WIF resource names.
