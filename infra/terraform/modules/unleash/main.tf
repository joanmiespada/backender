variable "namespace" {
  type        = string
  description = "The namespace in which to deploy Unleash"
}

resource "helm_release" "unleash" {
  name       = "unleash"
  repository = "https://charts.bitnami.com/bitnami"
  chart      = "unleash"
  version    = "2.0.4" # Or latest stable
  namespace  = var.namespace
  create_namespace = false

  values = [
    file("${path.module}/values.yaml")
  ]
}