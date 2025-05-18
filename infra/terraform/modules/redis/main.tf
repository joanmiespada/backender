variable "namespace" {
  type        = string
  description = "The namespace in which to deploy Redis"
}

resource "helm_release" "redis" {
  name       = "redis"
  repository = "https://charts.bitnami.com/bitnami"
  chart      = "redis"
  version    = "17.14.2" # Stable version as of 2024-2025
  namespace  = var.namespace
  create_namespace = false

  values = [
    file("${path.module}/values.yaml")
  ]
}
