# Cw-Infuser

## Creating An Infustion 

## Infusing 

## Token URI's

## TODO
- add documentation
- add optional fee required for infusion 
- configure scripts 
- add pagination for querying infusions
- create infusion minter contract

## Scripts 
There are cw-orchestrator libraries available to deploy the infusion contracts. First, ensure your environment variables are set, such as mnemnoics state file locations, artifacts directories, transaction options, and logging. Full environment envariable details can be found here: https://orchestrator.abstract.money/contracts/env-variable.html

### Compile Infuser
```sh
cargo wasm 
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
 cargo run --bin create -- --col-addrs-eligible <collection-addr1,collection-addr2> --col-min-required 4,2 --infuse-col-name infusion-test --infuse-col-symbol INFUSE --infuse-col-base-uri ipfs://bafybeidyqe2abtu5eccg4uazsjnq5bstscwaxcounqxsvhtum4aalvy2hy/stars.png --config-min-per-bundle 1
```


### Infuse
To infuse:\
`--collection-ids` sets a list of collections separated by `,`, and with token-ids by `-`,
```sh
 cargo run --bin infuse -- --id 1 --collections <collection-addr1,collection-addr2> --collection-ids 69-70-71-72,79-78
```
## State.json
The deployed contracts state can be found [here](./state.json)

## Additional Info 