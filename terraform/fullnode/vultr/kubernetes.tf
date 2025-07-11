provider "kubernetes" {
  host                   = yamldecode(base64decode(vultr_kubernetes.k8.kube_config)).clusters[0].cluster["server"]
  cluster_ca_certificate = base64decode(yamldecode(base64decode(vultr_kubernetes.k8.kube_config)).clusters[0].cluster["certificate-authority-data"])
  client_certificate     = base64decode(yamldecode(base64decode(vultr_kubernetes.k8.kube_config)).users[0].user["client-certificate-data"])
  client_key             = base64decode(yamldecode(base64decode(vultr_kubernetes.k8.kube_config)).users[0].user["client-key-data"])
}

resource "kubernetes_namespace" "cedra" {
  metadata {
    name = var.k8s_namespace
  }
}

provider "helm" {
  kubernetes {
    host                   = yamldecode(base64decode(vultr_kubernetes.k8.kube_config)).clusters[0].cluster["server"]
    cluster_ca_certificate = base64decode(yamldecode(base64decode(vultr_kubernetes.k8.kube_config)).clusters[0].cluster["certificate-authority-data"])
    client_certificate     = base64decode(yamldecode(base64decode(vultr_kubernetes.k8.kube_config)).users[0].user["client-certificate-data"])
    client_key             = base64decode(yamldecode(base64decode(vultr_kubernetes.k8.kube_config)).users[0].user["client-key-data"])
  }
}

locals {
  fullnode_helm_chart_path = "${path.module}/../../helm/fullnode"
}

resource "helm_release" "fullnode" {
  count            = var.num_fullnodes
  name             = "${terraform.workspace}${count.index}"
  chart            = local.fullnode_helm_chart_path
  max_history      = 100
  wait             = false
  namespace        = var.k8s_namespace
  create_namespace = true

  values = [
    jsonencode({
      chain = {
        era  = var.era
        name = var.chain_name
      }
      image = {
        tag = var.image_tag
      }
      nodeSelector = {
        "vke.vultr.com/node-pool" = "cedra-fullnode"
      }
      storage = {
        class = var.block_storage_class
      }
      service = {
        type = "LoadBalancer"
      }
    }),
    jsonencode(var.fullnode_helm_values),
    jsonencode(var.fullnode_helm_values_list == {} ? {} : var.fullnode_helm_values_list[count.index]),
  ]

  # inspired by https://stackoverflow.com/a/66501021 to trigger redeployment whenever any of the charts file contents change.
  set {
    name  = "chart_sha1"
    value = sha1(join("", [for f in fileset(local.fullnode_helm_chart_path, "**") : filesha1("${local.fullnode_helm_chart_path}/${f}")]))
  }
}

