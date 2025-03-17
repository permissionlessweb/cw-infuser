# Authorize Wasm Upload: Discover Decentralization DAO
Hi Stargaze Community! 

## Overview 
To upload and manage iterations of cw-infuser, this proposal requests to add the Discover Decentralization DAO to upload and migrate contracts. cw-infuser is an unaudited nft collection minter for creating infused collections. Infused collections are minted by burning the required tokens from each collection that is eligible in an infusion. 

- [Cw-Infuser Source Code](https://github.com/permissionlessweb/cw-infuser)
- [Discover Decentralization DAO](https://daodao.zone/dao/juno1aj6mmjw5jry3g5pmmhjhjtete9gxkrlmgu76j78um098lst8ppqq42ttjp/home): stars1x7krclfpvt3d50ae4cvukckz4fe46g5gx393y2cjtdpar3aw6r3q3g8pd0 

Inside the source repo there are basic integration scripts confirming the contract functions as expected. Please review these test and the source code itself, its needed to ephasise that these are not audited contracts.

## Goals 
Here is a list of some short term goals with this project. 
- An initial UI for configuring single instances of an infusion has been created and will be made open source in the near future
- A minter contract for creating infusion contracts 
- Enchances to contracts internal fee logic
- Enhances to various infused collection options
- Improvements to URI determinations (onchain & offchain)

## Proposal JSON 
```sh
[
  {
    "@type": "/cosmwasm.wasm.v1.MsgAddCodeUploadParamsAddresses",
    "authority": "stars10d07y265gmmuvt4z0w9aw880jnsr700jw7ycaz",
    "addresses": [
      "stars1x7krclfpvt3d50ae4cvukckz4fe46g5gx393y2cjtdpar3aw6r3q3g8pd0"
    ]
  }
]
```