terraform {
  required_providers {
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.13"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.25"
    }
  }
}

provider "kubernetes" {
  config_path    = "~/.kube/config"
  config_context = "docker-desktop"
}

provider "helm" {
  kubernetes {
    config_path    = "~/.kube/config"
    config_context = "docker-desktop"
  }
}

# Shared namespace
resource "kubernetes_namespace" "infra" {
  metadata {
    name = "infra"
  }
}

module "redis" {
  source  = "../../modules/redis"
  namespace = kubernetes_namespace.infra.metadata[0].name
}

module "mysql" {
  source  = "../../modules/mysql"
  namespace = kubernetes_namespace.infra.metadata[0].name
}

module "postgresql" {
  source    = "../../modules/postgresql"
  namespace = kubernetes_namespace.infra.metadata[0].name
}

module "keycloak" {
  source  = "../../modules/keycloak"
  namespace = kubernetes_namespace.infra.metadata[0].name
}

module "unleash" {
  source  = "../../modules/unleash"
  namespace = kubernetes_namespace.infra.metadata[0].name
}

module "elasticsearch" {
  source  = "../../modules/elasticsearch"
  namespace = kubernetes_namespace.infra.metadata[0].name
}

module "kibana" {
  source  = "../../modules/kibana"
  namespace = kubernetes_namespace.infra.metadata[0].name
}

module "prometheus" {
  source  = "../../modules/prometheus"
  namespace = kubernetes_namespace.infra.metadata[0].name
}

module "grafana" {
  source  = "../../modules/grafana"
  namespace = kubernetes_namespace.infra.metadata[0].name
}

module "traefik" {
  source  = "../../modules/traefik"
  namespace = kubernetes_namespace.infra.metadata[0].name
}
