resource "digitalocean_kubernetes_cluster" "cedra" {
  name    = "cedra-${terraform.workspace}"
  region  = var.region
  version = "1.22.8-do.1"

  node_pool {
    name       = "fullnodes"
    size       = var.machine_type
    node_count = var.num_fullnodes
    tags       = ["fullnodes"]
  }
}