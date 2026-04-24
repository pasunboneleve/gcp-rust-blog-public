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

## Dress rehearsals

Run isolated infrastructure rehearsals from this directory:

```bash
dress
```

`dress` injects `is_dress_rehearsal=true` and a unique `dress_run_id`
into the child Terraform/OpenTofu process. The HCL uses those values to
give disposable resources run-scoped names.

Rehearsals intentionally skip resources that are not safe to create and
destroy:

- Workload Identity pools and providers, because deleted IDs enter a
  provider-side tombstone period and cannot be recreated immediately.
- GitHub Actions secrets, because they mutate the real repository.
- Cloud DNS managed zones and records, because they control the public
  production domain.
- Organisation IAM bindings, because they mutate an organisation-level
  control plane outside a disposable test run.
- Managed SSL certificates, because they validate real public domains.

The rehearsal path still exercises run-scoped service accounts, project
IAM bindings, Artifact Registry, and load-balancer scaffolding. Cloud Run
itself is deployed by CI rather than Terraform, so the serverless NEG uses
a run-scoped service name during rehearsals but does not create an
application service.
