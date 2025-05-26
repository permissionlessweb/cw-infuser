
## Current Infusion Minters
 `stargaze-1`:
- stars1333zgwvcxe04apsg98mccpc2fg7ft5xwl9ewkey2fwgf70zghsrse5nglu
- stars1zkdqlly53sdafh6dhcpuapxxc3llxyqw4v9ekk9x553mc4mv0xlqkyvg3l


## TODO: 
- add pagination for querying infusions
- support for depositing nfts (used to ommit nfts approval tx requirement)
- immuatbility support (cannot update baseuri, ...)
- add bundle recipies: configure what types of bundles can be made:
  <!-- - allOf: requires the minimum for all eligible collections -->
  <!-- - anyOf: any of 1 of the eligible collections -->
  - anyOfBlend: select which elgible collections may be set to have blended requirements.
- instantiate fee-split upon infusion creation for burnt collecction royalty recipients


### Compile Infuser
```sh
just wasm-optimize
```

### Deploy Infuser
The following uploads, and instantiates a new infusion to the test network:
```sh 
cargo run --bin deploy -- --network testnet 
```

### Create an Infusion 
A minimum json message to create an infusion:
`--col-min-require` in the same order as collections defined, set the minimum tokens required for each to need to infuse.
```sh
 cargo run --bin create -- --col-addrs-eligible <collection-addr1,collection-addr2> --col-min-required 4,2 --infuse-col-name infusion-test --infuse-col-symbol INFUSE --infuse-col-base-uri ipfs:// --config-min-per-bundle 1
```
<!-- cargo run --bin create -- --col-addrs-eligible stars18vng693zqjgwd08p3ypzy26h8f7d7yjweahn5hxq2xnuu837emuslfzn5w,stars1pxcrcl2kt30qdjny8ek6fpkffye4xstvypqdgmh5ssr4yrfu8sgs7450ql --col-min-required 4,2 --infuse-col-name infusion-test --infuse-col-symbol INFUSE --infuse-col-base-uri ipfs://bafybeidyqe2abtu5eccg4uazsjnq5bstscwaxcounqxsvhtum4aalvy2hy/stars.png --config-min-per-bundle 1 -->

### Infuse
To infuse:\
`--collection-ids` sets a list of collections separated by `,`, and with token-ids by `-`,
```sh
 cargo run --bin infuse -- --id 1 --collections <collection-addr1,collection-addr2> --collection-ids 69-70-71-72,79-78
```
<!-- cargo run --bin infuse -- --id 1 --collections stars18vng693zqjgwd08p3ypzy26h8f7d7yjweahn5hxq2xnuu837emuslfzn5w,stars1pxcrcl2kt30qdjny8ek6fpkffye4xstvypqdgmh5ssr4yrfu8sgs7450ql --collection-ids 91-90-89-88,86-58 -->



