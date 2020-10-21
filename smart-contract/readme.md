# Smart Contract 4IT on Substrate

Spec can be found [here](./spec.md).

## Installation

```
cd substrate-node
cargo build --release
```

This will build the node binary in release mode, once built you can execute it by doing following:

`./target/release/node-template --dev --tmp --ws-external`

> You need the `ws-external` flag in order to connect from a zos node to substrate in a local setup.

Now you can build the client to interact with this node:

You need Yarn in order to continue.

```
cd client
yarn install
```

> The client uses the keys of **Bob** the sign for transactions in all the examples. The user **Bob** is a dummy user created when the chain starts.

If you want to run this example on a zos node: [get zos ready](./zos.md)

## Creating a contract for a Volume reservation

Parameters:

* **-n**: ID of a node to deploy the reservation on.
* **-t**: Disktype, (1 for ssd, 2 for hdd).
* **-s**: Size of the volume in Gigabyte.

`node index.js create -n 2gKiAZgeA8C1HsvSYMfdnZYPWNm51xMdYRBNnZxAthWr -t 1 -s 10`

## Fetching the contract's details.

Contract ID's are incremented sequentially. If you create your first contract the ID will be 0, the second will be 1, etc...

`node index.js get --id 0`

## Funding a contract.

Contract's can be funded with an arbitrary amount of tokens. Only if a contract is funded a reservation can be deployed.

Parameters:

* **-a**: Amount to fund the contract with.

`node index.js pay --id 0 --a 5000`

## Cancelling a contract.

Will decomission the workload on zos and refund the user.

`node index.js cancel --id 0`

## Accepting a contract

Contract's can be accepted by the farmer, this will set the boolean `accepted` to true, indicating that the contract's prices are aggreed, the contract is funded and the workload is ready to deploy.

Parameters:

* **-m**: Mnemonic of the farmer.

`node index.js accept --id 0 --m seedwords`

## Claiming funds of a contract

Contract funds can be claimed only by the farmer party of the contract. To claim funds:

Parameters:

* **-m**: Mnemonic of the farmer.

`node index.js claim --id 0 --m seedwords`