resource "azuread_application" "cedra" {
  display_name            = "cedra-${terraform.workspace}/cluster"
  prevent_duplicate_names = true
}

resource "azuread_service_principal" "cedra" {
  application_id = azuread_application.cedra.application_id
}

//  Per https://registry.terraform.io/providers/hashicorp/azuread/latest/docs/resources/application_password,
//  SP I am authenticated with  must have permissions to both Read and Write all applications and Sign in and Read user profile within the Windows Azure Active Directory API
resource "azuread_application_password" "cedra" {
  application_object_id = azuread_application.cedra.object_id
  end_date_relative     = "8760h"
}

resource "azurerm_role_assignment" "subnet" {
  principal_id         = azuread_service_principal.cedra.id
  role_definition_name = "Network Contributor"
  scope                = azurerm_subnet.nodes.id
}

resource "azurerm_user_assigned_identity" "vault" {
  name                = "cedra-${terraform.workspace}-vault"
  resource_group_name = azurerm_resource_group.cedra.name
  location            = azurerm_resource_group.cedra.location
}
