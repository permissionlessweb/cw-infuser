# Cw-Infuser
 
 <div align="center">

[![preview](/public/gallery.png)]()\
*Art By [Henley Bealle](https://www.henleybeall.com/), post-processed with an [ascii-image-converter](./art/generators/ascii-image-converter/README).*
</div>

## Introduction
Cw-Infusion is a nft minter for burning to mint tokens in very programmable ways. Bundles, or the collection of tokens one intends to infuse,are formed and burned in exchange for a new token from the infused collection. Eligible collections to infuse may have an fee-subsitution option, allowing to include a fungible token amount in replace of the minimum non-fungible token required.


## [Documentation](https://permissionless.money/docs/infusions)
Documentation & Guides for the infuser can be found [here](https://permissionless.money/docs/infusions).

## Deployment Verification
```sh
# current stargaze-1 code-id: 682
1d790c3e54032cbec82d461d919a4ed25e396f3e082b44c4ceb0b5e4f051c2e7  cw_infusion_minter.wasm
```

## Tests
```sh
# integration tests
cargo test --package test-suite --lib -- infuser::test::init --show-output 
# unit tests
cargo test
```


## [License](https://github.com/permissionlessweb/cw-infuser/blob/main/LICENSE)
