const { getApiClient } = require('./api')
const { Keyring } = require('@polkadot/api')
const bip39 = require('bip39')

async function createContract (nodeID, diskType, size, callback) {
  const api = await getApiClient()
  const keyring = new Keyring({ type: 'sr25519' })
  const BOB = keyring.addFromUri('//Bob', { name: 'Bob default' })

  const volume = {
    disk_type: diskType,
    size
  }

  return api.tx.templateModule
    .createContract(nodeID, volume)
    .signAndSend(BOB, callback)
}

async function getContract (id) {
  const api = await getApiClient()
  const contract = await api.query.templateModule.contracts(id)
  const volume = await api.query.templateModule.volumeReservations(id)

  // Retrieve the account balance via the system module
  const { data: balance } = await api.query.system.account(contract.account_id)

  const json = contract.toJSON()
  json.node_id = hexToAscii(contract.node_id).trim().replace(/\0/g, '')

  return {
    ...json,
    balance: balance.free.toNumber(),
    volume: volume.toJSON()
  }
}

async function payContract (id, amount, callback) {
  const api = await getApiClient()
  const keyring = new Keyring({ type: 'sr25519' })
  const BOB = keyring.addFromUri('//Bob', { name: 'Bob default' })

  return api.tx.templateModule
    .pay(id, amount)
    .signAndSend(BOB, callback)
}

async function acceptContract (id, mnemonic, callback) {
  const api = await getApiClient()

  const key = getPrivatekey(mnemonic)

  return api.tx.templateModule
    .acceptContract(id)
    .signAndSend(key, callback)
}

async function claimContractFunds (id, mnemonic, callback) {
  const api = await getApiClient()

  console.log(mnemonic)

  const key = getPrivatekey(mnemonic)

  return api.tx.templateModule
    .claimFunds(id)
    .signAndSend(key, callback)
}

async function cancelContract (id, callback) {
  const api = await getApiClient()
  const keyring = new Keyring({ type: 'sr25519' })
  const BOB = keyring.addFromUri('//Bob', { name: 'Bob default' })

  return api.tx.templateModule
    .cancelContract(id)
    .signAndSend(BOB, callback)
}

function hexToAscii (str1) {
  const hex = str1.toString()
  let str = ''
  for (let n = 0; n < hex.length; n += 2) {
    str += String.fromCharCode(parseInt(hex.substr(n, 2), 16))
  }
  return str
}

function getPrivatekey (mnemonic) {
  let entropy = bip39.mnemonicToEntropy(mnemonic)
  entropy = '0x' + entropy

  const keyring = new Keyring()
  return keyring.addFromUri(entropy)
}

module.exports = {
  createContract,
  getContract,
  payContract,
  acceptContract,
  claimContractFunds,
  cancelContract
}
