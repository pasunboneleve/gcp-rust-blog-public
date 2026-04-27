output "workload_identity_pool_name" {
  value = local.wif_pool_name
}

output "workload_identity_provider_name" {
  value = local.wif_provider_name
}

output "managed_ssl_certificate_id" {
  value = google_compute_managed_ssl_certificate.blog_ssl.id
}

output "deploy_service_account_email" {
  value = google_service_account.github_actions.email
}

output "admin_service_account_email" {
  value = google_service_account.admin.email
}
