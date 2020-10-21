const { ApiPromise, WsProvider } = require('@polkadot/api')

async function getApiClient () {
  const wsProvider = new WsProvider('ws://192.168.0.170:9944')
  return ApiPromise.create({
    provider: wsProvider,
    types: {
      Contract: {
        resource_prices: 'ResourcePrice',
        account_id: 'AccountId',
        node_id: 'Vec<u8>',
        farmer_account: 'AccountId',
        user_account: 'AccountId',
        accepted: 'bool',
        workload_state: 'WorkloadState',
        expires_at: 'u64',
        last_claimed: 'u64'
      },
      VolumeType: {
        disk_type: 'u8',
        size: 'u64'
      },
      // override custom
      Address: 'AccountId',
      LookupSource: 'AccountId',
      BalanceOf: 'Balance',
      Public: '[u8;32]',
      WorkloadState: {
        _enum: ['Created', 'Deployed', 'Cancelled']
      },
      RefCount: 'u32',
      ResourcePrice: {
        currency: 'u64',
        sru: 'u64',
        hru: 'u64',
        cru: 'u64',
        nru: 'u64',
        mru: 'u64'
      }
    }
  })
}

module.exports = { getApiClient }
