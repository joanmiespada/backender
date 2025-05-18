variable "namespace" {
  type        = string
  description = "The namespace in which to deploy Traefik"
}

resource "helm_release" "traefik" {
  name       = "traefik"
  repository = "https://helm.traefik.io/traefik"
  chart      = "traefik"
  version    = "24.0.0"
  namespace  = var.namespace
  create_namespace = false

  values = [
    file("${path.module}/values.yaml")
  ]
}