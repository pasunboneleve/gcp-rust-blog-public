# GitHub Repository Secrets
# These secrets will be automatically configured for GitHub Actions
#
# GitHub secrets mutate the real repository control plane and require a real
# token. Dress rehearsals skip them so disposable runs cannot overwrite
# production deployment settings.

resource "github_actions_secret" "gcp_project_id" {
  count = local.manage_production_only_resources ? 1 : 0

  repository      = var.github_repo
  secret_name     = "GCP_PROJECT_ID"
  plaintext_value = var.project_id
}

resource "github_actions_secret" "gcp_project_number" {
  count = local.manage_production_only_resources ? 1 : 0

  repository      = var.github_repo
  secret_name     = "GCP_PROJECT_NUMBER"
  plaintext_value = var.project_number
}

resource "github_actions_secret" "gcp_region" {
  count = local.manage_production_only_resources ? 1 : 0

  repository      = var.github_repo
  secret_name     = "GCP_REGION"
  plaintext_value = var.region
}

resource "github_actions_secret" "gcp_service_name" {
  count = local.manage_production_only_resources ? 1 : 0

  repository      = var.github_repo
  secret_name     = "GCP_SERVICE_NAME"
  plaintext_value = var.service_name
}

resource "github_actions_secret" "gcp_repository_id" {
  count = local.manage_production_only_resources ? 1 : 0

  repository      = var.github_repo
  secret_name     = "GCP_REPOSITORY_ID"
  plaintext_value = var.repository_id
}

resource "github_actions_secret" "gcp_workload_identity_pool" {
  count = local.manage_production_only_resources ? 1 : 0

  repository      = var.github_repo
  secret_name     = "GCP_WORKLOAD_IDENTITY_POOL"
  plaintext_value = var.pool_id
}

resource "github_actions_secret" "gcp_workload_identity_provider" {
  count = local.manage_production_only_resources ? 1 : 0

  repository      = var.github_repo
  secret_name     = "GCP_WORKLOAD_IDENTITY_PROVIDER"
  plaintext_value = var.provider_id
}

resource "github_actions_secret" "gcp_service_account" {
  count = local.manage_production_only_resources ? 1 : 0

  repository      = var.github_repo
  secret_name     = "GCP_SERVICE_ACCOUNT"
  plaintext_value = google_service_account.github_actions.email
}
