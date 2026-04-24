locals {
  rehearsal_suffix = substr(replace(lower(var.dress_run_id), "/[^a-z0-9-]/", "-"), 0, 12)

  deploy_service_account_id = var.is_dress_rehearsal ? "dr-${local.rehearsal_suffix}-deploy" : "github-actions-deploy"
  admin_service_account_id  = var.is_dress_rehearsal ? "dr-${local.rehearsal_suffix}-admin" : "infrastructure-admin"
  artifact_repository_id    = var.is_dress_rehearsal ? "blog-${local.rehearsal_suffix}" : var.repository_id
  cloud_run_service_name    = var.is_dress_rehearsal ? "blog-${local.rehearsal_suffix}" : var.service_name

  # Roles the deploy SA needs at the project level
  sa_roles = [
    "roles/run.admin",
    "roles/iam.serviceAccountUser",
    "roles/artifactregistry.writer",
    "roles/serviceusage.serviceUsageAdmin",
    "roles/compute.loadBalancerAdmin",
  ]

  # Required APIs for the deployment pipeline
  required_apis = [
    "run.googleapis.com",
    "artifactregistry.googleapis.com",
    "iam.googleapis.com",
    "iamcredentials.googleapis.com",
    "dns.googleapis.com",
    "cloudresourcemanager.googleapis.com",
    "compute.googleapis.com",
  ]

  # Full resource names
  wif_pool_name     = "projects/${var.project_number}/locations/global/workloadIdentityPools/${var.pool_id}"
  wif_provider_name = "${local.wif_pool_name}/providers/${var.provider_id}"

  # GitHub repository selector (owner/repo)
  github_repo_attr = "${var.github_owner}/${var.github_repo}"

  # Workload Identity resources enter a provider-side soft-delete tombstone
  # after deletion. Dress rehearsals must skip them because a create/destroy
  # cycle can block the same IDs from being recreated for production tests.
  manage_wif_resources = !var.is_dress_rehearsal

  # Public DNS and organization IAM mutate production control planes that are
  # not owned by a disposable rehearsal run.
  manage_production_only_resources = !var.is_dress_rehearsal
}

# Enable required Google Cloud APIs
resource "google_project_service" "apis" {
  for_each = toset(local.required_apis)
  project  = var.project_id
  service  = each.value

  disable_dependent_services = false
  disable_on_destroy         = false
}

resource "google_iam_workload_identity_pool" "github" {
  count = local.manage_wif_resources ? 1 : 0

  project                   = var.project_id
  workload_identity_pool_id = var.pool_id
  display_name              = var.pool_id

  depends_on = [google_project_service.apis]
}

resource "google_iam_workload_identity_pool_provider" "github" {
  count = local.manage_wif_resources ? 1 : 0

  project                            = var.project_id
  workload_identity_pool_id          = google_iam_workload_identity_pool.github[count.index].workload_identity_pool_id
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

# Create the service account for GitHub Actions
resource "google_service_account" "github_actions" {
  project      = var.project_id
  account_id   = local.deploy_service_account_id
  display_name = "GitHub Actions Deploy"
  description  = "Service account for GitHub Actions deployments"

  depends_on = [google_project_service.apis]
}

# Create administrative service account for organization-level tasks
resource "google_service_account" "admin" {
  project      = var.project_id
  account_id   = local.admin_service_account_id
  display_name = "Infrastructure Admin"
  description  = "Service account for administrative and organization policy tasks"

  depends_on = [google_project_service.apis]
}

# Create Artifact Registry repository
resource "google_artifact_registry_repository" "blog" {
  project       = var.project_id
  location      = var.region
  repository_id = local.artifact_repository_id
  description   = "Blog container images"
  format        = "DOCKER"

  depends_on = [google_project_service.apis]
}

# Create the service account for GitHub Actions
resource "google_service_account_iam_binding" "wif_impersonation" {
  count = local.manage_wif_resources ? 1 : 0

  service_account_id = google_service_account.github_actions.id
  role               = "roles/iam.workloadIdentityUser"
  members = [
    "principalSet://iam.googleapis.com/${local.wif_pool_name}/attribute.repository/${local.github_repo_attr}"
  ]
}

# Project-level roles for the deploy SA
resource "google_project_iam_member" "sa_roles" {
  for_each = toset(local.sa_roles)
  project  = var.project_id
  role     = each.value
  member   = "serviceAccount:${google_service_account.github_actions.email}"
}

# DNS Zone for boneleve.blog
resource "google_dns_managed_zone" "boneleve_blog" {
  count = local.manage_production_only_resources ? 1 : 0

  project     = var.project_id
  name        = var.dns_zone_name
  dns_name    = "${var.domain_name}."
  description = "DNS zone for ${var.domain_name}"

  lifecycle {
    prevent_destroy = true
  }

  depends_on = [google_project_service.apis]
}

# A record pointing to load balancer for www subdomain
resource "google_dns_record_set" "blog_www_a_record" {
  count = local.manage_production_only_resources ? 1 : 0

  project      = var.project_id
  managed_zone = google_dns_managed_zone.boneleve_blog[count.index].name
  name         = "www.${var.domain_name}."
  type         = "A"
  ttl          = 300
  rrdatas      = [google_compute_global_address.blog_ip.address]

  depends_on = [google_dns_managed_zone.boneleve_blog, google_compute_global_address.blog_ip]
}

# Allow public access to the blog service
resource "google_cloud_run_service_iam_member" "public_access" {
  count = local.manage_production_only_resources ? 1 : 0

  location = var.region
  project  = var.project_id
  service  = var.service_name
  role     = "roles/run.invoker"
  member   = "allUsers"

  depends_on = [google_project_service.apis]
}

# Organization-level roles for administrative service account
resource "google_organization_iam_member" "admin_org_policy" {
  count = local.manage_production_only_resources ? 1 : 0

  org_id = var.organization_id
  role   = "roles/orgpolicy.policyAdmin"
  member = "serviceAccount:${google_service_account.admin.email}"
}

resource "google_organization_iam_member" "admin_security" {
  count = local.manage_production_only_resources ? 1 : 0

  org_id = var.organization_id
  role   = "roles/securitycenter.adminViewer"
  member = "serviceAccount:${google_service_account.admin.email}"
}

resource "google_organization_iam_member" "admin_service_usage" {
  count = local.manage_production_only_resources ? 1 : 0

  org_id = var.organization_id
  role   = "roles/serviceusage.serviceUsageAdmin"
  member = "serviceAccount:${google_service_account.admin.email}"
}

# Allow owner to impersonate the admin service account
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

# A record for apex domain pointing to load balancer
resource "google_dns_record_set" "blog_apex_a_record" {
  count = local.manage_production_only_resources ? 1 : 0

  project      = var.project_id
  managed_zone = google_dns_managed_zone.boneleve_blog[count.index].name
  name         = "${var.domain_name}."
  type         = "A"
  ttl          = 300
  rrdatas      = [google_compute_global_address.blog_ip.address]

  depends_on = [google_dns_managed_zone.boneleve_blog, google_compute_global_address.blog_ip]
}
