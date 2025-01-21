import codegen from '@cosmwasm/ts-codegen';

codegen({
  contracts: [
    {
      name: 'CwInfuser',
      dir: '../../contracts/cw-infuser/schema'
    },
  ],
  outPath: './src/',

  // options are completely optional ;)
  options: {
    bundle: {
      bundleFile: 'bundle.ts',
      scope: 'contracts'
    },
    types: {
      enabled: true
    },
    client: {
      enabled: true
    },
    reactQuery: {
      enabled: true,
      optionalClient: true,
      version: 'v4',
      mutations: false,
      queryKeys: true,
      queryFactory: true,
    },
    recoil: {
      enabled: false
    },
    messageComposer: {
        enabled: true
    },
  }
}).then(() => {
  console.log('✨ all done!');
});