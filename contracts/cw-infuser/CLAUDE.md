# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
# Build the contract in debug mode
cargo build

# Build the contract in release mode with optimization
cargo run-script optimize

# Build the contract for ARM64 architecture
cargo run-script optimize-arm

# Run tests
cargo test

# Generate schema
cargo schema
```

## Project Overview

CW-Infuser is a CosmWasm smart contract that enables the creation of "infused" NFT collections by burning existing NFTs. The project allows users to define various recipes or "bundle types" that determine how NFTs can be combined or "infused" to create new NFTs.

### Key Features

1. **Infusion Creation**: Users can create infusion recipes that define which NFT collections are eligible for infusion and what the resulting NFT collection will be.
2. **Bundle Types**: Support for different bundle types:
   - `AllOf`: Requires the minimum number of NFTs from all eligible collections
   - `AnyOf`: Accepts any NFT from a specified list of collections
   - `AnyOfBlend`: Allows combinations of different NFTs (still under implementation)
3. **Fee Management**: Supports creation fees and mint fees with configurable fee splitting between contract owner and infusion creator
4. **Payment Substitutes**: Option to substitute NFTs with token payments

## Core Components

### Configuration

- `Config`: Global contract configuration (max infusions, fees, code_id for new collections)
- `InstantiateMsg`: Parameters for contract initialization

### Infusion Mechanism

- `InfusionState`: Current state of an infusion including eligible collections and infusion parameters
- `BundleType`: Defines the rules for how NFTs can be infused (AllOf, AnyOf, AnyOfBlend)
- `NFTCollection`: Represents an eligible NFT collection with min/max requirements
- `Bundle`: Group of NFTs submitted by a user for infusion
- `InfusedCollection`: Parameters for the new NFT collection that will be created

### State Management

The contract uses various storage maps to track:
- Infusion mappings and IDs
- Token position mappings
- Mintable tokens
- Mint counts

## Common Operations

### Creating an Infusion

An infusion defines which NFT collections can be burned to create new NFTs. The owner can set:
- Eligible collections and their requirements
- The new NFT collection's parameters (name, symbol, base URI)
- Mint fees and payment configurations
- Bundle type rules

### Performing an Infusion (Burning NFTs)

Users can submit NFTs in bundles according to the infusion rules. The contract will:
1. Verify ownership of the NFTs
2. Check if the bundle satisfies the infusion requirements
3. Burn the submitted NFTs
4. Mint new NFTs to the sender from the infused collection
5. Handle any fees according to the configuration

### Managing Infusions

Infusion owners can:
- Update base URI of the infused collection
- Update eligible collections (add/remove)
- Update mint fees
- Change bundle type
- End an infusion

## Migrations

The contract supports migrations between versions with version-specific migration logic in the `upgrades` module. The current contract version is 0.4.2.

## Stargaze Integration

The contract includes optional Stargaze (SG) integration through feature flags:
- `sg`: Enables Stargaze-specific functionality
- Default feature is `sg`