
## Current Infusion Minters

<div align="center">

[![preview](../../public/peaceful-nightsky.png)]()\
*Art By [Jinxto](https://www.stargaze.zone/p/jinxto/collections), generated with go installed followed by running the following:\
`cd art/go/ascii-image-converter && go build && ./ascii-image-converter ../../../public/peaceful-nightsky.png -C`*
</div>


 `stargaze-1`:
- stars1333zgwvcxe04apsg98mccpc2fg7ft5xwl9ewkey2fwgf70zghsrse5nglu
- stars1zkdqlly53sdafh6dhcpuapxxc3llxyqw4v9ekk9x553mc4mv0xlqkyvg3l

## Additional Info 

###  Infused Collection
#### Token IDs
For each infusion, a new infused collection is created. These token-id's are incremented from 0, and kept track of the next token id in the infuser contract. 

#### Base-URI
The base uri is the folder stored to ipfs containing a list of ipfs documents. The contract sets the uri for each new token being minted based on the count:
This requireds the base uri to be provided with the format of `ipfs://abcd`


### State.json
The deployed contracts state can be found [here](./state.json). This file is generated from making use of cw-orchestrator scripts. 

### Creation Fees 
A minimum creation fee may be set on the contract level, requiring a fee in order to create an new infusion. This fee goes to the admin of the contract.

### Infusion Fees 
A minimum fee required to infuse may be set on the contract level, requiring all unique infusions for a contract to have at least this fee and token set as eligible fee.

### Goals & TODO:
- add documentation
- add pagination for querying infusions
- create infusion minter contract


## Scripts 
There are cw-orchestrator libraries available to deploy the infusion contracts. First, ensure your environment variables are set, such as mnemnoics state file locations, artifacts directories, transaction options, and logging. Full environment envariable details can be found here: https://orchestrator.abstract.money/contracts/env-variable.html

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



