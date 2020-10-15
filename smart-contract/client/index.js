const yargs = require('yargs')
const { exit } = require('yargs')
const { createContract, getContract, payContract, acceptContract, claimContractFunds, cancelContract } = require('./src/contracts')

const argv = yargs
  .command('create', 'Create a volume contract', {
    nodeID: {
      description: 'ID of the node to deploy on',
      alias: 'n',
      type: 'string'
    },
    type: {
      description: 'Volume disk type (1 for HDD, 2 for SSD)',
      alias: 't',
      type: 'number'
    },
    size: {
      description: 'Volume size in GB',
      alias: 's',
      type: 'number'
    }
  })
  .command('get', 'Get a contract by ID', {
    contractID: {
      description: 'Contract ID',
      alias: 'id',
      type: 'string'
    }
  })
  .command('pay', 'Pay for a contract by ID', {
    contractID: {
      description: 'Contract ID',
      alias: 'id',
      type: 'string'
    },
    amount: {
      description: 'Amount to pay',
      alias: 'a',
      type: 'number'
    }
  })
  .command('accept', 'Accept a contract by ID', {
    contractID: {
      description: 'Contract ID',
      alias: 'id',
      type: 'string'
    },
    mnemonic: {
      description: 'Mnemonic to sign with',
      alias: 'm',
      type: 'string'
    }
  })
  .command('claim', 'Claim funds off a contract by ID', {
    contractID: {
      description: 'Contract ID',
      alias: 'id',
      type: 'string'
    },
    mnemonic: {
      description: 'Mnemonic to sign with',
      alias: 'm',
      type: 'string'
    }
  })
  .command('cancel', 'Cancel a contract by ID', {
    contractID: {
      description: 'Contract ID',
      alias: 'id',
      type: 'string'
    }
  })
  .help()
  .alias('help', 'h')
  .argv

if (argv._.includes('create')) {
  if (!argv.n || !argv.t || argv.s) {
    console.log('Bad Params')
    exit(1)
  }

  createContract(argv.n, argv.t, argv.s, ({ events = [], status }) => {
    console.log(`Current status is ${status.type}`)

    if (status.isFinalized) {
      console.log(`Transaction included at blockHash ${status.asFinalized}`)

      // Loop through Vec<EventRecord> to display all events
      events.forEach(({ phase, event: { data, method, section } }) => {
        console.log(`\t' ${phase}: ${section}.${method}:: ${data}`)
      })
      exit(1)
    }
  }).catch(err => {
    console.log(err)
    exit(1)
  })
}
if (argv._.includes('get')) {
  if (!argv.id) {
    console.log('Bad Params')
    exit(1)
  }

  getContract(argv.id)
    .then(contract => {
      console.log('\ncontract: ')
      console.log(contract)
      exit(0)
    })
    .catch(err => {
      console.log(err)
      exit(1)
    })
}
if (argv._.includes('pay')) {
  if (!argv.id || !argv.a) {
    console.log('Bad Params')
    exit(1)
  }

  payContract(argv.id, argv.a.toString(), ({ events = [], status }) => {
    console.log(`Current status is ${status.type}`)

    if (status.isFinalized) {
      console.log(`Transaction included at blockHash ${status.asFinalized}`)

      // Loop through Vec<EventRecord> to display all events
      events.forEach(({ phase, event: { data, method, section } }) => {
        console.log(`\t' ${phase}: ${section}.${method}:: ${data}`)
      })
      exit(1)
    }
  }).catch(err => {
    console.log(err)
    exit(1)
  })
}
if (argv._.includes('accept')) {
  if (argv.id === '' || !argv.m) {
    console.log('Bad Params')
    exit(1)
  }

  acceptContract(argv.id, argv.m, ({ events = [], status }) => {
    console.log(`Current status is ${status.type}`)

    if (status.isFinalized) {
      console.log(`Transaction included at blockHash ${status.asFinalized}`)

      // Loop through Vec<EventRecord> to display all events
      events.forEach(({ phase, event: { data, method, section } }) => {
        console.log(`\t' ${phase}: ${section}.${method}:: ${data}`)
      })
      exit(1)
    }
  }).catch(err => {
    console.log(err)
    exit(1)
  })
}
if (argv._.includes('claim')) {
  if (argv.id === '' || !argv.m) {
    console.log('Bad Params')
    exit(1)
  }

  claimContractFunds(argv.id, argv.m, ({ events = [], status }) => {
    console.log(`Current status is ${status.type}`)

    if (status.isFinalized) {
      console.log(`Transaction included at blockHash ${status.asFinalized}`)

      // Loop through Vec<EventRecord> to display all events
      events.forEach(({ phase, event: { data, method, section } }) => {
        console.log(`\t' ${phase}: ${section}.${method}:: ${data}`)
      })
      exit(1)
    }
  }).catch(err => {
    console.log(err)
    exit(1)
  })
}
if (argv._.includes('cancel')) {
  if (!argv.id) {
    console.log('Bad Params')
    exit(1)
  }

  cancelContract(argv.id, ({ events = [], status }) => {
    console.log(`Current status is ${status.type}`)

    if (status.isFinalized) {
      console.log(`Transaction included at blockHash ${status.asFinalized}`)

      // Loop through Vec<EventRecord> to display all events
      events.forEach(({ phase, event: { data, method, section } }) => {
        console.log(`\t' ${phase}: ${section}.${method}:: ${data}`)
      })
      exit(1)
    }
  }).catch(err => {
    console.log(err)
    exit(1)
  })
}
