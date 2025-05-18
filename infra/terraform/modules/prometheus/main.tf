variable "namespace" {
  type        = string
  description = "The namespace in which to deploy Prometheus"
}

resource "helm_release" "prometheus" {
  name       = "prometheus"
  repository = "https://charts.bitnami.com/bitnami"
  chart      = "kube-prometheus"
  version    = "8.25.0"
  namespace  = var.namespace
  create_namespace = false

  values = [
    file("${path.module}/values.yaml")
  ]
}