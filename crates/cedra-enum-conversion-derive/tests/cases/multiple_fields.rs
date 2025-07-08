// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use cedra_enum_conversion_derive::EnumConversion;

#[derive(EnumConversion)]
enum Messages {
    Test(String, String)
}

fn main() {

}
