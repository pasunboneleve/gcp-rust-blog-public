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
  description = "Workload Identity Pool ID (e.g., github-pool)"
  type        = string
}

variable "provider_id" {
  description = "Workload Identity Provider ID (e.g., github-provider)"
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
  description = "Cloud Run service name"
  type        = string
}

variable "domain_name" {
  description = "Primary domain name (without trailing dot), e.g. example.com"
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
    error_message = "dns_zone_name must use letters, numbers, and hyphens only (no dots), start with a letter, and be at most 63 chars."
  }
}

variable "organization_id" {
  description = "Google Cloud Organization ID (for org-level policies)"
  type        = string
}

variable "admin_user_email" {
  description = "Email allowed to impersonate the admin service account"
  type        = string
}

variable "cloud_run_url" {
  description = "Cloud Run service URL for CNAME record. Find this in Google Cloud Console: Cloud Run > [service-name] > copy the URL from the service details page (e.g., service-name-hash.region.run.app)"
  type        = string
}

variable "github_token" {
  description = "GitHub Personal Access Token with repo scope for managing repository secrets"
  type        = string
  sensitive   = true
}
