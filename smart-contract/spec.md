# Smart contract 4 IT.

## Overview

The current smart contract 4 IT on the Threefold Grid is centralized. It is owned by the *explorer* and all it's data sits in a MongoDB. We want to decentralize the way a User and a Farmer aggree on what needs to be deployed on their nodes.

## Current architecture

The TFexplorer is responsible for deploying, decomissioning, refunding, .. of workloads. It is the intermediate party between the User and the Farmer on the Threefold Grid. For more details on the workings of the current model: https://manual2.threefold.io/#/smartcontract_details

The issue with this architecture is that there is a single point of failure. In this case it can be the machine where this explorer is running on. 

We, as a company, also promote decentralization in any way possible. The very way we run this critical component centralized is contradictionary.

## Proposed architecture

We need to change how the intermediate party is handled in order to achieve true decentralization. This component needs to act as a trusted party which nobody has control over, not even us. The best way to achieve this is with some sort of blockchain technology. A blockchain Where we can have the smart contract 4 IT on it. If it's on the blockchain, everybody can verify that it exists and that it cannot be tampered with. Eventually this party handles money, which is very sensitive.

The blockchain can be used to track the following information which is crucial in the deployment of workloads:

- Make an aggreement between a farmer and a user for a workload.
- Manage funds of this aggreement.
- Manage the state of a workload.
- ..

We currently use Stellar blockchain for our Token economy. Stellar does not support smart contracts in a way that we would need them to decentralize the explorer.

## Proposed technology

As we need a blockchain to be this intermediate party to achieve decentralization, we will be looking at [Substrate](https://www.parity.io/substrate/). This is a new blockchain technology which is written in Rust and is modular by design. This means we can plug in additional features which fits our needs. This can be done either by:

- Runtime pallets
- Smart contracts
- Offchain workers

These features will enable us to create a blockchain that has a sole purpose of decentralizing the User <-> Farmer relationship on the Threefold grid.

If we run a Substrate node that has the smart contract 4 IT capability, then any farmer / user can connect to this node and start using it. If they want to extend trust to this blockchain they can run a node themself, and connect to that one. If more and more farmers also run a node, then we create a decentralized network of trust for the smart contract 4 IT to live on.

We could for example run a substrate node inside ZOS on any farmer node.

## Caveats

Currently we have chosen to implement our Token economy on the Stellar blockchain. To make this compatible with the Substrate chain we need to bridge payments. This means we need some service that for example, when a user pays for a workload in Stellar TFT, we bridge this payment and fund the contract on chain with Substrate TFT.

## Proof of concept

In the first phase we would make a proof of concept that handles the following parts:

- Create an aggreement to deploy a Volume reservation on a specific node.
- Handle funds of this aggreement (Payments will not include Stellar TFT).
- Deploy Volume reservation on a ZOS node.
- Cancel reservation
- ..

If we can manage to get this working for this specific reservation type, we most likely can handle any other type.