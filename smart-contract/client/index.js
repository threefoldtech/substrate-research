const yargs = require('yargs')
const { exit } = require('yargs')
const { createReservation, getReservation, payReservation } = require('./src/reservations')

const argv = yargs
  .command('create', 'Create a volume reservation', {
    nodeID: {
      description: 'ID of the node to deploy on',
      alias: 'n',
      type: 'string'
    },
    type: {
      description: 'Volume disk type',
      alias: 't',
      type: 'number'
    },
    size: {
      description: 'Volume size in GB',
      alias: 's',
      type: 'number'
    }
  })
  .command('getReservation', 'Get a reservation by ID', {
    reservationId: {
      description: 'Reservation ID',
      alias: 'id',
      type: 'string'
    }
  })
  .command('payReservation', 'Get a reservation by ID', {
    reservationId: {
      description: 'Reservation ID',
      alias: 'id',
      type: 'string'
    },
    amount: {
      description: 'Amount to pay',
      alias: 'a',
      type: 'number'
    }
  })
  .help()
  .alias('help', 'h')
  .argv

if (argv._.includes('create')) {
  if (argv.n === '' || argv.t === '' || argv.s === '') console.log('Bad params')

  createReservation(argv.n, argv.t, argv.s)
    .then(() => {
      console.log('success')
      exit(0)
    })
    .catch(err => {
      console.log(err)
      exit(1)
    })
}
if (argv._.includes('getReservation')) {
  if (argv.id === '') console.log('Bad params')

  getReservation(argv.id)
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
if (argv._.includes('payReservation')) {
  if (argv.id === '' || argv.a === 0) console.log('Bad params')

  payReservation(argv.id, argv.a.toString())
    .then(() => {
      console.log(`\nPayment for contract: ${argv.id}. Success!`)
      exit(0)
    })
    .catch(err => {
      console.log(err)
      exit(1)
    })
}
