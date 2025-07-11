
## Current Infusion Minters
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
- support for depositing nfts (used to ommit nfts approval tx requirement)
- immuatbility support (cannot update baseuri, ...)
- add bundle recipies: configure what types of bundles can be made:
  - allOf: requires the minimum for all eligible collections
  - anyOf: any of 1 of the eligible collections
  - anyOfBlend: select which elgible collections may be set to have blended requirements.
- instantiate fee-split upon infusion creation for 

## Instantiate
```json
{
    "admin": "stars1x7krclfpvt3d50ae4cvukckz4fe46g5gx393y2cjtdpar3aw6r3q3g8pd0",
    "admin_fee": 2,
    "min_creation_fee": {
        "denom": "ibc/4A1C18CA7F50544760CF306189B810CE4C1CB156C7FC870143D401FE7280E591",
        "amount": "50000000"
    },
    "max_infusions": 2,
    "min_per_bundle": 1,
    "max_per_bundle": 10,
    "max_bundles": 5,
    "cw721_code_id": 15
}
```

### Create  An Infusion
```json
{
  "create_infusion":{
"collections":[{
  "collections":[{"addr":"stars1u08g6aqwnwn3248jujhhjft4fqe6x4hvljm6c9glf6sj9tc8r6jshteaqm","min_req":2},
  {"addr":"stars1t4rjvp298atd47yp4xefmgku4vc5ky95dk342q7qndr6ueuxhh6qq3yte6","min_req":1},
  {"addr":"stars1a0a2lkw7hydkav58px9xdfu3t34wsh6wudwnx69zzp5693nkf7wq96fx97","min_req":2},
  ],
  "infused_collection": {"sg":true,"name":"infusions2","symbol":"INFUSE2","base_uri":"ipfs://xyz","num_tokens": 7000},
"infusion_params": {
  "mint_fee":  {"denom":"ustars","amount": "10"}
}
}
]}}
```

###  Update An Infusion BaseURI
```json
{"update_infusion_base_uri": {"infusion_id": 2, "base_uri":"ipfs://QmXWasD3MnpSUNxva3aARnTxVb3hHcb59yMBQ4VRWKw1oB"}}
```

###  Update An Infusion Bundle Type
```json
{
  "update_infusion_bundle_type": {
    "id": 2,
    "bundle_type": {
      "any_of": {
        "addrs": [
          "stars156x86uprzaj04v7qwnpl8djj5jws3gn73jz08qkydmkd0c0lp6gqv575pm",
          "stars1ha2lthlyxleqszwah869hhg7wvzjtxz59w6tk4s2jkmk62shk3vskftn66",
          "stars1xy930u7nzynzzeld2erved4rtdkzrleqt9jr2fvkxn3d6ct4s5xs3lynaj",
          "stars1swkzrx40jj2q5q3nh45pp60lqvpm6gwjcumpg5e72h2y8dyjh87sxh3ven"
        ]
      }
    }
  }
}
```


### Update Infusion Mint Fee
```json
{
  "update_infusion_mint_fee": {
    "id": 2,
    "mint_fee": {"denom":"ustars","amount":"100000000"}
  }
}
```
### Update an Infusion Eligible Collections Parameters
```json
{"update_infusions_eligible_collections":{"id": 2,"to_add": [{"addr":"stars156x86uprzaj04v7qwnpl8djj5jws3gn73jz08qkydmkd0c0lp6gqv575pm","min_req":3,"max_req":3,"payment_substitute":{"denom":"ustars","amount":"10000000000"}}],"to_remove":[]}}
```