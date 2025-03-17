
## Current Infusion Minters
 `stargaze-1`:
- stars1333zgwvcxe04apsg98mccpc2fg7ft5xwl9ewkey2fwgf70zghsrse5nglu
- stars1zkdqlly53sdafh6dhcpuapxxc3llxyqw4v9ekk9x553mc4mv0xlqkyvg3l


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
{"update_infusion_base_uri": {"infusion_id": 2, "base_uri":"ipfs://QmPNMu4bKFaVRkyaZssto252vpUGiowair2V27DfdEP4yq", "image":"ipfs://QmRQwG97mf8b3Cpc8YbHSLgaG55cYpiUcZD6W8saUxY2Pd"}}
```


### Update an Infusion Eligible Collections Parameters
```json
{"update_infusions_eligible_collections":{
  "id": 2,"to_add": [{"addr":"stars156x86uprzaj04v7qwnpl8djj5jws3gn73jz08qkydmkd0c0lp6gqv575pm","min_req":3,"max_req":3,payment_substitute:{"denom":"ustars",amount:"10000000000"}}],"to_remove":[]

}}
```