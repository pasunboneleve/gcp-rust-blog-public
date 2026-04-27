# APIs are enabled by infra/immutable. The testable root assumes they exist so
# dress can focus on resources that are safe to create and destroy.

resource "google_artifact_registry_repository" "blog" {
  project       = var.project_id
  location      = var.region
  repository_id = var.repository_id
  description   = "Blog container images"
  format        = "DOCKER"
}

resource "google_compute_global_address" "blog_ip" {
  project = var.project_id
  name    = var.load_balancer_ip_name
}

resource "google_compute_region_network_endpoint_group" "blog_neg" {
  project               = var.project_id
  name                  = var.network_endpoint_group_name
  network_endpoint_type = "SERVERLESS"
  region                = var.region

  cloud_run {
    service = var.service_name
  }
}

resource "google_compute_backend_service" "blog_backend" {
  project     = var.project_id
  name        = var.backend_service_name
  protocol    = "HTTP"
  port_name   = "http"
  timeout_sec = 30

  backend {
    group = google_compute_region_network_endpoint_group.blog_neg.id
  }
}

resource "google_compute_url_map" "blog_url_map" {
  project         = var.project_id
  name            = var.url_map_name
  default_service = google_compute_backend_service.blog_backend.id
}

resource "google_compute_target_https_proxy" "blog_https_proxy" {
  project          = var.project_id
  name             = var.https_proxy_name
  url_map          = google_compute_url_map.blog_url_map.id
  ssl_certificates = [var.managed_ssl_certificate_id]
}

resource "google_compute_global_forwarding_rule" "blog_https_forwarding_rule" {
  project    = var.project_id
  name       = var.https_forwarding_rule_name
  target     = google_compute_target_https_proxy.blog_https_proxy.id
  port_range = "443"
  ip_address = google_compute_global_address.blog_ip.address
}

resource "google_compute_url_map" "blog_http_redirect" {
  project = var.project_id
  name    = var.http_redirect_url_map_name

  default_url_redirect {
    https_redirect         = true
    redirect_response_code = "MOVED_PERMANENTLY_DEFAULT"
    strip_query            = false
  }
}

resource "google_compute_target_http_proxy" "blog_http_proxy" {
  project = var.project_id
  name    = var.http_proxy_name
  url_map = google_compute_url_map.blog_http_redirect.id
}

resource "google_compute_global_forwarding_rule" "blog_http_forwarding_rule" {
  project    = var.project_id
  name       = var.http_forwarding_rule_name
  target     = google_compute_target_http_proxy.blog_http_proxy.id
  port_range = "80"
  ip_address = google_compute_global_address.blog_ip.address
}
