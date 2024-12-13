# Cw-Infuser
```

                     /"\
                    |\./|
                    |   |
                    |   |
                    |>~<|
                    |   |
                 /'\|   |/'\..
             /~\|   |   |   | \
            |   =[@]=   |   |  \
            |   |   |   |   |   \
            | ~   ~   ~   ~ |`   )
            |                   /
             \                 /
              \               /
               \    _____    /
                |--//''`\--|
                | (( +==)) |
                |--\_|_//--|

```

## Introduction
Cw-Infusion is a nft minter for burning to mint tokens in very programmable ways. Various bundle requirements, or the collection of tokens one intends to infuse in exchange for a new token.
## Additional Info 

###  Infused Collection
#### Token IDs
For each infusion, a new infused collection is created. These token-id's are incremented from 0, and kept track of the next token id in the infuser contract. 

#### Base-URI
The base uri is the folder stored to ipfs containing a list of ipfs documents. The contract sets the uri for each new token being minted based on the count: 
`token_uri: Some(infusion.infused_collection.base_uri.clone() + &token_id.to_string()),`.

This requireds the base uri to be provided with the format of `ipfs://abcd`, so 

### State.json
The deployed contracts state can be found [here](./state.json)

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
sh scripts/build.sh
```
### Deploy Infuser
The following uploads, and instantiates a new infusion to the test network:
```sh 
cargo run --bin deploy -- --network testnet 
```

### Create an Infusion 
A minimum json message to create an infusion:\
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

