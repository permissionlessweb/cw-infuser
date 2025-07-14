use cosmwasm_std::{coin, Addr, Fraction, Storage};
use cw_infusions::{
    nfts::{InfusedCollection, RoyaltyInfoResponse},
    state::{EligibleNFTCollection, InfusionParamState, InfusionState},
};

use crate::ContractError;

// #[cfg(test)]
// mod test {
//     use crate::contract::random_token_list;
//     use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
//     use cw_orch::anyhow;

//     use super::*;

//     #[test]
//     fn test_migration() {
//         use cosmwasm_std::{Coin, Timestamp};
//         use cw_infuser_v050::state::{BundleType, RoyaltyInfoResponse as RoyaltyInfoResponseV050};
//         use cw_infusions::nfts::RoyaltyInfoResponse;

//         let mut deps = mock_dependencies();
//         let env = mock_env();

//         // Create v050 test data - State 1
//         let v050_key1 = (Addr::unchecked("collection1"), 1);
//         let v050_state1 = cw_infuser_v050::state::InfusionState {
//             payment_recipient: Addr::unchecked("recipient1"),
//             enabled: true,
//             owner: Addr::unchecked("owner1"),
//             collections: vec![cw_infuser_v050::state::NFTCollection {
//                 addr: Addr::unchecked("eligible1"),
//                 min_req: 1,
//                 max_req: Some(3),
//                 payment_substitute: Some(Coin::new(100, "ujuno")),
//             }],
//             infused_collection: cw_infuser_v050::state::InfusedCollection {
//                 sg: true,
//                 admin: Some("admin1".to_string()),
//                 name: "Name1".to_string(),
//                 description: "Description1".to_string(),
//                 symbol: "SYM1".to_string(),
//                 base_uri: "https://base.uri/1".to_string(),
//                 image: "image1.png".to_string(),
//                 num_tokens: 100,
//                 royalty_info: Some(RoyaltyInfoResponseV050 {
//                     payment_address: "royalty1".to_string(),
//                     share: cosmwasm_std::Decimal::percent(5),
//                 }),
//                 start_trading_time: Some(Timestamp::from_seconds(123456789)),
//                 explicit_content: None,
//                 external_link: Some("https://external.link/1".to_string()),
//                 addr: Some("infused1".to_string()),
//             },
//             infusion_params: cw_infuser_v050::state::InfusionParamState {
//                 bundle_type: BundleType::AnyOf {
//                     addrs: vec![Addr::unchecked("addr1"), Addr::unchecked("addr2")],
//                 },
//                 mint_fee: Some(Coin::new(500, "ujuno")),
//                 params: None,
//             },
//         };

//         // Create v050 test data - State 2 (with None values)
//         let v050_key2 = (Addr::unchecked("collection2"), 2);
//         let v050_state2 = cw_infuser_v050::state::InfusionState {
//             payment_recipient: Addr::unchecked("recipient2"),
//             enabled: false,
//             owner: Addr::unchecked("owner2"),
//             collections: vec![],
//             infused_collection: cw_infuser_v050::state::InfusedCollection {
//                 sg: false,
//                 admin: Some("admin2".to_string()),
//                 name: "Name2".to_string(),
//                 description: "".to_string(),
//                 symbol: "SYM2".to_string(),
//                 base_uri: "".to_string(),
//                 image: "".to_string(),
//                 num_tokens: 200,
//                 royalty_info: None,
//                 start_trading_time: None,
//                 explicit_content: None,
//                 external_link: None,
//                 addr: Some("infused2".to_string()),
//             },
//             infusion_params: cw_infuser_v050::state::InfusionParamState {
//                 bundle_type: BundleType::AllOf {},
//                 mint_fee: None,
//                 params: None,
//             },
//         };

//         // Save v050 data to storage
//         cw_infuser_v050::state::INFUSION
//             .save(deps.as_mut().storage, v050_key1.clone(), &v050_state1)
//             .unwrap();
//         cw_infuser_v050::state::INFUSION
//             .save(deps.as_mut().storage, v050_key2.clone(), &v050_state2)
//             .unwrap();

//         // Perform migration
//         let migrated_items = v050_patch_upgrade(deps.as_mut().storage, env.clone()).unwrap();

//         // Verify migration results
//         assert_eq!(migrated_items.len(), 2);

//         // Find migrated items by key
//         let (_, migrated_state1) = migrated_items
//             .iter()
//             .find(|(key, _)| *key == v050_key1)
//             .unwrap();
//         let (_, migrated_state2) = migrated_items
//             .iter()
//             .find(|(key, _)| *key == v050_key2)
//             .unwrap();

//         // Test state1 conversion
//         assert_eq!(
//             migrated_state1.payment_recipient,
//             Addr::unchecked("recipient1")
//         );
//         assert!(migrated_state1.enabled);
//         assert_eq!(migrated_state1.owner, Addr::unchecked("owner1"));

//         // Collections conversion
//         assert_eq!(migrated_state1.collections.len(), 1);
//         let collection = &migrated_state1.collections[0];
//         assert_eq!(collection.addr, Addr::unchecked("eligible1"));
//         assert_eq!(collection.min_req, 1);
//         assert_eq!(collection.max_req, Some(3));
//         assert_eq!(collection.payment_substitute, Some(Coin::new(100, "ujuno")));

//         // Infused collection conversion
//         assert!(migrated_state1.infused_collection.sg);
//         assert_eq!(migrated_state1.infused_collection.admin, "admin1");
//         assert_eq!(migrated_state1.infused_collection.name, "Name1");
//         assert_eq!(
//             migrated_state1.infused_collection.description,
//             "Description1"
//         );
//         assert_eq!(migrated_state1.infused_collection.symbol, "SYM1");
//         assert_eq!(
//             migrated_state1.infused_collection.base_uri,
//             "https://base.uri/1"
//         );
//         assert_eq!(migrated_state1.infused_collection.image, "image1.png");
//         assert_eq!(migrated_state1.infused_collection.num_tokens, 100);
//         assert!(migrated_state1.infused_collection.royalty_info.is_some());
//         let royalty = migrated_state1
//             .infused_collection
//             .royalty_info
//             .as_ref()
//             .unwrap();
//         assert_eq!(royalty.payment_address, "royalty1");
//         assert_eq!(royalty.share, 5); // Decimal converted to u64
//         assert_eq!(
//             migrated_state1.infused_collection.start_trading_time,
//             Some(Timestamp::from_seconds(123456789))
//         );
//         assert!(!migrated_state1.infused_collection.explicit_content);
//         assert_eq!(
//             migrated_state1.infused_collection.external_link,
//             Some("https://external.link/1".to_string())
//         );
//         assert_eq!(
//             migrated_state1.infused_collection.addr,
//             Addr::unchecked("infused1")
//         );

//         // Infusion params conversion
//         match &migrated_state1.infusion_params.bundle_type {
//             cw_infusions::bundles::BundleType::AnyOf { addrs } => {
//                 assert_eq!(addrs.len(), 2);
//                 assert_eq!(addrs[0], Addr::unchecked("addr1"));
//                 assert_eq!(addrs[1], Addr::unchecked("addr2"));
//             }
//             _ => panic!("Expected AnyOf bundle type"),
//         }
//         assert_eq!(
//             migrated_state1.infusion_params.mint_fee,
//             Some(Coin::new(500, "ujuno"))
//         );
//         assert!(!migrated_state1.infusion_params.wavs_enabled);

//         // Test state2 conversion
//         assert_eq!(
//             migrated_state2.payment_recipient,
//             Addr::unchecked("recipient2")
//         );
//         assert!(!migrated_state2.enabled);
//         assert_eq!(migrated_state2.owner, Addr::unchecked("owner2"));
//         assert!(migrated_state2.collections.is_empty());

//         // Infused collection conversion with None values
//         assert!(!migrated_state2.infused_collection.sg);
//         assert_eq!(migrated_state2.infused_collection.admin, "admin2");
//         assert_eq!(migrated_state2.infused_collection.name, "Name2");
//         assert_eq!(migrated_state2.infused_collection.description, "");
//         assert_eq!(migrated_state2.infused_collection.symbol, "SYM2");
//         assert_eq!(migrated_state2.infused_collection.base_uri, "");
//         assert_eq!(migrated_state2.infused_collection.image, "");
//         assert_eq!(migrated_state2.infused_collection.num_tokens, 200);
//         assert!(migrated_state2.infused_collection.royalty_info.is_none());
//         assert!(migrated_state2
//             .infused_collection
//             .start_trading_time
//             .is_none());
//         assert!(migrated_state2.infused_collection.explicit_content);
//         assert!(migrated_state2.infused_collection.external_link.is_none());
//         assert_eq!(
//             migrated_state2.infused_collection.addr,
//             Addr::unchecked("infused2")
//         );

//         // Infusion params conversion - AllOf
//         match migrated_state2.infusion_params.bundle_type {
//             cw_infusions::bundles::BundleType::AllOf {} => {}
//             _ => panic!("Expected AllOf bundle type"),
//         }
//         assert!(migrated_state2.infusion_params.mint_fee.is_none());
//         assert!(!migrated_state2.infusion_params.wavs_enabled);

//         // Test saving migrated data
//         save_patch_upgrade(deps.as_mut().storage, env, migrated_items).unwrap();

//         // Verify new storage
//         let new_state1 = INFUSION.load(&deps.storage, v050_key1).unwrap();
//         assert_eq!(new_state1.payment_recipient, Addr::unchecked("recipient1"));

//         let new_state2 = INFUSION.load(&deps.storage, v050_key2).unwrap();
//         assert_eq!(new_state2.payment_recipient, Addr::unchecked("recipient2"));
//     }
// }
