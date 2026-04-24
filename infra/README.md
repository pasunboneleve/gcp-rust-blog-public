# Infrastructure (OpenTofu/Terraform)

This folder has two Terraform/OpenTofu roots:

- `immutable/` owns production resources that are not safe to create and
  destroy in rehearsals.
- `testable/` owns the production instances of resources that can be
  rehearsed with alternate names in isolated `dress` runs.

Run rehearsals through `scripts/dress-testable.sh`. The wrapper points `dress`
at `infra/testable` and gives every create/destroy resource an alternate name.
Do not use `dress --disable-isolation` for this repository.

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
exports them as `TF_VAR_*` values and renders:

- `infra/immutable/backend.auto.hcl`
- `infra/testable/backend.auto.hcl`

If you do not use direnv, source `.env` and run
`scripts/render-backend-config.sh` before `tofu init`.

## 2) Create a remote state bucket (one-time)
```bash
./scripts/bootstrap-tf-state.sh
```

## 3) Init remote states
```bash
tofu -chdir=infra/immutable init -backend-config=backend.auto.hcl
tofu -chdir=infra/testable init -backend-config=backend.auto.hcl
```

## 4) Apply production state
Import existing production resources before the first apply. Then apply each
root only after its state matches the resources it owns.

Notes:
- `domain_name` is the public domain (e.g., `boneleve.blog`).
- `dns_zone_name` is the GCP managed-zone identifier (e.g., `boneleve-blog`) and must not contain dots.
- The DNS managed zone is protected with `prevent_destroy` to avoid accidental production DNS deletion.
- Do not use `prod.tfvars` or any other tfvars file. Add or change inputs
  in `.env.template`, `.envrc`, and the root-specific `variables.tf`.

Outputs will include the WIF resource names.

## Dress rehearsals

Run isolated infrastructure rehearsals against the testable root:

```bash
./scripts/dress-testable.sh
```

The script sets `DRESS_DEPLOYMENT_ROOT=infra/testable`, exports run-scoped
`TF_VAR_*` names, then invokes `dress` in its default isolated mode. The HCL
does not contain rehearsal conditionals. A resource belongs to exactly one
root.

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
- Production service accounts and their IAM grants, because destroying them
  can break GitHub authentication, deployment, or operator access. A service
  account may live in `testable/` only when all resources depending on it are
  also safe to destroy and recreate.

The `infra/testable` remote state still owns the production instances of those
resources. The isolated `dress` run does not touch that remote state.

## State migration

This split uses two remote backend prefixes:

- `gcp-rust-blog/immutable`
- `gcp-rust-blog/testable`

Import existing production resources into the matching root. Do not apply the
new roots against empty remote state until imports have been reviewed.

`LOAD_BALANCER_IP` is required because immutable DNS records must not appear or
disappear based on an empty string. During migration, import the existing
global address into `infra/testable`, put its IP in `.env`, then import and
plan `infra/immutable`.

Initial ownership map:

- `immutable/`
  - Google project services
  - Workload Identity pool and provider
  - Deploy and admin service accounts
  - WIF impersonation binding
  - Project IAM bindings for the deploy service account
  - Admin service account impersonation bindings
  - Cloud Run public invoker binding
  - Cloud DNS zone and A records
  - Managed SSL certificate
  - Organisation IAM bindings
  - GitHub Actions secrets
- `testable/`
  - Artifact Registry repository
  - Global address, serverless NEG, backend service, URL maps, proxies, and
    forwarding rules

`testable/` resources use production names during normal remote-state applies
and alternate names from `scripts/dress-testable.sh` during isolated `dress`
runs.
