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
             \       610      /
              \               /
               \    _____    /
                |--//''`\--|
                | (( +==)) |
                |--\_|_//--|
```

## Introduction
Cw-Infusion is a nft minter for burning to mint tokens in very programmable ways. Bundles, or the collection of tokens one intends to infuse,are formed and burned in exchange for a new token from the infused collection.
 
<!-- ### State.json
The deployed contracts state can be found [here](./state.json). This file is generated from making use of cw-orchestrator scripts.  -->


## Scripts 
There are cw-orchestrator libraries available to deploy the infusion contracts. First, ensure your environment variables are set, such as mnemnoics state file locations, artifacts directories, transaction options, and logging. Full environment envariable details can be found here: https://orchestrator.abstract.money/contracts/env-variable.html


## Tests
```sh
 # for unit tests
 cargo test
# for integration tests
 cargo test --package test-suite --lib -- infuser::test::init --show-output 
```

