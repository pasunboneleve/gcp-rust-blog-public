variable "project_id" {
  description = "GCP project ID"
  type        = string
}

variable "region" {
  description = "Default region for APIs that require one"
  type        = string
}

variable "repository_id" {
  description = "Artifact Registry repository name."
  type        = string
}

variable "service_name" {
  description = "Cloud Run service name targeted by the serverless NEG."
  type        = string
}

variable "managed_ssl_certificate_id" {
  description = "Managed SSL certificate ID owned by infra/immutable."
  type        = string
}

variable "load_balancer_ip_name" {
  description = "Global address resource name."
  type        = string
}

variable "network_endpoint_group_name" {
  description = "Serverless network endpoint group resource name."
  type        = string
}

variable "backend_service_name" {
  description = "Backend service resource name."
  type        = string
}

variable "url_map_name" {
  description = "HTTPS URL map resource name."
  type        = string
}

variable "https_proxy_name" {
  description = "Target HTTPS proxy resource name."
  type        = string
}

variable "https_forwarding_rule_name" {
  description = "HTTPS global forwarding rule resource name."
  type        = string
}

variable "http_redirect_url_map_name" {
  description = "HTTP redirect URL map resource name."
  type        = string
}

variable "http_proxy_name" {
  description = "Target HTTP proxy resource name."
  type        = string
}

variable "http_forwarding_rule_name" {
  description = "HTTP global forwarding rule resource name."
  type        = string
}
