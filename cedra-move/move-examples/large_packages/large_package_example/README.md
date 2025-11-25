
# Large packages publish tutorial

If LargePackages move module is not published yet
1. Go to large-packages module(`cedra-network/cedra-move/move-examples/large_packages`) and compile it
`cedra move compile`


2. Publish to network you want to use devnet/testnet etc.
`cedra move publish --named-addresses large_packages=0x3c9124028c90111d7cfd47a28fae30612e397d115c7b78f69713fb729347a77e --assume-yes`

Next jsut compile and publish packages taht you want to chunked-publish:
`cedra move compile`
`cedra move publish --chunked-publish --large-packages-module-address 3c9124028c90111d7cfd47a28fae30612e397d115c7b78f69713fb729347a77e`

Note: If 3c9124028c90111d7cfd47a28fae30612e397d115c7b78f69713fb729347a77e address is set as default large-packages-module-address is not required.
