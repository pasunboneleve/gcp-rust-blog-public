locals {
  required_apis = [
    "run.googleapis.com",
    "artifactregistry.googleapis.com",
    "iam.googleapis.com",
    "iamcredentials.googleapis.com",
    "dns.googleapis.com",
    "cloudresourcemanager.googleapis.com",
    "compute.googleapis.com",
  ]

  deploy_roles = [
    "roles/run.admin",
    "roles/iam.serviceAccountUser",
    "roles/artifactregistry.writer",
    "roles/serviceusage.serviceUsageAdmin",
    "roles/compute.loadBalancerAdmin",
  ]

  wif_pool_name     = "projects/${var.project_number}/locations/global/workloadIdentityPools/${var.pool_id}"
  wif_provider_name = "${local.wif_pool_name}/providers/${var.provider_id}"
  github_repo_attr  = "${var.github_owner}/${var.github_repo}"
}

# API enablement is long-lived project configuration. Keep
# disable_on_destroy=false so removing this state does not disable APIs.
resource "google_project_service" "apis" {
  for_each = toset(local.required_apis)
  project  = var.project_id
  service  = each.value

  disable_dependent_services = false
  disable_on_destroy         = false
}

# WIF pools/providers are immutable here because deleted IDs enter a
# provider-side tombstone period and cannot be recreated immediately.
resource "google_iam_workload_identity_pool" "github" {
  project                   = var.project_id
  workload_identity_pool_id = var.pool_id
  display_name              = var.pool_id

  depends_on = [google_project_service.apis]
}

resource "google_iam_workload_identity_pool_provider" "github" {
  project                            = var.project_id
  workload_identity_pool_id          = google_iam_workload_identity_pool.github.workload_identity_pool_id
  workload_identity_pool_provider_id = var.provider_id
  display_name                       = var.provider_id

  attribute_mapping = {
    "google.subject"       = "assertion.sub"
    "attribute.actor"      = "assertion.actor"
    "attribute.repository" = "assertion.repository"
    "attribute.ref"        = "assertion.ref"
    "attribute.workflow"   = "assertion.workflow"
    "attribute.aud"        = "assertion.aud"
  }

  attribute_condition = "attribute.repository == '${var.github_owner}/${var.github_repo}'"

  oidc {
    issuer_uri        = "https://token.actions.githubusercontent.com"
    allowed_audiences = ["sts.googleapis.com"]
  }
}

# Production service accounts are immutable here because deleting them breaks
# deployment or operator access even though service accounts are technically
# recreatable.
resource "google_service_account" "github_actions" {
  project      = var.project_id
  account_id   = var.deploy_service_account_id
  display_name = "GitHub Actions Deploy"
  description  = "Service account for GitHub Actions deployments"

  depends_on = [google_project_service.apis]
}

resource "google_service_account" "admin" {
  project      = var.project_id
  account_id   = var.admin_service_account_id
  display_name = "Infrastructure Admin"
  description  = "Service account for administrative and organization policy tasks"

  depends_on = [google_project_service.apis]
}

resource "google_service_account_iam_binding" "wif_impersonation" {
  service_account_id = google_service_account.github_actions.id
  role               = "roles/iam.workloadIdentityUser"
  members = [
    "principalSet://iam.googleapis.com/${local.wif_pool_name}/attribute.repository/${local.github_repo_attr}"
  ]

  depends_on = [google_project_service.apis]
}

resource "google_project_iam_member" "deploy_roles" {
  for_each = toset(local.deploy_roles)
  project  = var.project_id
  role     = each.value
  member   = "serviceAccount:${google_service_account.github_actions.email}"

  depends_on = [google_project_service.apis]
}

resource "google_cloud_run_service_iam_member" "public_access" {
  location = var.region
  project  = var.project_id
  service  = var.service_name
  role     = "roles/run.invoker"
  member   = "allUsers"
}

# Public DNS is immutable because it controls the live delegated domain.
resource "google_dns_managed_zone" "boneleve_blog" {
  project     = var.project_id
  name        = var.dns_zone_name
  dns_name    = "${var.domain_name}."
  description = "DNS zone for ${var.domain_name}"

  lifecycle {
    prevent_destroy = true
  }

  depends_on = [google_project_service.apis]
}

resource "google_dns_record_set" "blog_www_a_record" {
  project      = var.project_id
  managed_zone = google_dns_managed_zone.boneleve_blog.name
  name         = "www.${var.domain_name}."
  type         = "A"
  ttl          = 300
  rrdatas      = [var.load_balancer_ip]
}

resource "google_dns_record_set" "blog_apex_a_record" {
  project      = var.project_id
  managed_zone = google_dns_managed_zone.boneleve_blog.name
  name         = "${var.domain_name}."
  type         = "A"
  ttl          = 300
  rrdatas      = [var.load_balancer_ip]
}

# The managed certificate validates real public domains, so it is not part of
# the create/destroy dress path.
resource "google_compute_managed_ssl_certificate" "blog_ssl" {
  project = var.project_id
  name    = "blog-ssl-cert"

  managed {
    domains = [
      var.domain_name,
      "www.${var.domain_name}",
    ]
  }

  depends_on = [google_project_service.apis]
}

# Organization IAM is shared control-plane state and stays outside dress.
resource "google_organization_iam_member" "admin_org_policy" {
  org_id = var.organization_id
  role   = "roles/orgpolicy.policyAdmin"
  member = "serviceAccount:${google_service_account.admin.email}"

  depends_on = [google_project_service.apis]
}

resource "google_organization_iam_member" "admin_security" {
  org_id = var.organization_id
  role   = "roles/securitycenter.adminViewer"
  member = "serviceAccount:${google_service_account.admin.email}"

  depends_on = [google_project_service.apis]
}

resource "google_organization_iam_member" "admin_service_usage" {
  org_id = var.organization_id
  role   = "roles/serviceusage.serviceUsageAdmin"
  member = "serviceAccount:${google_service_account.admin.email}"

  depends_on = [google_project_service.apis]
}

resource "google_service_account_iam_binding" "admin_impersonation" {
  service_account_id = google_service_account.admin.id
  role               = "roles/iam.serviceAccountTokenCreator"
  members = [
    "user:${var.admin_user_email}"
  ]
}

resource "google_service_account_iam_binding" "admin_user" {
  service_account_id = google_service_account.admin.id
  role               = "roles/iam.serviceAccountUser"
  members = [
    "user:${var.admin_user_email}"
  ]
}

# GitHub secrets mutate the real repository control plane.
resource "github_actions_secret" "gcp_project_id" {
  repository      = var.github_repo
  secret_name     = "GCP_PROJECT_ID"
  plaintext_value = var.project_id
}

resource "github_actions_secret" "gcp_project_number" {
  repository      = var.github_repo
  secret_name     = "GCP_PROJECT_NUMBER"
  plaintext_value = var.project_number
}

resource "github_actions_secret" "gcp_region" {
  repository      = var.github_repo
  secret_name     = "GCP_REGION"
  plaintext_value = var.region
}

resource "github_actions_secret" "gcp_service_name" {
  repository      = var.github_repo
  secret_name     = "GCP_SERVICE_NAME"
  plaintext_value = var.service_name
}

resource "github_actions_secret" "gcp_repository_id" {
  repository      = var.github_repo
  secret_name     = "GCP_REPOSITORY_ID"
  plaintext_value = var.repository_id
}

resource "github_actions_secret" "gcp_workload_identity_pool" {
  repository      = var.github_repo
  secret_name     = "GCP_WORKLOAD_IDENTITY_POOL"
  plaintext_value = var.pool_id
}

resource "github_actions_secret" "gcp_workload_identity_provider" {
  repository      = var.github_repo
  secret_name     = "GCP_WORKLOAD_IDENTITY_PROVIDER"
  plaintext_value = var.provider_id
}

resource "github_actions_secret" "gcp_service_account" {
  repository      = var.github_repo
  secret_name     = "GCP_SERVICE_ACCOUNT"
  plaintext_value = google_service_account.github_actions.email
}
