# Infrastructure (OpenTofu/Terraform)

This folder provisions:
- Workload Identity Pool and Provider for GitHub OIDC
- IAM binding to let your GitHub repo impersonate the deploy service account
- Project roles for the deploy service account (Cloud Run, IAM service-account use, Artifact Registry, Service Usage, Load Balancer Admin)
- Artifact Registry, Cloud DNS, load balancer resources, and GitHub Actions secrets

## Prereqs
- gcloud (authenticated to the target project)
- Terraform/OpenTofu 1.5+

## 1) Create a root environment file

```bash
cp .env.template .env
direnv allow
```

Update `.env` with the project, GitHub, domain, and organisation values.
This file is the local source of truth for Terraform inputs. `.envrc`
exports them as `TF_VAR_*` values and renders `infra/backend.auto.hcl`.
Keep `GCP_TF_STATE_PREFIX=gcp-rust-blog/infra` unless you intentionally
migrate or create a separate Terraform state path.

If you do not use direnv, source `.env` and run
`scripts/render-backend-config.sh` before `tofu init`.

## 2) Create a remote state bucket (one-time)
```bash
./scripts/bootstrap-tf-state.sh
```

## 3) Init with GCS backend
```bash
cd infra
tofu init
```

## 4) Apply
```bash
tofu apply
```

Notes:
- `domain_name` is the public domain (e.g., `boneleve.blog`).
- `dns_zone_name` is the GCP managed-zone identifier (e.g., `boneleve-blog`) and must not contain dots.
- The DNS managed zone is protected with `prevent_destroy` to avoid accidental production DNS deletion.
- Do not use `prod.tfvars` or any other tfvars file. Add or change inputs
  in `.env.template`, `.envrc`, and `infra/variables.tf`.

Outputs will include the WIF resource names.
