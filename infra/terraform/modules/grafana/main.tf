variable "namespace" {
  type        = string
  description = "The namespace in which to deploy Grafana"
}

resource "helm_release" "grafana" {
  name       = "grafana"
  repository = "https://charts.bitnami.com/bitnami"
  chart      = "grafana"
  version    = "9.3.8"
  namespace  = var.namespace
  create_namespace = false

  values = [
    file("${path.module}/values.yaml")
  ]
}