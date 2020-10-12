const { ApiPromise, WsProvider } = require('@polkadot/api')

async function getApiClient () {
  const wsProvider = new WsProvider('ws://localhost:9944')
  return ApiPromise.create({
    provider: wsProvider,
    types: {
      Contract: {
        cu_price: 'u64',
        su_price: 'u64',
        account_id: 'AccountId',
        node_id: 'Vec<u8>',
        farmer_pubkey: 'Vec<u8>'
      },
      VolumeType: {
        disk_type: 'u8',
        size: 'u64'
      },
      WorkloadState: {
        _enum: ['Created', 'Deployed', 'Cancelled']
      },
      // override custom
      Address: 'AccountId',
      LookupSource: 'AccountId',
      BalanceOf: 'Balance'
    }
  })
}

module.exports = { getApiClient }
