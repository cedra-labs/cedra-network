terraform {
  backend "azurerm" {}
}

provider "azurerm" {
  features {}
}

data "azurerm_client_config" "current" {}

resource "azurerm_resource_group" "cedra" {
  name     = "cedra-${terraform.workspace}"
  location = var.region
}
