# Smart Contract 4IT on Substrate

Spec can be found [here](./spec.md).

## Installation

```
cd substrate-node
cargo build --release
```

This will build the node binary in relaese mode, once built you can execute it by doing following:

`./target/release/node-template --dev --tmp`

Now you can build the client to interact with this node:

You need Yarn in order to continue.

```
cd client
yarn install
```

> The client uses the keys of **Bob** the sign for transactions in all the examples. The user **Bob** is a dummy user created when the chain starts.

## Creating a contract for a Volume reservation

Parameters:

* **-n**: ID of a node to deploy the reservation on.
* **-t**: Disktype, (1 for ssd, 2 for hdd).
* **-s**: Size of the volume in Gigabyte.

`node index.js create -n 2gKiAZgeA8C1HsvSYMfdnZYPWNm51xMdYRBNnZxAthWr -t 1 -s 10`

## Fetching the contract's details.

Contract ID's are incremented sequentially. If you create your first contract the ID will be 0, the second will be 1, etc...

`node index.js getReservation --id 0`

## Funding a contract.

Contract's can be funded with an arbitrary amount of tokens. Only if a contract is funded a reservation can be deployed.

Parameters:

* **-a**: Amount to fund the contract with.

`node index.js payReservation --id 0 --a 5000`