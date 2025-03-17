

## Requirements
- Infusion minter address 
- Infusion ID
- Bundle of NFTs to infuse
- Approval granted to infusion minter for all nfts in bundle

## Workflow
When a bundle is being infused, the contract loads the infusion based on the infusion id provided.
the contract validates any static mint fee required to infuse a bundle,and splits this fee between the infusion payment recipient and the infusion global admin, if set. 

*An infusion owner is bypassed from this fee validation if infusing.*

Next, all bundles `[b]` are validated by ensuring each nft is owned by the address infusing the bundle. Along with ownership validation, the contract determines how many infuseed nfts to mint, and which nfts in bundles are to burn or omit from burning.  This decision is made based on the parameters of the specific infusion, and what has been sent in the bundle.

## 1. Creating Bundles To Burn  
## 2. Payment Substitute 
Each eligible collection can have an optional payment substitute, where a specific token amount will satisfy the requirement of that collection, as if the minimum number of nfts were included in the bundle. This is true for both bundle types `AllOf` and `AnyOf`, but for `AnyOfBlend`, each blend can disable or enable the payment substitute. If it is enabled, the 'cost' per token for the blend options is determined by the ratio between the minimim required & the amount of the paymment substitute an eligible collection has set. 

0. for each of an infusions eligible collections  `e`:
1. iterate through bundle and count how many nfts are from collection `e`.
- if `e` is eligible for a payment subsitute, we check the bundle type:
    - if `BundleType::AllOf`: fee substitute must always be sent, for all eligible collections.
    - if `BundleType::AnyOf`:  atleast one of the eligible collections must have fee subsitute, or minimum in bundle.
        -  Track list of eligible anyOf collection that used payment substitute: `anyofmap`.

- if no payment substitute for `e` exists, we ensure for `AllOf` bundles that there are eligible nfts from `e` in bundle. `AnyOf` & `AnyOfBlend`  bundle types needs to go through all of the eligible collections to determine the amount of infsed nfts or errors to return, in contrast to `AllOf` where if a bundle does not satisfy an  eligible collection parameters, the contract immedieately rejects the infusion msg

- if type 3: atleast one of either the eligible collections, or one of the bundleBlends have been satisfied
