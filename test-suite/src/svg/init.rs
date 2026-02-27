use cosmwasm_std::{coin, coins, Addr, Binary, Timestamp};
use cw721_svg::interface::Cw721Svg;
use cw721_svg::msg::*;
use cw721_svg::state::{MAX_SVG_SIZE, MAX_TOTAL_SUPPLY};
use cw_orch::{anyhow, mock::MockBech32, prelude::*};
use rs_merkle::MerkleTree;
use whitelist_mtree::interface::WhitelistMerkleTree;
use whitelist_mtree::tests::{hasher::SortingBlake3Hasher, test_helpers::hash_and_build_tree};

pub struct CwSvgSuite<Chain> {
    pub chain: MockBech32,
    pub svg: Cw721Svg<Chain>,
    pub wlist: WhitelistMerkleTree<Chain>,
    pub admin: Addr,
}

const TEST_SVG_TEMPLATE: &str =
    "<svg><circle fill='${color_yin}' /><path fill='${color_yang}' /></svg>";

fn test_variables() -> Vec<VariableDef> {
    vec![
        VariableDef {
            name: "color_yin".to_string(),
            kind: VariableKind::Options(vec![
                "rgb(64,38,14)".to_string(),
                "rgb(34,34,37)".to_string(),
                "rgb(177,182,180)".to_string(),
                "rgb(0,0,0)".to_string(),
            ]),
        },
        VariableDef {
            name: "color_yang".to_string(),
            kind: VariableKind::Options(vec![
                "rgb(141,93,51)".to_string(),
                "rgb(56,56,64)".to_string(),
                "rgb(208,229,226)".to_string(),
                "rgb(221,221,222)".to_string(),
            ]),
        },
    ]
}

fn options_list(var: &VariableDef) -> &Vec<String> {
    match &var.kind {
        VariableKind::Options(opts) => opts,
        _ => panic!("expected Options variant"),
    }
}

fn test_seed() -> Binary {
    Binary::from(b"test-seed-entropy-value-1234567")
}

/// Compute template slots from a template and variables (simulates off-chain tooling).
fn compute_slots(template: &str, variables: &[VariableDef]) -> Vec<TemplateSlot> {
    let bytes = template.as_bytes();
    let len = bytes.len();
    let mut slots = Vec::new();
    let mut i = 0;
    while i < len.saturating_sub(1) {
        if bytes[i] == b'$' && bytes[i + 1] == b'{' {
            let start = i;
            let name_start = i + 2;
            let mut j = name_start;
            while j < len && bytes[j] != b'}' {
                j += 1;
            }
            if j >= len {
                panic!("unclosed placeholder in test template");
            }
            let name = &template[name_start..j];
            let end = j + 1;
            let var_idx = variables
                .iter()
                .position(|v| v.name == name)
                .unwrap_or_else(|| panic!("unknown variable '{}' in test template", name));
            slots.push(TemplateSlot {
                start: start as u32,
                end: end as u32,
                var_idx: var_idx as u16,
            });
            i = end;
        } else {
            i += 1;
        }
    }
    slots
}

/// Helper to build a standard mint message (no whitelist proof).
fn mint_msg(amount: u64) -> ExecuteMsg {
    ExecuteMsg::Mint {
        amount,
        proof_hashes: None,
        allocation: None,
    }
}

impl CwSvgSuite<MockBech32> {
    fn setup() -> anyhow::Result<Self> {
        let mock = MockBech32::new("mock");
        let admin = mock.addr_make("admin");

        let svg = Cw721Svg::new(mock.clone());
        let wlist = WhitelistMerkleTree::new(mock.clone());

        svg.upload()?;

        let init_msg = InstantiateMsg {
            name: "Test SVG Collection".to_string(),
            symbol: "TSVG".to_string(),
            svg_template: TEST_SVG_TEMPLATE.to_string(),
            variables: test_variables(),
            total: 100,
            seed: test_seed(),
            owner: Some(admin.to_string()),
            mint_start_time: None,
            mint_end_time: None,
            price_tiers: Vec::new(),
            payment_address: Some(admin.to_string()),
            whitelist: None,
            template_slots: compute_slots(TEST_SVG_TEMPLATE, &test_variables()),
        };

        svg.instantiate(&init_msg, Some(&admin), &[])?;

        Ok(CwSvgSuite {
            chain: mock,
            svg,
            admin,
            wlist,
        })
    }

    fn setup_with_delayed_start(delay_secs: u64) -> anyhow::Result<Self> {
        let mock = MockBech32::new("mock");
        let admin = mock.addr_make("admin");

        let svg = Cw721Svg::new(mock.clone());
        let wlist = WhitelistMerkleTree::new(mock.clone());

        svg.upload()?;

        let init_msg = InstantiateMsg {
            name: "Delayed SVG Collection".to_string(),
            symbol: "DSVG".to_string(),
            svg_template: TEST_SVG_TEMPLATE.to_string(),
            variables: test_variables(),
            total: 100,
            seed: test_seed(),
            owner: Some(admin.to_string()),
            mint_start_time: Some(delay_secs),
            mint_end_time: None,
            price_tiers: Vec::new(),
            payment_address: Some(admin.to_string()),
            whitelist: None,
            template_slots: compute_slots(TEST_SVG_TEMPLATE, &test_variables()),
        };

        svg.instantiate(&init_msg, Some(&admin), &[])?;

        Ok(CwSvgSuite {
            chain: mock,
            svg,
            admin,
            wlist,
        })
    }

    fn setup_with_end_time(end_secs: u64) -> anyhow::Result<Self> {
        let mock = MockBech32::new("mock");
        let admin = mock.addr_make("admin");

        let svg = Cw721Svg::new(mock.clone());
        let wlist = WhitelistMerkleTree::new(mock.clone());

        svg.upload()?;

        let init_msg = InstantiateMsg {
            name: "Cutoff SVG Collection".to_string(),
            symbol: "CSVG".to_string(),
            svg_template: TEST_SVG_TEMPLATE.to_string(),
            variables: test_variables(),
            total: 100,
            seed: test_seed(),
            owner: Some(admin.to_string()),
            mint_start_time: None,
            mint_end_time: Some(end_secs),
            price_tiers: Vec::new(),
            payment_address: Some(admin.to_string()),
            whitelist: None,
            template_slots: compute_slots(TEST_SVG_TEMPLATE, &test_variables()),
        };

        svg.instantiate(&init_msg, Some(&admin), &[])?;

        Ok(CwSvgSuite {
            chain: mock,
            svg,
            admin,
            wlist,
        })
    }

    fn setup_with_whitelist(
        members: &[String],
        price_tiers: Vec<PriceTier>,
    ) -> anyhow::Result<(Self, MerkleTree<SortingBlake3Hasher>)> {
        let mock = MockBech32::new("mock");
        let admin = mock.addr_make("admin");

        // Build merkle tree from member addresses
        let tree = hash_and_build_tree(members);
        let root = tree.root_hex().expect("non-empty tree");

        // Deploy whitelist contract
        let wlist = WhitelistMerkleTree::new(mock.clone());
        wlist.upload()?;

        let now = mock.block_info()?.time;
        let wl_init = whitelist_mtree::msg::InstantiateMsg {
            merkle_root: root,
            merkle_tree_uri: None,
            start_time: Timestamp::from_seconds(now.seconds() + 1),
            end_time: Timestamp::from_seconds(now.seconds() + 100_000),
            mint_price: coin(0, "ustars"),
            per_address_limit: 100,
            admins: vec![admin.to_string()],
            admins_mutable: false,
        };
        wlist.instantiate(&wl_init, Some(&admin), &[])?;
        mock.wait_seconds(2)?;

        // Deploy SVG contract with whitelist
        let svg = Cw721Svg::new(mock.clone());
        svg.upload()?;

        let svg_init = InstantiateMsg {
            name: "WL Collection".to_string(),
            symbol: "WLSVG".to_string(),
            svg_template: TEST_SVG_TEMPLATE.to_string(),
            variables: test_variables(),
            total: 100,
            seed: test_seed(),
            owner: Some(admin.to_string()),
            mint_start_time: None,
            mint_end_time: None,
            price_tiers,
            payment_address: Some(admin.to_string()),
            whitelist: Some(wlist.address()?.to_string()),
            template_slots: compute_slots(TEST_SVG_TEMPLATE, &test_variables()),
        };
        svg.instantiate(&svg_init, Some(&admin), &[])?;

        let suite = CwSvgSuite {
            chain: mock,
            svg,
            wlist,
            admin,
        };

        Ok((suite, tree))
    }
}

// ---------------------------------------------------------------------------
// Existing tests (fixed for whitelist + proof_hashes fields)
// ---------------------------------------------------------------------------

#[test]
fn test_successful_instantiate() -> anyhow::Result<()> {
    let suite = CwSvgSuite::setup()?;

    // Query config
    let config: ConfigResponse = suite.svg.query(&QueryMsg::Config {})?;
    assert_eq!(config.config.total, 100);
    assert_eq!(config.config.mint_count, 0);
    assert!(!config.config.paused);
    assert_eq!(config.config.seed, test_seed());

    // Query template
    let template: SvgTemplateResponse = suite.svg.query(&QueryMsg::SvgTemplate {})?;
    assert_eq!(template.template, TEST_SVG_TEMPLATE);

    // Query contract info via cw721
    let info: cw721::msg::CollectionInfoAndExtensionResponse<
        cw721::EmptyOptionalCollectionExtension,
    > = suite.svg.query(&QueryMsg::ContractInfo {})?;
    assert_eq!(info.name, "Test SVG Collection");
    assert_eq!(info.symbol, "TSVG");

    // Query num tokens (should be 0)
    let num: cw721::msg::NumTokensResponse = suite.svg.query(&QueryMsg::NumTokens {})?;
    assert_eq!(num.count, 0);

    Ok(())
}

#[test]
fn test_mint_single() -> anyhow::Result<()> {
    let suite = CwSvgSuite::setup()?;

    suite.svg.execute(&mint_msg(1), &[])?;

    // Verify mint count incremented
    let config: ConfigResponse = suite.svg.query(&QueryMsg::Config {})?;
    assert_eq!(config.config.mint_count, 1);

    // Verify num tokens
    let num: cw721::msg::NumTokensResponse = suite.svg.query(&QueryMsg::NumTokens {})?;
    assert_eq!(num.count, 1);

    // Verify token owner is the sender
    let owner: cw721::msg::OwnerOfResponse = suite.svg.query(&QueryMsg::OwnerOf {
        token_id: "0".to_string(),
        include_expired: None,
    })?;
    assert_eq!(owner.owner, suite.chain.sender_addr().to_string());

    // Verify NftInfo returns params
    let nft_info: cw721::msg::NftInfoResponse<SvgMetadata> =
        suite.svg.query(&QueryMsg::NftInfo {
            token_id: "0".to_string(),
        })?;
    assert_eq!(nft_info.extension.params.len(), 2);
    assert_eq!(nft_info.extension.params[0].name, "color_yin");
    assert_eq!(nft_info.extension.params[1].name, "color_yang");

    // Verify each param value is from the options list
    let vars = test_variables();
    assert!(options_list(&vars[0]).contains(&nft_info.extension.params[0].value));
    assert!(options_list(&vars[1]).contains(&nft_info.extension.params[1].value));

    Ok(())
}

#[test]
fn test_mint_multiple() -> anyhow::Result<()> {
    let suite = CwSvgSuite::setup()?;

    suite.svg.execute(&mint_msg(5), &[])?;

    // Verify mint count
    let config: ConfigResponse = suite.svg.query(&QueryMsg::Config {})?;
    assert_eq!(config.config.mint_count, 5);

    // Verify num tokens
    let num: cw721::msg::NumTokensResponse = suite.svg.query(&QueryMsg::NumTokens {})?;
    assert_eq!(num.count, 5);

    // Verify all tokens exist and have correct owner
    for i in 0..5 {
        let owner: cw721::msg::OwnerOfResponse = suite.svg.query(&QueryMsg::OwnerOf {
            token_id: i.to_string(),
            include_expired: None,
        })?;
        assert_eq!(owner.owner, suite.chain.sender_addr().to_string());
    }

    Ok(())
}

#[test]
fn test_entropy_produces_valid_params() -> anyhow::Result<()> {
    let suite = CwSvgSuite::setup()?;

    // Mint several tokens and verify all params are from the options
    suite.svg.execute(&mint_msg(10), &[])?;

    let vars = test_variables();
    for i in 0..10 {
        let nft_info: cw721::msg::NftInfoResponse<SvgMetadata> =
            suite.svg.query(&QueryMsg::NftInfo {
                token_id: i.to_string(),
            })?;

        for (idx, param) in nft_info.extension.params.iter().enumerate() {
            assert_eq!(param.name, vars[idx].name);
            let opts = options_list(&vars[idx]);
            assert!(
                opts.contains(&param.value),
                "Token {} param {} has value {} which is not in options {:?}",
                i,
                param.name,
                param.value,
                opts
            );
        }
    }

    Ok(())
}

#[test]
fn test_svg_token_uri_substitution() -> anyhow::Result<()> {
    let suite = CwSvgSuite::setup()?;

    suite.svg.execute(&mint_msg(1), &[])?;

    // Get the resolved SVG
    let svg_resp: SvgTokenUriResponse = suite.svg.query(&QueryMsg::SvgTokenUri {
        token_id: "0".to_string(),
    })?;

    // SVG should NOT contain any unresolved placeholders
    assert!(
        !svg_resp.svg.contains("${"),
        "SVG still contains unresolved placeholders: {}",
        svg_resp.svg
    );

    // Get the token params to verify substitution
    let nft_info: cw721::msg::NftInfoResponse<SvgMetadata> =
        suite.svg.query(&QueryMsg::NftInfo {
            token_id: "0".to_string(),
        })?;

    // Verify the SVG contains the resolved values
    for param in &nft_info.extension.params {
        assert!(
            svg_resp.svg.contains(&param.value),
            "SVG does not contain resolved value '{}' for param '{}'",
            param.value,
            param.name
        );
    }

    Ok(())
}

#[test]
fn test_mint_exceeds_total_supply() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let admin = mock.addr_make("admin");

    let svg = Cw721Svg::new(mock.clone());
    svg.upload()?;

    // Create collection with total supply of 3
    let init_msg = InstantiateMsg {
        name: "Tiny Collection".to_string(),
        symbol: "TINY".to_string(),
        svg_template: TEST_SVG_TEMPLATE.to_string(),
        variables: test_variables(),
        total: 3,
        seed: test_seed(),
        owner: Some(admin.to_string()),
        mint_start_time: None,
        mint_end_time: None,
        price_tiers: Vec::new(),
        payment_address: Some(admin.to_string()),
        whitelist: None,
        template_slots: compute_slots(TEST_SVG_TEMPLATE, &test_variables()),
    };
    svg.instantiate(&init_msg, Some(&admin), &[])?;

    // Mint 3 should succeed
    svg.execute(&mint_msg(3), &[])?;

    // Mint 1 more should fail
    svg.execute(&mint_msg(1), &[])
        .expect_err("should fail: exceeds total supply");

    Ok(())
}

#[test]
fn test_mint_partial_exceeds_total_supply() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let admin = mock.addr_make("admin");

    let svg = Cw721Svg::new(mock.clone());
    svg.upload()?;

    let init_msg = InstantiateMsg {
        name: "Small Collection".to_string(),
        symbol: "SMOL".to_string(),
        svg_template: TEST_SVG_TEMPLATE.to_string(),
        variables: test_variables(),
        total: 5,
        seed: test_seed(),
        owner: Some(admin.to_string()),
        mint_start_time: None,
        mint_end_time: None,
        price_tiers: Vec::new(),
        payment_address: Some(admin.to_string()),
        whitelist: None,
        template_slots: compute_slots(TEST_SVG_TEMPLATE, &test_variables()),
    };
    svg.instantiate(&init_msg, Some(&admin), &[])?;

    // Mint 3 succeeds
    svg.execute(&mint_msg(3), &[])?;

    // Mint 3 more exceeds total of 5
    svg.execute(&mint_msg(3), &[])
        .expect_err("should fail: exceeds total supply");

    // Mint 2 should still succeed (exactly at limit)
    svg.execute(&mint_msg(2), &[])?;

    let config: ConfigResponse = svg.query(&QueryMsg::Config {})?;
    assert_eq!(config.config.mint_count, 5);

    Ok(())
}

#[test]
fn test_pause_and_unpause() -> anyhow::Result<()> {
    let suite = CwSvgSuite::setup()?;

    // Minting works before pause
    suite.svg.execute(&mint_msg(1), &[])?;

    // Admin pauses
    suite.chain.call_as(&suite.admin).execute(
        &ExecuteMsg::Pause { pause: true },
        &[],
        &suite.svg.address()?,
    )?;

    // Minting fails when paused
    suite
        .svg
        .execute(&mint_msg(1), &[])
        .expect_err("should fail: minting is paused");

    // Admin unpauses
    suite.chain.call_as(&suite.admin).execute(
        &ExecuteMsg::Pause { pause: false },
        &[],
        &suite.svg.address()?,
    )?;

    // Minting works again
    suite.svg.execute(&mint_msg(1), &[])?;

    let config: ConfigResponse = suite.svg.query(&QueryMsg::Config {})?;
    assert_eq!(config.config.mint_count, 2);

    Ok(())
}

#[test]
fn test_pause_unauthorized() -> anyhow::Result<()> {
    let suite = CwSvgSuite::setup()?;

    // Non-owner tries to pause
    let rando = suite.chain.addr_make("rando");
    suite
        .chain
        .call_as(&rando)
        .execute(
            &ExecuteMsg::Pause { pause: true },
            &[],
            &suite.svg.address()?,
        )
        .expect_err("should fail: not the owner");

    Ok(())
}

#[test]
fn test_mint_before_start_time() -> anyhow::Result<()> {
    // Delay start by 1000 seconds
    let suite = CwSvgSuite::setup_with_delayed_start(1000)?;

    // Minting should fail because start time hasn't arrived
    suite
        .svg
        .execute(&mint_msg(1), &[])
        .expect_err("should fail: minting not started yet");

    Ok(())
}

#[test]
fn test_transfer_nft() -> anyhow::Result<()> {
    let suite = CwSvgSuite::setup()?;

    suite.svg.execute(&mint_msg(1), &[])?;

    let recipient = suite.chain.addr_make("recipient");

    suite.svg.execute(
        &ExecuteMsg::TransferNft {
            recipient: recipient.to_string(),
            token_id: "0".to_string(),
        },
        &[],
    )?;

    // Verify new owner
    let owner: cw721::msg::OwnerOfResponse = suite.svg.query(&QueryMsg::OwnerOf {
        token_id: "0".to_string(),
        include_expired: None,
    })?;
    assert_eq!(owner.owner, recipient.to_string());

    // Params should still be intact after transfer
    let nft_info: cw721::msg::NftInfoResponse<SvgMetadata> =
        suite.svg.query(&QueryMsg::NftInfo {
            token_id: "0".to_string(),
        })?;
    assert_eq!(nft_info.extension.params.len(), 2);

    Ok(())
}

#[test]
fn test_all_tokens_query() -> anyhow::Result<()> {
    let suite = CwSvgSuite::setup()?;

    suite.svg.execute(&mint_msg(5), &[])?;

    let tokens: cw721::msg::TokensResponse = suite.svg.query(&QueryMsg::AllTokens {
        start_after: None,
        limit: None,
    })?;
    assert_eq!(tokens.tokens.len(), 5);
    assert_eq!(tokens.tokens, vec!["0", "1", "2", "3", "4"]);

    Ok(())
}

#[test]
fn test_tokens_by_owner() -> anyhow::Result<()> {
    let suite = CwSvgSuite::setup()?;

    suite.svg.execute(&mint_msg(3), &[])?;

    let tokens: cw721::msg::TokensResponse = suite.svg.query(&QueryMsg::Tokens {
        owner: suite.chain.sender_addr().to_string(),
        start_after: None,
        limit: None,
    })?;
    assert_eq!(tokens.tokens.len(), 3);

    Ok(())
}

#[test]
fn test_svg_token_uri_multiple_tokens_differ() -> anyhow::Result<()> {
    let suite = CwSvgSuite::setup()?;

    suite.svg.execute(&mint_msg(10), &[])?;

    // Collect all SVG URIs
    let mut svgs = vec![];
    for i in 0..10 {
        let svg_resp: SvgTokenUriResponse = suite.svg.query(&QueryMsg::SvgTokenUri {
            token_id: i.to_string(),
        })?;
        svgs.push(svg_resp.svg);
    }

    // With 4 options per variable and 10 tokens, we should see some variation
    // (not all SVGs should be identical)
    let unique_count = svgs.iter().collect::<std::collections::HashSet<_>>().len();
    assert!(
        unique_count > 1,
        "Expected variation across 10 tokens, but all SVGs are identical"
    );

    Ok(())
}

#[test]
fn test_dao_builder_helper() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let admin = mock.addr_make("admin");

    let svg = Cw721Svg::new(mock.clone());
    svg.upload()?;

    // Use the cw-svg dao helper to build instantiate msg
    let init_msg = cw_svg::dao::dao_instantiate_msg(
        "8-Bit DAO",
        "8BITDAO",
        50,
        Binary::from(b"dao-seed-value"),
    );

    svg.instantiate(&init_msg, Some(&admin), &[])?;

    // Verify template is the yin-yang SVG
    let template: SvgTemplateResponse = svg.query(&QueryMsg::SvgTemplate {})?;
    assert!(template.template.contains("${color_yin}"));
    assert!(template.template.contains("${color_yang}"));

    // Mint and verify SVG resolves
    svg.execute(&mint_msg(1), &[])?;

    let svg_resp: SvgTokenUriResponse = svg.query(&QueryMsg::SvgTokenUri {
        token_id: "0".to_string(),
    })?;
    assert!(!svg_resp.svg.contains("${"));
    assert!(svg_resp.svg.contains("rgb("));

    Ok(())
}

#[test]
fn test_mintout_full_collection() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let admin = mock.addr_make("admin");

    let svg = Cw721Svg::new(mock.clone());
    svg.upload()?;

    let init_msg = InstantiateMsg {
        name: "Full Mint".to_string(),
        symbol: "FULL".to_string(),
        svg_template: TEST_SVG_TEMPLATE.to_string(),
        variables: test_variables(),
        total: 10,
        seed: test_seed(),
        owner: Some(admin.to_string()),
        mint_start_time: None,
        mint_end_time: None,
        price_tiers: Vec::new(),
        payment_address: Some(admin.to_string()),
        whitelist: None,
        template_slots: compute_slots(TEST_SVG_TEMPLATE, &test_variables()),
    };
    svg.instantiate(&init_msg, Some(&admin), &[])?;

    // Mint all 10
    svg.execute(&mint_msg(10), &[])?;

    let config: ConfigResponse = svg.query(&QueryMsg::Config {})?;
    assert_eq!(config.config.mint_count, 10);
    assert_eq!(config.config.total, 10);

    // Can't mint any more
    svg.execute(&mint_msg(1), &[])
        .expect_err("should fail: collection fully minted");

    // All tokens queryable
    let tokens: cw721::msg::TokensResponse = svg.query(&QueryMsg::AllTokens {
        start_after: None,
        limit: None,
    })?;
    assert_eq!(tokens.tokens.len(), 10);

    Ok(())
}

#[test]
fn test_approve_and_transfer_by_operator() -> anyhow::Result<()> {
    let suite = CwSvgSuite::setup()?;

    suite.svg.execute(&mint_msg(1), &[])?;

    let operator = suite.chain.addr_make("operator");
    let recipient = suite.chain.addr_make("recipient");

    // Owner approves operator for token 0
    suite.svg.execute(
        &ExecuteMsg::Approve {
            spender: operator.to_string(),
            token_id: "0".to_string(),
            expires: None,
        },
        &[],
    )?;

    // Operator transfers the token
    suite.chain.call_as(&operator).execute(
        &ExecuteMsg::TransferNft {
            recipient: recipient.to_string(),
            token_id: "0".to_string(),
        },
        &[],
        &suite.svg.address()?,
    )?;

    // Verify new owner
    let owner: cw721::msg::OwnerOfResponse = suite.svg.query(&QueryMsg::OwnerOf {
        token_id: "0".to_string(),
        include_expired: None,
    })?;
    assert_eq!(owner.owner, recipient.to_string());

    Ok(())
}

#[test]
fn test_all_nft_info() -> anyhow::Result<()> {
    let suite = CwSvgSuite::setup()?;

    suite.svg.execute(&mint_msg(1), &[])?;

    // AllNftInfo returns both owner and extension data
    let all_info: cw721::msg::AllNftInfoResponse<SvgMetadata> =
        suite.svg.query(&QueryMsg::AllNftInfo {
            token_id: "0".to_string(),
            include_expired: None,
        })?;

    assert_eq!(all_info.access.owner, suite.chain.sender_addr().to_string());
    assert_eq!(all_info.info.extension.params.len(), 2);
    assert_eq!(all_info.info.extension.params[0].name, "color_yin");
    assert_eq!(all_info.info.extension.params[1].name, "color_yang");

    Ok(())
}

#[test]
fn test_svg_template_too_large() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let admin = mock.addr_make("admin");

    let svg = Cw721Svg::new(mock.clone());
    svg.upload()?;

    // Create an SVG template larger than 0.420 MB (430,081 bytes)
    let oversized_template = "x".repeat(MAX_SVG_SIZE + 1);

    let init_msg = InstantiateMsg {
        name: "Too Large".to_string(),
        symbol: "BIG".to_string(),
        svg_template: oversized_template,
        variables: test_variables(),
        total: 10,
        seed: test_seed(),
        owner: Some(admin.to_string()),
        mint_start_time: None,
        mint_end_time: None,
        price_tiers: Vec::new(),
        payment_address: Some(admin.to_string()),
        whitelist: None,
        template_slots: vec![],
    };

    svg.instantiate(&init_msg, Some(&admin), &[])
        .expect_err("should fail: SVG template exceeds max size");

    Ok(())
}

#[test]
fn test_svg_template_at_max_size() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let admin = mock.addr_make("admin");

    let svg = Cw721Svg::new(mock.clone());
    svg.upload()?;

    // Exactly at the limit should succeed
    let max_template = "x".repeat(MAX_SVG_SIZE);

    let init_msg = InstantiateMsg {
        name: "Max Size".to_string(),
        symbol: "MAX".to_string(),
        svg_template: max_template,
        variables: vec![],
        total: 10,
        seed: test_seed(),
        owner: Some(admin.to_string()),
        mint_start_time: None,
        mint_end_time: None,
        price_tiers: Vec::new(),
        payment_address: Some(admin.to_string()),
        whitelist: None,
        template_slots: vec![],
    };

    svg.instantiate(&init_msg, Some(&admin), &[])?;

    Ok(())
}

#[test]
fn test_total_supply_too_high() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let admin = mock.addr_make("admin");

    let svg = Cw721Svg::new(mock.clone());
    svg.upload()?;

    let init_msg = InstantiateMsg {
        name: "Too Many".to_string(),
        symbol: "MANY".to_string(),
        svg_template: TEST_SVG_TEMPLATE.to_string(),
        variables: test_variables(),
        total: MAX_TOTAL_SUPPLY + 1,
        seed: test_seed(),
        owner: Some(admin.to_string()),
        mint_start_time: None,
        mint_end_time: None,
        price_tiers: Vec::new(),
        payment_address: Some(admin.to_string()),
        whitelist: None,
        template_slots: compute_slots(TEST_SVG_TEMPLATE, &test_variables()),
    };

    svg.instantiate(&init_msg, Some(&admin), &[])
        .expect_err("should fail: total supply exceeds maximum");

    Ok(())
}

#[test]
fn test_total_supply_at_max() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let admin = mock.addr_make("admin");

    let svg = Cw721Svg::new(mock.clone());
    svg.upload()?;

    // Exactly at max should succeed
    let init_msg = InstantiateMsg {
        name: "Max Supply".to_string(),
        symbol: "MAXS".to_string(),
        svg_template: TEST_SVG_TEMPLATE.to_string(),
        variables: test_variables(),
        total: MAX_TOTAL_SUPPLY,
        seed: test_seed(),
        owner: Some(admin.to_string()),
        mint_start_time: None,
        mint_end_time: None,
        price_tiers: Vec::new(),
        payment_address: Some(admin.to_string()),
        whitelist: None,
        template_slots: compute_slots(TEST_SVG_TEMPLATE, &test_variables()),
    };

    svg.instantiate(&init_msg, Some(&admin), &[])?;

    let config: ConfigResponse = svg.query(&QueryMsg::Config {})?;
    assert_eq!(config.config.total, MAX_TOTAL_SUPPLY);

    Ok(())
}

#[test]
fn test_mint_end_time_stored_in_config() -> anyhow::Result<()> {
    let suite = CwSvgSuite::setup_with_end_time(3600)?;

    let config: ConfigResponse = suite.svg.query(&QueryMsg::Config {})?;
    assert!(config.config.mint_end_time.is_some());

    Ok(())
}

#[test]
fn test_no_mint_end_time() -> anyhow::Result<()> {
    let suite = CwSvgSuite::setup()?;

    let config: ConfigResponse = suite.svg.query(&QueryMsg::Config {})?;
    assert!(config.config.mint_end_time.is_none());

    // Minting works without an end time
    suite.svg.execute(&mint_msg(1), &[])?;

    Ok(())
}

// ===========================================================================
// Price tier tests
// ===========================================================================

#[test]
fn test_price_tier_single_tier() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let admin = mock.addr_make("admin");
    let treasury = mock.addr_make("treasury");
    let minter = mock.addr_make_with_balance("minter", coins(10_000_000_000, "ustars"))?;

    let svg = Cw721Svg::new(mock.clone());
    svg.upload()?;

    let init_msg = InstantiateMsg {
        name: "Priced Collection".to_string(),
        symbol: "PRCD".to_string(),
        svg_template: TEST_SVG_TEMPLATE.to_string(),
        variables: test_variables(),
        total: 100,
        seed: test_seed(),
        owner: Some(admin.to_string()),
        mint_start_time: None,
        mint_end_time: None,
        price_tiers: vec![PriceTier {
            until_count: 100,
            price: coin(1_000_000, "ustars"),
        }],
        payment_address: Some(treasury.to_string()),
        whitelist: None,
        template_slots: compute_slots(TEST_SVG_TEMPLATE, &test_variables()),
    };
    svg.instantiate(&init_msg, Some(&admin), &[])?;

    // Mint without funds should fail
    mock.call_as(&minter)
        .execute(&mint_msg(1), &[], &svg.address()?)
        .expect_err("should fail: no payment");

    // Mint with wrong amount should fail
    mock.call_as(&minter)
        .execute(&mint_msg(1), &coins(500_000, "ustars"), &svg.address()?)
        .expect_err("should fail: incorrect payment");

    // Mint with correct amount should succeed
    mock.call_as(&minter)
        .execute(&mint_msg(1), &coins(1_000_000, "ustars"), &svg.address()?)?;

    let config: ConfigResponse = svg.query(&QueryMsg::Config {})?;
    assert_eq!(config.config.mint_count, 1);

    Ok(())
}

#[test]
fn test_price_tier_batch_mint_cost() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let admin = mock.addr_make("admin");
    let treasury = mock.addr_make("treasury");
    let minter = mock.addr_make_with_balance("minter", coins(100_000_000_000, "ustars"))?;

    let svg = Cw721Svg::new(mock.clone());
    svg.upload()?;

    let init_msg = InstantiateMsg {
        name: "Batch Price".to_string(),
        symbol: "BTCH".to_string(),
        svg_template: TEST_SVG_TEMPLATE.to_string(),
        variables: test_variables(),
        total: 100,
        seed: test_seed(),
        owner: Some(admin.to_string()),
        mint_start_time: None,
        mint_end_time: None,
        price_tiers: vec![PriceTier {
            until_count: 100,
            price: coin(2_000_000, "ustars"),
        }],
        payment_address: Some(treasury.to_string()),
        whitelist: None,
        template_slots: compute_slots(TEST_SVG_TEMPLATE, &test_variables()),
    };
    svg.instantiate(&init_msg, Some(&admin), &[])?;

    // Batch mint 5 tokens: should cost 5 * 2_000_000 = 10_000_000
    mock.call_as(&minter)
        .execute(&mint_msg(5), &coins(10_000_000, "ustars"), &svg.address()?)?;

    let config: ConfigResponse = svg.query(&QueryMsg::Config {})?;
    assert_eq!(config.config.mint_count, 5);

    // Wrong total for batch should fail
    mock.call_as(&minter)
        .execute(
            &mint_msg(3),
            &coins(5_000_000, "ustars"), // should be 6_000_000
            &svg.address()?,
        )
        .expect_err("should fail: incorrect batch payment");

    Ok(())
}

#[test]
fn test_price_tier_multiple_tiers() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let admin = mock.addr_make("admin");
    let treasury = mock.addr_make("treasury");
    let minter = mock.addr_make_with_balance("minter", coins(100_000_000_000, "ustars"))?;

    let svg = Cw721Svg::new(mock.clone());
    svg.upload()?;

    // Tier 1: first 3 mints cost 1_000_000
    // Tier 2: mints 3-6 cost 5_000_000
    // Tier 3: mints 6-10 cost 10_000_000
    let init_msg = InstantiateMsg {
        name: "Tiered".to_string(),
        symbol: "TIER".to_string(),
        svg_template: TEST_SVG_TEMPLATE.to_string(),
        variables: test_variables(),
        total: 10,
        seed: test_seed(),
        owner: Some(admin.to_string()),
        mint_start_time: None,
        mint_end_time: None,
        price_tiers: vec![
            PriceTier {
                until_count: 3,
                price: coin(1_000_000, "ustars"),
            },
            PriceTier {
                until_count: 6,
                price: coin(5_000_000, "ustars"),
            },
            PriceTier {
                until_count: 10,
                price: coin(10_000_000, "ustars"),
            },
        ],
        payment_address: Some(treasury.to_string()),
        whitelist: None,
        template_slots: compute_slots(TEST_SVG_TEMPLATE, &test_variables()),
    };
    svg.instantiate(&init_msg, Some(&admin), &[])?;

    // Mint 3 in tier 1: 3 * 1_000_000 = 3_000_000
    mock.call_as(&minter)
        .execute(&mint_msg(3), &coins(3_000_000, "ustars"), &svg.address()?)?;

    let config: ConfigResponse = svg.query(&QueryMsg::Config {})?;
    assert_eq!(config.config.mint_count, 3);

    // Mint 3 in tier 2: 3 * 5_000_000 = 15_000_000
    mock.call_as(&minter)
        .execute(&mint_msg(3), &coins(15_000_000, "ustars"), &svg.address()?)?;

    let config: ConfigResponse = svg.query(&QueryMsg::Config {})?;
    assert_eq!(config.config.mint_count, 6);

    // Mint 2 in tier 3: 2 * 10_000_000 = 20_000_000
    mock.call_as(&minter)
        .execute(&mint_msg(2), &coins(20_000_000, "ustars"), &svg.address()?)?;

    let config: ConfigResponse = svg.query(&QueryMsg::Config {})?;
    assert_eq!(config.config.mint_count, 8);

    Ok(())
}

#[test]
fn test_price_tier_cross_boundary_mint() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let admin = mock.addr_make("admin");
    let treasury = mock.addr_make("treasury");
    let minter = mock.addr_make_with_balance("minter", coins(100_000_000_000, "ustars"))?;

    let svg = Cw721Svg::new(mock.clone());
    svg.upload()?;

    // Tier 1: first 2 mints cost 1_000_000
    // Tier 2: mints 2-5 cost 5_000_000
    let init_msg = InstantiateMsg {
        name: "Boundary".to_string(),
        symbol: "BNDRY".to_string(),
        svg_template: TEST_SVG_TEMPLATE.to_string(),
        variables: test_variables(),
        total: 5,
        seed: test_seed(),
        owner: Some(admin.to_string()),
        mint_start_time: None,
        mint_end_time: None,
        price_tiers: vec![
            PriceTier {
                until_count: 2,
                price: coin(1_000_000, "ustars"),
            },
            PriceTier {
                until_count: 5,
                price: coin(5_000_000, "ustars"),
            },
        ],
        payment_address: Some(treasury.to_string()),
        whitelist: None,
        template_slots: compute_slots(TEST_SVG_TEMPLATE, &test_variables()),
    };
    svg.instantiate(&init_msg, Some(&admin), &[])?;

    // Mint 3 spanning tier boundary: 2 * 1_000_000 + 1 * 5_000_000 = 7_000_000
    mock.call_as(&minter)
        .execute(&mint_msg(3), &coins(7_000_000, "ustars"), &svg.address()?)?;

    let config: ConfigResponse = svg.query(&QueryMsg::Config {})?;
    assert_eq!(config.config.mint_count, 3);

    Ok(())
}

#[test]
fn test_price_tier_free_mint_rejects_funds() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let admin = mock.addr_make("admin");
    let minter = mock.addr_make_with_balance("minter", coins(10_000_000, "ustars"))?;

    let svg = Cw721Svg::new(mock.clone());
    svg.upload()?;

    // No price tiers = free mint
    let init_msg = InstantiateMsg {
        name: "Free Mint".to_string(),
        symbol: "FREE".to_string(),
        svg_template: TEST_SVG_TEMPLATE.to_string(),
        variables: test_variables(),
        total: 100,
        seed: test_seed(),
        owner: Some(admin.to_string()),
        mint_start_time: None,
        mint_end_time: None,
        price_tiers: Vec::new(),
        payment_address: Some(admin.to_string()),
        whitelist: None,
        template_slots: compute_slots(TEST_SVG_TEMPLATE, &test_variables()),
    };
    svg.instantiate(&init_msg, Some(&admin), &[])?;

    // Free mint with no funds should succeed
    mock.call_as(&minter)
        .execute(&mint_msg(1), &[], &svg.address()?)?;

    // Free mint with funds should fail
    mock.call_as(&minter)
        .execute(&mint_msg(1), &coins(1_000_000, "ustars"), &svg.address()?)
        .expect_err("should fail: free mint rejects funds");

    Ok(())
}

#[test]
fn test_price_tier_payment_goes_to_treasury() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let admin = mock.addr_make("admin");
    let treasury = mock.addr_make("treasury");
    let minter = mock.addr_make_with_balance("minter", coins(100_000_000, "ustars"))?;

    let svg = Cw721Svg::new(mock.clone());
    svg.upload()?;

    let init_msg = InstantiateMsg {
        name: "Treasury Test".to_string(),
        symbol: "TRSY".to_string(),
        svg_template: TEST_SVG_TEMPLATE.to_string(),
        variables: test_variables(),
        total: 100,
        seed: test_seed(),
        owner: Some(admin.to_string()),
        mint_start_time: None,
        mint_end_time: None,
        price_tiers: vec![PriceTier {
            until_count: 100,
            price: coin(5_000_000, "ustars"),
        }],
        payment_address: Some(treasury.to_string()),
        whitelist: None,
        template_slots: compute_slots(TEST_SVG_TEMPLATE, &test_variables()),
    };
    svg.instantiate(&init_msg, Some(&admin), &[])?;

    // Get treasury balance before
    let treasury_before = mock.query_balance(&treasury, "ustars")?;

    // Mint 2 tokens: 2 * 5_000_000 = 10_000_000
    mock.call_as(&minter)
        .execute(&mint_msg(2), &coins(10_000_000, "ustars"), &svg.address()?)?;

    // Treasury should have received the payment
    let treasury_after = mock.query_balance(&treasury, "ustars")?;
    assert_eq!(
        treasury_after.u128() - treasury_before.u128(),
        10_000_000,
        "Treasury should have received 10_000_000 ustars"
    );

    Ok(())
}

// ===========================================================================
// Mint start / end time enforcement tests
// ===========================================================================

#[test]
fn test_delayed_start_blocks_early_mints() -> anyhow::Result<()> {
    let suite = CwSvgSuite::setup_with_delayed_start(600)?;

    // Immediate mint should fail (start is 600 seconds in the future)
    suite
        .svg
        .execute(&mint_msg(1), &[])
        .expect_err("should fail: mint not started");

    Ok(())
}

#[test]
fn test_delayed_start_allows_mints_after_wait() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let admin = mock.addr_make("admin");

    let svg = Cw721Svg::new(mock.clone());
    svg.upload()?;

    // Delay by 100 seconds
    let init_msg = InstantiateMsg {
        name: "Delayed".to_string(),
        symbol: "DLY".to_string(),
        svg_template: TEST_SVG_TEMPLATE.to_string(),
        variables: test_variables(),
        total: 100,
        seed: test_seed(),
        owner: Some(admin.to_string()),
        mint_start_time: Some(100),
        mint_end_time: None,
        price_tiers: Vec::new(),
        payment_address: Some(admin.to_string()),
        whitelist: None,
        template_slots: compute_slots(TEST_SVG_TEMPLATE, &test_variables()),
    };
    svg.instantiate(&init_msg, Some(&admin), &[])?;

    // Should fail now
    svg.execute(&mint_msg(1), &[])
        .expect_err("should fail: not started yet");

    // Advance time past the start
    mock.wait_seconds(200)?;

    // Should succeed now
    svg.execute(&mint_msg(1), &[])?;

    let config: ConfigResponse = svg.query(&QueryMsg::Config {})?;
    assert_eq!(config.config.mint_count, 1);

    Ok(())
}

#[test]
fn test_end_time_blocks_late_mints() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let admin = mock.addr_make("admin");

    let svg = Cw721Svg::new(mock.clone());
    svg.upload()?;

    // End time 100 seconds from instantiation
    let init_msg = InstantiateMsg {
        name: "Cutoff".to_string(),
        symbol: "CUT".to_string(),
        svg_template: TEST_SVG_TEMPLATE.to_string(),
        variables: test_variables(),
        total: 100,
        seed: test_seed(),
        owner: Some(admin.to_string()),
        mint_start_time: None,
        mint_end_time: Some(100),
        price_tiers: Vec::new(),
        payment_address: Some(admin.to_string()),
        whitelist: None,
        template_slots: compute_slots(TEST_SVG_TEMPLATE, &test_variables()),
    };
    svg.instantiate(&init_msg, Some(&admin), &[])?;

    // Should succeed before end time
    svg.execute(&mint_msg(1), &[])?;

    // Advance past end time
    mock.wait_seconds(200)?;

    // Should fail after end time
    svg.execute(&mint_msg(1), &[])
        .expect_err("should fail: minting ended");

    let config: ConfigResponse = svg.query(&QueryMsg::Config {})?;
    assert_eq!(config.config.mint_count, 1);

    Ok(())
}

#[test]
fn test_start_and_end_time_window() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let admin = mock.addr_make("admin");

    let svg = Cw721Svg::new(mock.clone());
    svg.upload()?;

    // Mint window: starts 50s from now, ends 150s from now
    let init_msg = InstantiateMsg {
        name: "Window".to_string(),
        symbol: "WIN".to_string(),
        svg_template: TEST_SVG_TEMPLATE.to_string(),
        variables: test_variables(),
        total: 100,
        seed: test_seed(),
        owner: Some(admin.to_string()),
        mint_start_time: Some(50),
        mint_end_time: Some(150),
        price_tiers: Vec::new(),
        payment_address: Some(admin.to_string()),
        whitelist: None,
        template_slots: compute_slots(TEST_SVG_TEMPLATE, &test_variables()),
    };
    svg.instantiate(&init_msg, Some(&admin), &[])?;

    // Too early
    svg.execute(&mint_msg(1), &[])
        .expect_err("should fail: before start");

    // Advance into the window
    mock.wait_seconds(60)?;
    svg.execute(&mint_msg(1), &[])?;

    // Advance past end
    mock.wait_seconds(200)?;
    svg.execute(&mint_msg(1), &[])
        .expect_err("should fail: after end");

    let config: ConfigResponse = svg.query(&QueryMsg::Config {})?;
    assert_eq!(config.config.mint_count, 1);

    Ok(())
}

// ===========================================================================
// Whitelist proof-of-inclusion tests
// ===========================================================================

#[test]
fn test_whitelist_bypasses_payment() -> anyhow::Result<()> {
    let mock_temp = MockBech32::new("mock");
    let wl_member_addr = mock_temp.addr_make("whitelisted");

    let members = vec![wl_member_addr.to_string()];
    let (suite, tree) = CwSvgSuite::setup_with_whitelist(
        &members,
        vec![PriceTier {
            until_count: 100,
            price: coin(5_000_000, "ustars"),
        }],
    )?;

    // The whitelisted address — get its proof
    let proof = tree.proof(&[0]).proof_hashes_hex();

    // Non-whitelisted user must pay
    let non_wl = suite
        .chain
        .addr_make_with_balance("normie", coins(100_000_000, "ustars"))?;
    suite
        .chain
        .call_as(&non_wl)
        .execute(&mint_msg(1), &[], &suite.svg.address()?)
        .expect_err("should fail: non-whitelisted without payment");

    // Whitelisted user can mint for free with valid proof
    suite.chain.call_as(&wl_member_addr).execute(
        &ExecuteMsg::Mint {
            amount: 1,
            proof_hashes: Some(proof),
            allocation: None,
        },
        &[],
        &suite.svg.address()?,
    )?;

    let config: ConfigResponse = suite.svg.query(&QueryMsg::Config {})?;
    assert_eq!(config.config.mint_count, 1);

    Ok(())
}

#[test]
fn test_whitelist_invalid_proof_rejected() -> anyhow::Result<()> {
    let mock_temp = MockBech32::new("mock");
    let wl_member_addr = mock_temp.addr_make("whitelisted");
    let impostor_addr = mock_temp.addr_make("impostor");

    let members = vec![wl_member_addr.to_string()];
    let (suite, tree) = CwSvgSuite::setup_with_whitelist(&members, vec![])?;

    // Get valid proof for the whitelisted address
    let proof = tree.proof(&[0]).proof_hashes_hex();

    // Impostor tries to use the proof — should fail
    suite
        .chain
        .call_as(&impostor_addr)
        .execute(
            &ExecuteMsg::Mint {
                amount: 1,
                proof_hashes: Some(proof),
                allocation: None,
            },
            &[],
            &suite.svg.address()?,
        )
        .expect_err("should fail: impostor using someone else's proof");

    Ok(())
}

#[test]
fn test_whitelist_multiple_members() -> anyhow::Result<()> {
    let mock_temp = MockBech32::new("mock");
    let alice = mock_temp.addr_make("alice");
    let bob = mock_temp.addr_make("bob");
    let charlie = mock_temp.addr_make("charlie");
    let dave = mock_temp.addr_make("dave");

    let members = vec![
        alice.to_string(),
        bob.to_string(),
        charlie.to_string(),
        dave.to_string(),
    ];
    let (suite, tree) = CwSvgSuite::setup_with_whitelist(&members, vec![])?;

    // Each member can mint with their own proof
    for (i, member) in [&alice, &bob, &charlie, &dave].iter().enumerate() {
        let proof = tree.proof(&[i]).proof_hashes_hex();
        suite.chain.call_as(member).execute(
            &ExecuteMsg::Mint {
                amount: 1,
                proof_hashes: Some(proof),
                allocation: None,
            },
            &[],
            &suite.svg.address()?,
        )?;
    }

    let config: ConfigResponse = suite.svg.query(&QueryMsg::Config {})?;
    assert_eq!(config.config.mint_count, 4);

    // Verify each owns their token
    for (i, member) in [&alice, &bob, &charlie, &dave].iter().enumerate() {
        let owner: cw721::msg::OwnerOfResponse = suite.svg.query(&QueryMsg::OwnerOf {
            token_id: i.to_string(),
            include_expired: None,
        })?;
        assert_eq!(owner.owner, member.to_string());
    }

    Ok(())
}

#[test]
fn test_whitelist_no_proof_requires_payment() -> anyhow::Result<()> {
    let mock_temp = MockBech32::new("mock");
    let wl_member = mock_temp.addr_make("whitelisted");

    let members = vec![wl_member.to_string()];
    let (suite, _tree) = CwSvgSuite::setup_with_whitelist(
        &members,
        vec![PriceTier {
            until_count: 100,
            price: coin(1_000_000, "ustars"),
        }],
    )?;

    let funded_wl = suite
        .chain
        .addr_make_with_balance("whitelisted", coins(10_000_000, "ustars"))?;

    // Whitelisted member mints without providing proof — should need payment
    suite
        .chain
        .call_as(&funded_wl)
        .execute(&mint_msg(1), &[], &suite.svg.address()?)
        .expect_err("should fail: no proof means normal payment rules apply");

    // With correct payment, should succeed even without proof
    suite.chain.call_as(&funded_wl).execute(
        &mint_msg(1),
        &coins(1_000_000, "ustars"),
        &suite.svg.address()?,
    )?;

    let config: ConfigResponse = suite.svg.query(&QueryMsg::Config {})?;
    assert_eq!(config.config.mint_count, 1);

    Ok(())
}

#[test]
fn test_whitelist_tracks_wl_mint_count() -> anyhow::Result<()> {
    let mock_temp = MockBech32::new("mock");
    let wl_member = mock_temp.addr_make("whitelisted");

    let members = vec![wl_member.to_string()];
    let (suite, tree) = CwSvgSuite::setup_with_whitelist(&members, vec![])?;

    let proof = tree.proof(&[0]).proof_hashes_hex();

    // Mint 3 tokens via whitelist
    suite.chain.call_as(&wl_member).execute(
        &ExecuteMsg::Mint {
            amount: 3,
            proof_hashes: Some(proof),
            allocation: None,
        },
        &[],
        &suite.svg.address()?,
    )?;

    // Query mint count for the address
    let count: MintCountResponse = suite.svg.query(&QueryMsg::MintCount {
        address: wl_member.to_string(),
    })?;
    assert_eq!(count.count, 3);

    Ok(())
}

#[test]
fn test_whitelist_update_by_admin() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let admin = mock.addr_make("admin");

    let svg = Cw721Svg::new(mock.clone());
    svg.upload()?;

    // Start without whitelist
    let init_msg = InstantiateMsg {
        name: "No WL".to_string(),
        symbol: "NOWL".to_string(),
        svg_template: TEST_SVG_TEMPLATE.to_string(),
        variables: test_variables(),
        total: 100,
        seed: test_seed(),
        owner: Some(admin.to_string()),
        mint_start_time: None,
        mint_end_time: None,
        price_tiers: Vec::new(),
        payment_address: Some(admin.to_string()),
        whitelist: None,
        template_slots: compute_slots(TEST_SVG_TEMPLATE, &test_variables()),
    };
    svg.instantiate(&init_msg, Some(&admin), &[])?;

    // Query whitelist — should be None
    let wl: Option<Addr> = svg.query(&QueryMsg::Whitelist {})?;
    assert!(wl.is_none());

    // Admin sets a whitelist
    let fake_wl_addr = mock.addr_make("fake_whitelist");
    mock.call_as(&admin).execute(
        &ExecuteMsg::UpdateWhitelist {
            address: Some(fake_wl_addr.to_string()),
        },
        &[],
        &svg.address()?,
    )?;

    // Query whitelist — should be set
    let wl: Option<Addr> = svg.query(&QueryMsg::Whitelist {})?;
    assert_eq!(wl, Some(fake_wl_addr.clone()));

    // Admin removes whitelist
    mock.call_as(&admin).execute(
        &ExecuteMsg::UpdateWhitelist { address: None },
        &[],
        &svg.address()?,
    )?;

    let wl: Option<Addr> = svg.query(&QueryMsg::Whitelist {})?;
    assert!(wl.is_none());

    Ok(())
}

// ===========================================================================
// Template slot validation tests
// ===========================================================================

#[test]
fn test_template_slot_var_idx_out_of_range() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let admin = mock.addr_make("admin");

    let svg = Cw721Svg::new(mock.clone());
    svg.upload()?;

    // Slot references var_idx=99 which is out of range
    let init_msg = InstantiateMsg {
        name: "Bad Idx".to_string(),
        symbol: "BIDX".to_string(),
        svg_template: TEST_SVG_TEMPLATE.to_string(),
        variables: test_variables(),
        total: 10,
        seed: test_seed(),
        owner: Some(admin.to_string()),
        mint_start_time: None,
        mint_end_time: None,
        price_tiers: Vec::new(),
        payment_address: Some(admin.to_string()),
        whitelist: None,
        template_slots: vec![TemplateSlot {
            start: 19,
            end: 31,
            var_idx: 99,
        }],
    };

    svg.instantiate(&init_msg, Some(&admin), &[])
        .expect_err("should fail: var_idx out of range");

    Ok(())
}

#[test]
fn test_template_slot_content_mismatch() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let admin = mock.addr_make("admin");

    let svg = Cw721Svg::new(mock.clone());
    svg.upload()?;

    // Slot claims ${color_yin} is at wrong position
    let init_msg = InstantiateMsg {
        name: "Bad Pos".to_string(),
        symbol: "BPOS".to_string(),
        svg_template: TEST_SVG_TEMPLATE.to_string(),
        variables: test_variables(),
        total: 10,
        seed: test_seed(),
        owner: Some(admin.to_string()),
        mint_start_time: None,
        mint_end_time: None,
        price_tiers: Vec::new(),
        payment_address: Some(admin.to_string()),
        whitelist: None,
        template_slots: vec![TemplateSlot {
            start: 0,
            end: 12,
            var_idx: 0,
        }],
    };

    svg.instantiate(&init_msg, Some(&admin), &[])
        .expect_err("should fail: slot content does not match template");

    Ok(())
}

#[test]
fn test_whitelist_update_unauthorized() -> anyhow::Result<()> {
    let suite = CwSvgSuite::setup()?;

    let rando = suite.chain.addr_make("rando");
    let fake_wl = suite.chain.addr_make("fake_wl");

    suite
        .chain
        .call_as(&rando)
        .execute(
            &ExecuteMsg::UpdateWhitelist {
                address: Some(fake_wl.to_string()),
            },
            &[],
            &suite.svg.address()?,
        )
        .expect_err("should fail: not the owner");

    Ok(())
}

#[test]
fn test_mint_count_query() -> anyhow::Result<()> {
    let suite = CwSvgSuite::setup()?;

    let minter = suite.chain.sender_addr();

    // Before minting
    let count: MintCountResponse = suite.svg.query(&QueryMsg::MintCount {
        address: minter.to_string(),
    })?;
    assert_eq!(count.count, 0);

    // Mint some
    suite.svg.execute(&mint_msg(3), &[])?;

    let count: MintCountResponse = suite.svg.query(&QueryMsg::MintCount {
        address: minter.to_string(),
    })?;
    assert_eq!(count.count, 3);

    // Mint more
    suite.svg.execute(&mint_msg(2), &[])?;

    let count: MintCountResponse = suite.svg.query(&QueryMsg::MintCount {
        address: minter.to_string(),
    })?;
    assert_eq!(count.count, 5);

    Ok(())
}

// ===========================================================================
// Placeholder query tests
// ===========================================================================

#[test]
fn test_svg_placeholder_query() -> anyhow::Result<()> {
    let suite = CwSvgSuite::setup()?;

    // Query placeholder without seed
    let resp: SvgTokenUriResponse = suite.svg.query(&QueryMsg::SvgPlaceholder { seed: None })?;

    // Should not contain unresolved placeholders
    assert!(
        !resp.svg.contains("${"),
        "Placeholder SVG still contains unresolved placeholders: {}",
        resp.svg
    );

    // Should contain resolved color values
    assert!(
        resp.svg.contains("rgb("),
        "Placeholder SVG should contain rgb values"
    );

    Ok(())
}

#[test]
fn test_svg_placeholder_different_seeds_vary() -> anyhow::Result<()> {
    let suite = CwSvgSuite::setup()?;

    let resp1: SvgTokenUriResponse = suite.svg.query(&QueryMsg::SvgPlaceholder {
        seed: Some("seed1".to_string()),
    })?;
    let resp2: SvgTokenUriResponse = suite.svg.query(&QueryMsg::SvgPlaceholder {
        seed: Some("seed2".to_string()),
    })?;
    let resp3: SvgTokenUriResponse = suite.svg.query(&QueryMsg::SvgPlaceholder {
        seed: Some("seed1".to_string()),
    })?;

    // Same seed produces same result
    assert_eq!(resp1.svg, resp3.svg, "Same seed should produce same SVG");

    // Different seeds may produce different results (not guaranteed with few options,
    // but both should be valid)
    assert!(
        !resp1.svg.contains("${"),
        "SVG should not contain unresolved placeholders"
    );
    assert!(
        !resp2.svg.contains("${"),
        "SVG should not contain unresolved placeholders"
    );

    Ok(())
}

// ===========================================================================
// Rgb variable kind tests
// ===========================================================================

#[test]
fn test_rgb_variable_kind() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let admin = mock.addr_make("admin");

    let svg = Cw721Svg::new(mock.clone());
    svg.upload()?;

    let template = "<svg><rect fill='${bg}'/></svg>";
    let variables = vec![VariableDef {
        name: "bg".to_string(),
        kind: VariableKind::Rgb,
    }];

    let init_msg = InstantiateMsg {
        name: "Rgb Test".to_string(),
        symbol: "RGB".to_string(),
        svg_template: template.to_string(),
        variables: variables.clone(),
        total: 10,
        seed: test_seed(),
        owner: Some(admin.to_string()),
        mint_start_time: None,
        mint_end_time: None,
        price_tiers: Vec::new(),
        payment_address: Some(admin.to_string()),
        whitelist: None,
        template_slots: compute_slots(template, &variables),
    };
    svg.instantiate(&init_msg, Some(&admin), &[])?;

    // Mint a token
    svg.execute(&mint_msg(1), &[])?;

    // Query the SVG
    let resp: SvgTokenUriResponse = svg.query(&QueryMsg::SvgTokenUri {
        token_id: "0".to_string(),
    })?;

    // Should contain rgb(N,N,N) and no unresolved placeholders
    assert!(
        !resp.svg.contains("${"),
        "SVG still contains unresolved placeholders: {}",
        resp.svg
    );
    assert!(
        resp.svg.contains("rgb("),
        "SVG should contain an rgb() value: {}",
        resp.svg
    );

    // Verify the param stored on the token is a valid rgb string
    let nft_info: cw721::msg::NftInfoResponse<SvgMetadata> = svg.query(&QueryMsg::NftInfo {
        token_id: "0".to_string(),
    })?;
    let bg_value = &nft_info.extension.params[0].value;
    assert!(
        bg_value.starts_with("rgb(") && bg_value.ends_with(')'),
        "Expected rgb(...) format, got: {}",
        bg_value
    );

    Ok(())
}

// ===========================================================================
// Rgb Styled variable kind tests
// ===========================================================================

#[test]
fn test_rgb_styled_variable_kind() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let admin = mock.addr_make("admin");

    let svg = Cw721Svg::new(mock.clone());
    svg.upload()?;

    let template = "<svg><rect fill='${accent}'/></svg>";
    let variables = vec![VariableDef {
        name: "accent".to_string(),
        kind: VariableKind::RgbStyled(vec![
            // Warm reds
            RgbRange {
                r_min: 180,
                r_max: 255,
                g_min: 0,
                g_max: 80,
                b_min: 0,
                b_max: 60,
            },
            // Cool blues
            RgbRange {
                r_min: 0,
                r_max: 60,
                g_min: 50,
                g_max: 150,
                b_min: 180,
                b_max: 255,
            },
        ]),
    }];

    let init_msg = InstantiateMsg {
        name: "Styled Rgb Test".to_string(),
        symbol: "SRGB".to_string(),
        svg_template: template.to_string(),
        variables: variables.clone(),
        total: 50,
        seed: test_seed(),
        owner: Some(admin.to_string()),
        mint_start_time: None,
        mint_end_time: None,
        price_tiers: Vec::new(),
        payment_address: Some(admin.to_string()),
        whitelist: None,
        template_slots: compute_slots(template, &variables),
    };
    svg.instantiate(&init_msg, Some(&admin), &[])?;

    // Mint several tokens
    svg.execute(&mint_msg(10), &[])?;

    for i in 0..10 {
        let resp: SvgTokenUriResponse = svg.query(&QueryMsg::SvgTokenUri {
            token_id: i.to_string(),
        })?;

        assert!(
            !resp.svg.contains("${"),
            "Token {} SVG has unresolved placeholders: {}",
            i,
            resp.svg
        );
        assert!(
            resp.svg.contains("rgb("),
            "Token {} SVG should contain rgb(): {}",
            i,
            resp.svg
        );

        // Parse the rgb values and verify they fall within one of the defined ranges
        let nft_info: cw721::msg::NftInfoResponse<SvgMetadata> = svg.query(&QueryMsg::NftInfo {
            token_id: i.to_string(),
        })?;
        let val = &nft_info.extension.params[0].value;
        let inner = val.strip_prefix("rgb(").unwrap().strip_suffix(')').unwrap();
        let parts: Vec<u8> = inner.split(',').map(|s| s.parse().unwrap()).collect();
        let (r, g, b) = (parts[0], parts[1], parts[2]);

        let in_warm = r >= 180 && g <= 80 && b <= 60;
        let in_cool = r <= 60 && (50..=150).contains(&g) && b >= 180;
        assert!(
            in_warm || in_cool,
            "Token {} rgb({},{},{}) not in any defined range",
            i,
            r,
            g,
            b
        );
    }

    Ok(())
}

#[test]
fn test_rgb_styled_empty_ranges_rejected() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let admin = mock.addr_make("admin");

    let svg = Cw721Svg::new(mock.clone());
    svg.upload()?;

    let template = "<svg><rect fill='${c}'/></svg>";
    let variables = vec![VariableDef {
        name: "c".to_string(),
        kind: VariableKind::RgbStyled(vec![]),
    }];

    let init_msg = InstantiateMsg {
        name: "Empty Ranges".to_string(),
        symbol: "EMPTY".to_string(),
        svg_template: template.to_string(),
        variables: variables.clone(),
        total: 10,
        seed: test_seed(),
        owner: Some(admin.to_string()),
        mint_start_time: None,
        mint_end_time: None,
        price_tiers: Vec::new(),
        payment_address: Some(admin.to_string()),
        whitelist: None,
        template_slots: compute_slots(template, &variables),
    };

    svg.instantiate(&init_msg, Some(&admin), &[])
        .expect_err("should fail: empty ranges list");

    Ok(())
}

#[test]
fn test_rgb_styled_min_gt_max_rejected() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let admin = mock.addr_make("admin");

    let svg = Cw721Svg::new(mock.clone());
    svg.upload()?;

    let template = "<svg><rect fill='${c}'/></svg>";
    let variables = vec![VariableDef {
        name: "c".to_string(),
        kind: VariableKind::RgbStyled(vec![RgbRange {
            r_min: 255,
            r_max: 100, // min > max
            g_min: 0,
            g_max: 255,
            b_min: 0,
            b_max: 255,
        }]),
    }];

    let init_msg = InstantiateMsg {
        name: "Bad Range".to_string(),
        symbol: "BAD".to_string(),
        svg_template: template.to_string(),
        variables: variables.clone(),
        total: 10,
        seed: test_seed(),
        owner: Some(admin.to_string()),
        mint_start_time: None,
        mint_end_time: None,
        price_tiers: Vec::new(),
        payment_address: Some(admin.to_string()),
        whitelist: None,
        template_slots: compute_slots(template, &variables),
    };

    svg.instantiate(&init_msg, Some(&admin), &[])
        .expect_err("should fail: r_min > r_max");

    Ok(())
}
