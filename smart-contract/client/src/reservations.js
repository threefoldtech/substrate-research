const { getApiClient } = require('./api')
const { Keyring } = require('@polkadot/api')

async function createReservation (nodeID, diskType, size) {
  const api = await getApiClient()
  const keyring = new Keyring({ type: 'sr25519' })
  const BOB = keyring.addFromUri('//Bob', { name: 'Bob default' })

  const volume = {
    disk_type: diskType,
    size
  }

  return api.tx.templateModule
    .createContract(nodeID, volume)
    .signAndSend(BOB)
}

async function getReservation (id) {
  const api = await getApiClient()
  const contract = await api.query.templateModule.contracts(id)

  // Retrieve the account balance via the system module
  const { data: balance } = await api.query.system.account(contract.account_id)

  const json = contract.toJSON()
  json.node_id = hexToAscii(contract.node_id).trim().replace(/\0/g, '')

  return {
    ...json,
    balance: balance.free.toNumber()
  }
}

async function payReservation (id, amount) {
  const api = await getApiClient()
  const keyring = new Keyring({ type: 'sr25519' })
  const BOB = keyring.addFromUri('//Bob', { name: 'Bob default' })

  const balance = api.createType('BalanceOf', amount * 1000000000000)

  return api.tx.templateModule
    .pay(id, balance)
    .signAndSend(BOB)
}

function hexToAscii (str1) {
  const hex = str1.toString()
  let str = ''
  for (let n = 0; n < hex.length; n += 2) {
    str += String.fromCharCode(parseInt(hex.substr(n, 2), 16))
  }
  return str
}

module.exports = {
  createReservation,
  getReservation,
  payReservation
}
