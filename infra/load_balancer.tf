# Load Balancer configuration for custom domain SSL support
# This enables boneleve.blog to work with proper SSL certificates
# since Cloud Run domain mappings are not supported in australia-southeast2

# Global IP address for the load balancer
resource "google_compute_global_address" "blog_ip" {
  project = var.project_id
  name    = "blog-ip"

  depends_on = [google_project_service.apis]
}

# SSL certificate for custom domains
resource "google_compute_managed_ssl_certificate" "blog_ssl" {
  project = var.project_id
  name    = "blog-ssl-cert"

  managed {
    domains = [
      var.domain_name,
      "www.${var.domain_name}"
    ]
  }

  depends_on = [google_project_service.apis]
}

# Network Endpoint Group pointing to Cloud Run service
resource "google_compute_region_network_endpoint_group" "blog_neg" {
  project               = var.project_id
  name                  = "blog-neg"
  network_endpoint_type = "SERVERLESS"
  region                = var.region

  cloud_run {
    service = var.service_name
  }

  depends_on = [google_project_service.apis]
}

# Backend service for the NEG
resource "google_compute_backend_service" "blog_backend" {
  project     = var.project_id
  name        = "blog-backend"
  protocol    = "HTTP"
  port_name   = "http"
  timeout_sec = 30

  backend {
    group = google_compute_region_network_endpoint_group.blog_neg.id
  }

  depends_on = [google_project_service.apis]
}

# URL map to route traffic to the backend
resource "google_compute_url_map" "blog_url_map" {
  project         = var.project_id
  name            = "blog-url-map"
  default_service = google_compute_backend_service.blog_backend.id

  depends_on = [google_project_service.apis]
}

# Target HTTPS proxy
resource "google_compute_target_https_proxy" "blog_https_proxy" {
  project          = var.project_id
  name             = "blog-https-proxy"
  url_map          = google_compute_url_map.blog_url_map.id
  ssl_certificates = [google_compute_managed_ssl_certificate.blog_ssl.id]

  depends_on = [google_project_service.apis]
}

# Global forwarding rule for HTTPS traffic
resource "google_compute_global_forwarding_rule" "blog_https_forwarding_rule" {
  project    = var.project_id
  name       = "blog-https-forwarding-rule"
  target     = google_compute_target_https_proxy.blog_https_proxy.id
  port_range = "443"
  ip_address = google_compute_global_address.blog_ip.address

  depends_on = [google_project_service.apis]
}

# HTTP to HTTPS redirect
resource "google_compute_url_map" "blog_http_redirect" {
  project = var.project_id
  name    = "blog-http-redirect"

  default_url_redirect {
    https_redirect         = true
    redirect_response_code = "MOVED_PERMANENTLY_DEFAULT"
    strip_query            = false
  }

  depends_on = [google_project_service.apis]
}

# Target HTTP proxy for redirect
resource "google_compute_target_http_proxy" "blog_http_proxy" {
  project = var.project_id
  name    = "blog-http-proxy"
  url_map = google_compute_url_map.blog_http_redirect.id

  depends_on = [google_project_service.apis]
}

# Global forwarding rule for HTTP traffic (redirect)
resource "google_compute_global_forwarding_rule" "blog_http_forwarding_rule" {
  project    = var.project_id
  name       = "blog-http-forwarding-rule"
  target     = google_compute_target_http_proxy.blog_http_proxy.id
  port_range = "80"
  ip_address = google_compute_global_address.blog_ip.address

  depends_on = [google_project_service.apis]
}
