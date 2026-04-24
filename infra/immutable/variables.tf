variable "project_id" {
  description = "GCP project ID"
  type        = string
}

variable "project_number" {
  description = "GCP project number"
  type        = string
}

variable "region" {
  description = "Default region for APIs that require one"
  type        = string
}

variable "pool_id" {
  description = "Workload Identity Pool ID"
  type        = string
}

variable "provider_id" {
  description = "Workload Identity Provider ID"
  type        = string
}

variable "github_owner" {
  description = "GitHub organization or user"
  type        = string
}

variable "github_repo" {
  description = "GitHub repository name"
  type        = string
}

variable "service_name" {
  description = "Cloud Run service name used by the deploy workflow"
  type        = string
}

variable "repository_id" {
  description = "Artifact Registry repository name used by the deploy workflow"
  type        = string
}

variable "deploy_service_account_id" {
  description = "Production deploy service account ID. Immutable because destroying it interrupts GitHub Actions deployment."
  type        = string
  default     = "github-actions-deploy"
}

variable "admin_service_account_id" {
  description = "Production administrative service account ID. Immutable because org IAM and operator access depend on it."
  type        = string
  default     = "infrastructure-admin"
}

variable "domain_name" {
  description = "Primary domain name without trailing dot"
  type        = string

  validation {
    condition     = can(regex("^[a-z0-9]([a-z0-9-]*[a-z0-9])?(\\.[a-z0-9]([a-z0-9-]*[a-z0-9])?)+$", var.domain_name))
    error_message = "domain_name must be a valid domain without trailing dot, e.g. boneleve.blog."
  }
}

variable "dns_zone_name" {
  description = "Cloud DNS managed zone resource name"
  type        = string

  validation {
    condition     = can(regex("^[a-z]([-a-z0-9]*[a-z0-9])?$", var.dns_zone_name)) && length(var.dns_zone_name) <= 63
    error_message = "dns_zone_name must use letters, numbers, and hyphens only, start with a letter, and be at most 63 chars."
  }
}

variable "organization_id" {
  description = "Google Cloud Organization ID for org-level IAM"
  type        = string
}

variable "load_balancer_ip" {
  description = "Production global load balancer IP from infra/testable."
  type        = string
}

variable "admin_user_email" {
  description = "Email allowed to impersonate the admin service account"
  type        = string
}

variable "github_token" {
  description = "GitHub Personal Access Token with repo scope for managing repository secrets"
  type        = string
  sensitive   = true
  default     = null
  nullable    = true
}
