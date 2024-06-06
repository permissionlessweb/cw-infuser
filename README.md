# Collection Infuser 

NFT minter for infusing (burning) nfts in creative ways.

## What Can I Decide? 
- which collections nfts can be used in a bundle
- how many tokens are required to infuse a bundle
- optional traits from each collection that are required to infuse a bundle

## Using the Justfile

This repository comes with a [`justfile`](https://github.com/casey/just), which is a handy task runner that helps with building, testing, and publishing your Abstract app module.

### Installing Tools

To fully make use of the `justfile`, you need to install a few tools first. You can do this by simply running `just install-tools`. See [tools used the template](https://docs.abstract.money/3_get_started/2_installation.html?#tools-used-in-the-template) for more information.

### Available Tasks

Here are some of the tasks available in the `justfile`:

- `install-tools`: Install all the tools needed to run the tasks.
- `wasm`: Optimize the contract.
- `test`: Run all tests.
- `fmt`: Format the codebase (including .toml).
- `lint`: Lint-check the codebase.
- `lintfix`: Fix linting errors automatically.
- `watch`: Watch the codebase and run `cargo check` on changes.
- `watch-test`: Watch the codebase and run tests on changes.
- `publish {{chain-id}}`: Publish the App to a network.
- `schema`: Generate the json schemas for the contract
<!-- - `ts-codegen`: Generate the typescript app code for the contract -->
<!-- - `ts-publish`: Publish the typescript app code to npm -->
- `publish-schemas`: Publish the schemas by creating a PR on the Abstract [schemas](https://github.com/AbstractSDK/schemas) repository.

You can see the full list of tasks available by running `just --list`.

### Compiling

You can compile your module(s) by running the following command:

```sh
just wasm
```

This should result in an artifacts directory being created in your project root. Inside you will find a `my_module.wasm` file that is your module’s binary.

### Testing

You can test the module using the different provided methods.

1. **Integration testing:** We provide an integration testing setup in both contracts. The App tests can be found here [here](./contracts/app/tests/integration.rs). You can re-use the setup provided in this file to test different execution and query entry-points of your module. Once you are satisfied with the results you can try publishing it to a real chain.
2. **Local Daemon (Optional):** Once you have confirmed that your module works as expected you can spin up a local node and deploy Abstract + your app onto the chain. You need [Docker](https://www.docker.com/) installed for this step. You can do this by running the [test-local](./contracts/app/examples/test-local.rs) example, which uses a locally running juno daemon to deploy to. You can setup local juno using `just juno-local` command. At this point you can also test your front-end with the contracts.

Once testing is done you can attempt an actual deployment on test and mainnet.
