# POC 1 - smart contract on chain

The goal of this POC was to create an implementation of a pallet which provided
a minimal deployment flow for a workload. The flow is a subset of the current
deployment flow on the explorer. We only implemented reservation of a volume, as
proving that we can deploy a single workload is sufficient. Multiple workloads can
easily be added. Furthermore, multisignature support was not added, as the current
implementation of that is trivially copied if a single signature works.

All in all, the aim was to comission, and then decomission, a volume on a node.
At the same time, money should be transfered to the farmer through the contract.
In essence, the contract acts as the escrow. Since the contract is managed by
logic on chain, this means the escrow is essentially the chain itself, without
any intervention. Also, since there is no more notion of a capacity pool, we can
now do a proper refund to a client.

## Result

With the POC finalized, all of the above aims have been achieved. The full flow
is as follows: first a customer creates a contract, i.e. a workload definition.
This is then saved on chain. When the contract is included in a block, an account
is created which is managed by this contract, and an off-chain worker goes to
the explorer and fetches the farmers price, and his public key. The price is added
to the contract, the public key is converted into an account ID. Next, the farmer
can approve the contract. Only the account matching the account ID just fetched
can perform this call successfully. Once this is done, the client can pay for the
contract. This transfers funds to the contract account. It also generates an event
which notifies listeners that the contract was funded. A node can listen for these,
check that the nodeID is its own, and start deployment. Since the nodeID used to
to deploy on is known, and the nodeID is just a base58 encoded ed25519 public key,
the public key of the node can be decoded, and converted  into an account ID. After
deployment, the node then calls a method specifying successful deployment. Validation
that this is the proper node based on the account ID the call is performed with
is done, like it is done with the farmer. Now that the system knows when the workload
is deployed, the time to live can be calculated based on the price of resources
which was set prior, and the amount of funds in the contract. This is important
so the system can know when to generate the expiration even in an efficient way
(assuming no further payments are done). Should there be further payments, the
expiration can be recomputed based on the added funds. Periodically, the farmer
can make calls to the chain to claim the balance they earned from the contract,
should they so desire. At any point, both the farmer and client can cancel a contract.
In this case, the farmer is automatically paid the owed balance, and all remaining
balance is refunded to the client. If the contract expires due to no more funds,
everything is transfered to the farmer. Once a contract cancelled/expired event
is generated, the node decomissions the workload.

## Observations

It seems to not be possible to request a history of events on a regular node (non
archive). This means that other measures must be implemented to make sure a node
does not miss important events. We already have a storage map where the nodeID
is used as a key, and the value is a list of currently active workload ID's. These
can trivially be queried by a node to make sure all its workloads come back online
after a reboot. More elaborate schemes can also be implemented. This should not
be an issue, since all state transitions are explicit in our code, either due to
calls made from outside of the chain which are handled, or through custom runtime
code.

There are seemingly no `String`s in the runtime. That is, there is no support
for dynamically sized Strings (there are regular `str`s though). This is not really
a problem if we understand that a `String` is really just a `Vec<u8>`, where all
data happens to be valid `utf-8`. It was found to be slightly inconvenient in the
off-chain worker used to fetch data from the explorer, since creating the correct
url to call requires concatenation of 2 byte vectors. This is otherwise unimportant.

We still rely on the explorer for pricing and farmer authentication (key retrieval).
This was specified in the spec of the POC, and can be resolved with the implementation
of DID's. In this case, a farm can have a custom DID where the price is set, and
nodes can be identified by DID's as well, linking to the right farm. This would
remove all dependencies on the explorer and fully decentralize this solution.

In this setup, the node itself performs the call back to the chain. While we think
this is an ideal scenario as there is nothing in between the zos node and the
blockchain node, it also means the node needs access to a funded account, and the
ability to make transactions. A funded account requires an account ID, and funds
to be transfered to it. The account ID we use is derived from the existing keypair
on the node. After a node boots for the first time, the farmer must transfer
a small amount of tokens to the node so that it can pay the transaction fees. This
does mean that if the node keypair is lost, all funds belonging to the node are
lost as well. Considering the low amount required to do calls, and the low amount
of calls needed to be made by the node, the size of this deposit should not be
an issue. To create transactions, the node requires the `subkey` tool to be available,
as the go client used is not able to sign transactions by itself, and offloads this
to this tool. That is not really an issue, an flist with a static build of the
tool can easily be created.

As mentioned, the node uses an ed25519 keypair. While substrate supports this,
the go client only knows how to work with sr25519 tools. To alleviate this, we
forked the client and modified it to work on ed25519 keys instead. If we continue
with this project, we should probably create a pull request to allow the client
to work with both types of keys.

Derivation of ed25519 keys from a bip-39 seed is not the same as we are familiar
with. We just decode the mnemonic, and use the 32 bytes as the private key, from
which we then derive the public key. Substrate, and more specifically the `subkey`
tool (and all conforming libraries) instead run the 32 bytes through PBKDF2, with
a SHA512 hasher. They then take the first 32 bytes of the produces 64 byte entropy
as the private key from which the public key is derived. This is merely inconvenient,
as it means we can't directly feed the seed into conforming tools, but rather we
have to derive the keypair first using our method, and can then use conforming tools
with the derived private key.

Substrate understands both ed25519 and sr25519 keys, but prefers to use sr25519
keys. These are supposedly faster for batch verification of signatures, and much
better suited for HDKD (Hard Deterministic Key Derivation). It should be noted
that from the same seed, both keypairs can be genrated, resulting in different
addresses on chain. This is something to keep in mind. Secp256k1 is also fully
supported. If we map existing keys to attributes on chain, we should push more
towards using ed25519 keys.

Currently automatic expiration detection is implemented by setting the contract
ID as a value in the database, to a key which is a unix timestamp. After every
block a sweep is done for every second from the previous block to the current block,
and all contracts under those keys are expired. As per the docs, the time spent
in this `on_finalize` function is limited. That being said, the amount of contracts
expiring and the speed of the block creationg should mean this scales pretty well.

Fund management for the contracts is currently done through the balances pallet.
This makes it easy, since we can just generate a semirandom account, and offload
the handling of transfers and funds to this pallet. This does raise an issue, since
the account managed by the contract is in essence a regular account. In other words,
payments can be done to this account as if it was a regular user, and the contract
expiration will not be updated as a result. To alleviate this, when a contract
goes out of funds, we should first fetch the balance again, and verify that it has
indeed expired (based on the last time the farmer withdrew funds, the amount of
available funds, and the price). If we detect that a payment was done to the account,
we can recalculate the real expiration, and try again at this time.

Currently a very small balance of 500 is set on a contract when its created. This
is the minimal deposit for an account in order to not be destroyed to free up space
in the storage. Without this, the account will not exist untill a payment is made
to it. We should investigate if this is required. It will be inconvenient for tools,
since they need to handle the case where the account does not exist (but the contract
does), though it should otherwise be possible. If it does prove to be impossible,
we will need to manually manage this small amount when performing calculations,
and destroy the account manually once the contract is finsihed.
