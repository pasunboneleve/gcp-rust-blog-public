output "artifact_registry_repository_id" {
  value = google_artifact_registry_repository.blog.repository_id
}

output "load_balancer_ip" {
  value = google_compute_global_address.blog_ip.address
}
