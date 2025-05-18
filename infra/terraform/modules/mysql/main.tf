variable "namespace" {
  type        = string
  description = "The namespace in which to deploy MySQL"
}

resource "helm_release" "mysql" {
  name       = "mysql"
  repository = "https://charts.bitnami.com/bitnami"
  chart      = "mysql"
  version    = "9.19.0" # Stable as of 2024–2025
  namespace  = var.namespace
  create_namespace = false

  values = [
    file("${path.module}/values.yaml")
  ]
}