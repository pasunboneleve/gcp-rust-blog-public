# Infrastructure (OpenTofu/Terraform)

This folder has two Terraform/OpenTofu roots:

- `immutable/` owns production resources that are not safe to create and
  destroy in rehearsals.
- `testable/` owns the production instances of resources that can be
  rehearsed with alternate names in isolated `dress` runs.

`dress` must run against `infra/testable` only, in its default isolated mode.
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
```bash
tofu -chdir=infra/testable apply
tofu -chdir=infra/immutable apply
```

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
dress
```

`.envrc` sets `DRESS_DEPLOYMENT_ROOT=infra/testable`. In default isolated
mode, `dress` copies that root, forces local state, injects
`is_dress_rehearsal=true` and `dress_run_id`, applies, collects outputs, and
destroys only the alternate-named rehearsal resources.

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

The `infra/testable` remote state still owns the production instances of those
resources. The isolated `dress` run does not touch that remote state.

## State migration

This split uses two remote backend prefixes:

- `gcp-rust-blog/immutable`
- `gcp-rust-blog/testable`

Import existing production resources into the matching root. Do not apply the
new roots against empty remote state until imports have been reviewed.
