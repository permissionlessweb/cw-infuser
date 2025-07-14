# Cw-Infuser
 
 <div align="center">

[![preview](/public/gallery.png)]()\
*Art By [Henley Bealle](https://www.henleybeall.com/), post-processed with an [ascii-image-converter](./art/generators/ascii-image-converter/README).*
</div>

## Introduction
Cw-Infusion is a nft minter for burning to mint tokens in very programmable ways. Bundles, or the collection of tokens one intends to infuse,are formed and burned in exchange for a new token from the infused collection.

## Tests
```sh
# integration tests
cargo test --package test-suite --lib -- infuser::test::init --show-output 
# unit tests
cargo test
```

## Verification
```sh
# todo:  verify specific version checksum matches with current codehash
```

## [License](https://github.com/permissionlessweb/cw-infuser/blob/main/LICENSE)
