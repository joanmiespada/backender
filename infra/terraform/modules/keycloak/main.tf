variable "namespace" {
  type        = string
  description = "The namespace in which to deploy Keycloak"
}

resource "helm_release" "keycloak" {
  name       = "keycloak"
  repository = "https://charts.bitnami.com/bitnami"
  chart      = "keycloak"
  version    = "20.0.8" # Latest stable from Bitnami
  namespace  = var.namespace
  create_namespace = false

  values = [
    file("${path.module}/values.yaml")
  ]
}