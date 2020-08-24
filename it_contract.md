# IT contract on substrate

Overview of a possible reimplementation of the IT contract, in order to solve
some of the current issues.

## Current situation

Right now threefold runs a single `Explorer` for the threefold grid. This explorer
is actually a unification of 3 components:

- Directory
- Phonebook
- Workloads (i.e. instances of the IT contract)

Furthermore, there are a few additional components which have been added, to facilitate
operation of the aforementioned. These are the `Escrow` and `Capacity pool`. Both
of these were designed and developed for operation with the `Workloads`. In essence
these modules handle payments for workloads, such that we, as third party running
the explorer, minimize the amount of time the funds required for a reservation
are with us, while locking them during the contract negotiation phase (the time
between creating the contract definition and actual deployment), and making
sure that cancelling a workload prematurely does not result in loss of funds (as
clients make a full payment up front). The notion of the capacity pool also allows
reservation extension, as a reservation for a workload lives as long as the pool
has enough capacity to support it.

In order to have some notion of blockchain, all reservations are done through an
immutable reservation object. This object carries further mutable state, which
is used by the explorer. In theory, by reiterating over all reservations, we
could reconstruct the state of any workload at any time.

## Flow of the workload/smart contract reservation and deployment

If we take a look at the workloads package, and what it actually does, we notice
the following flow:

- A user creates a capacity pool registration and sends it to the explorer.
- The explorer processes the reservation request, and if it succeeds, it sets up
an escrow for the user, and returns a payment details object to the user.
- The user pays the requested amount to the escrow account
- The explorer verifies that the escrow account holds enough funds. Once this is
indeed the case, the capacity pool is filled with the requested capacity, and
the farmer is paid for his capacity. The price for the capacity was taken when
the original capacity reservation was made. The threefold foundation takes a small
cut of the farmer payout (10%).
- With his capacity pool created and filled, the user now creates a workload reservation
linked to the pool (the original smart contract for IT), and sends it to the explorer.
- The explorer verifies that the linked pool has enough capacity available to at
least sustain the workload for a short while, and if it does, it puts the workload
in a state such that it can be discovered by the node in question.
- The node picks up the workload definition and attempts to deploy it, ultimately
sending back the result to the explorer.
- The explorer receives the result. Based on the success of the deployment, the
capacity pool starts to decrease at the determined rate.
- If the user does not want his reservation to expire, he continues to refill the
pool.

There are some interesting observations we can make here. First of all, notice
that there never is any (direct) involvement from the farmer in this flow. He sets
his price, starts his nodes with his farm id, and thats it. In practice, the explorer
will also take on the role of the farmer threebot, by implicitly authorizing any
reservation made. This means a farmer has no control over which workloads end up
on his node currently.

Secondly, the payment escrow. This was originally introduced to have the ability
to refund a customer should a deployment go wrong. Indeed, before the introduction
of the capacity pool, a smart contract itself would be paid, and if deployment
failed, the user would receive a full refund. The problem here was simple:
if initial deployment succeeds but the workload dies after 1 minute, the
farmer would still receive all funds. This led to the introduction of a capacity
pool. Even though the farmer still receives all funds upfront, the explorer decrements
the pool capacity in real time, meaning if a workload dies half way through, a user
can still reserve an identical workload (perhaps on a different node) with the same
pool, and the explorer would deploy it for the remaining time. So no funds are lost
from the users perspective. Therefore we can think of the capacity pool as a concept
of incremental payment of sorts. But the capacity pool is not without its flaws either.
We also see that the escrow takes a small cut of the farmer proceeds, and sends
this to the foundation. This is only possible due to the centralized nature of the
escrow. If every farmer is running his own threebot, and there is no centralized
explorer, it would be trivial for a farmer to remove this code section.

Thirdly, monitoring is not inherently part of this flow. This is not a problem.
After all, both a customer and the node will have different ideas of what a healthy
workload is. For instance, a node will be happy if the container it spawned is running,
whereas the customer will want his application to be actually running inside that
container, processing data, doing whatever. It is up to the customer to actually
define what is healthy. Thanks to the capacity pool, there is no cost for a customer
to cancel his reservation (he can still deploy something else without needing to
pay extra). Therefore we can place the burden of monitoring the workload on the
customer. If for whatever reason he decides the workload is not healthy (e.g. node
went down, or the application crashed due to the customers fault), he can just
cancel the workload and start over. Do keep in mind that the capacity pool is a
bit flawed here as said previously. Most notably, if the farmer shuts down all
his nodes and quits, he will have all money for the still active capacity pools,
and the customer(s) will have no way to recover their funds.

Fourth, there is no state history. Currently all workloads are stored in their
current state in a mongodb cluster. For an active workload, figuring out why
a workload ended up in a given state mostly implies checking the data on the workload
state and identifying what was added last. Although there is the idea of moving
this to bcdb, which would give state history, this means we have a history of the
active state of the workload, not of the actual state transition and its triggers.

### Abstract requirements of the workloads/ smart contract

In the previous section, we explored the concrete flow for the workload reservation
as it currently is implemented. After some analysis, it was seen that there were
some problems with this current flow. We will now take a higher level look at what
actually happens for the IT contract, and what concepts we actually need.

- A user creates the IT contract. It must be **stored** somewhere, and it must
be **immutable**.
- The farmer **notices** the IT contract, reviews it, and **accepts** or **rejects**
it. Should he accept, the **price for the workload over some period of time** also
needs to be locked in
- The user makes a payment for the reservation. Ideally, this payment behaves as
follows:
  - There is a **reasonable deposit**, such that the customer **proves ownership
  of sufficient funds**, and simultaneously **locks required funds** for the workload.
  - For a successfully deployed workload, the **farmer can receive or claim periodic
  funds**, in accordance to the amount of time past since the workload was deployed.
  - In case of an error or deletion of the workload, a **refund** is given for the
  locked funds, after once again settling the open due amount for the farmer.
  - It is possible to **make further deposits**, which prevents the funds from
  being exhausted, thus increasing the duration of the reservation.
- After an initial payment, the node can pick up the reservation, and **start to
deploy** the workload.
- Deployment result is **communicated to the IT contract**.
- If the **deployment failed** (or timed out), the customer **is refunded**
- If the **deployment succeeds**, the farmer receives periodic payments for the
workload.
- If the **locked funds run out**, or the customer (and maybe the farmer?) decide
to cancel the workload, the **farmer balance is settled** and the **remaining funds
are refunded to the user**

## Substrate smart contracts and pallets

[Substrate](https://substrate.io) is a rust based blockchain framework. On of
their features, is the ability to have smart contracts on the chain. Smart contracts
can do a lot of things: they can define and manage a piece of storage, expose
functions callable by the outside world, manage a balance, ...

On substrate, a smart contract is a wasm binary. The entire binary is uploaded,
along with a piece of metadata exposing which functions can be called. One option
would be to implement the smart contract as IT as a smart contract. This too has
some downsides. For instance, every user would be free to implement the contract
how they see fit, meaning that a farmer would probably want to review the code
manually. This is less than ideal. Furthermore the entire binary is saved while
the contract is alive, which adds quite a bit of overhead, since the bytecode
for the logic itself is thus stored multiple times. Although there is a cache which
leads to multiple copies of the same contract being stored only once, developers
taking some freedom and changing the implementation a bit might lead to storage
bloat.

Fortunately, there is a solution here in the form of the `pallet`. A pallet is
essentially part of the runtime and created by the blockchain author, in contrast
to the smart contract, which can be created by any user of the chain. This allows
for some optimizations, at the cost of losing some protection, such as the sandboxed
execution environment. This is fine however, since as chain authors, we can reasonably
be assumed to not be malicious, and carefully design the pallet to not have a
bad performance impact. Pallets can also be interacted with through rpc calls,
can communicate with other pallets, manage storage, and more. By implementing a
pallet, we also unify the way the IT contract works, and prevent third parties
from designing a malicious contract.

### Smart contract for IT on substrate

To implement the smart contract for IT on a substrate based chain, we would have
an implementation as follows:

- A rust implementation of the IT contract primitives, such as container,
network, ...
- Create an `IT contract pallet`, which will expose a set of rpc functions,
and manage some storage, so that we can save active IT contracts.
- Whenever an entity creates a new IT contract, a new identifier is created and
returned, which identifies the contract.
- Once a contract is initially created, the identifier is generated and returned,
and a **ContractCreated** event is emitted, container some identification of
the farmer. This way, the farmer knows a new contract for one of its nodes is
on chain that needs his review.
- The farmer either declines the contract, causing the pallet to emit a **ContractRejected**
event, signaling the end of this contract, and cleaning up its storage. Or the contract
is accepted, causing a **ContractAccepted** event to be generated, signaling the
original author that he can proceed to paying. As part of accepting the contract,
the farmer could also set a price, if this was not yet negotiated otherwise (t.b.d).
- The original author gets funding for the contract, which in turn causes a **ContractFunded**
event to be emitted. This event could also include the node ID, which in turn allows
the node itself to pick up the event and start deployment.
- When deploying is done, the result is pushed to the chain. The chain can verify
the signature based on the node ID which is just the public key. If the result is
OK, a **ContractDeployed** event is emitted, signaling the original author that the
workload is live, and the farmer can start to claim periodic payments from the
deposited funds. At the same time, the pallet can calculate when the contract would
expire based on the available funds (need to investigate if this can be done, and
if it can be done efficiently). If the result is an error, a **ContractDeployError**
is emitted, notifying the author that his workload has not been deployed (and he
should try again, possibly on another node). The funds which have been deposited
already are returned back to the author, and the contract storage is cleaned up.
- If the contract is deleted for any reason, a **ContractDeleted** event is emitted,
remaining due balance for the farmer is paid, and remaining balance is refunded
to the user.

Again we can make some observations. The most important one is that we clean up
all storage when a workload is no longer deployed. This is fine, since every
mutation of the workload state is done through an rpc call, an extrinsic, which
is included in a block. Therefore the state of an IT contract at any time can be
recovered by walking the blocks in the blockchain. The events also help here, as
these are stored as well. Therefore an explorer or otherwise archive node will be
able to fully show what happened over a contract lifetime.

What this brings us is the following. First, we can remove the notion of the
capacity pool again, which was a workaround in the first place. Secondly, the
escrow is no longer needed, since the chain will take that responsibility. Because
we control the full logic of the chain, we can also add the foundation payout
in the chain code, meaning decentralizing the explorer no longer risks farmers
removing the foundation payout logic. Thirdly, there is no longer a need for
an explorer workloads package, since that is also handled completely on chain now.
In fact, this idea could be extended to have node and farm registrations on the
chain itself as well, which would completely remove the need for an explorer,
and allow visualization as we have now to be done by any archive node.

Of course, the above does mean that there will likely need to be some notion of
a threebot, for the farmer mostly, and perhaps for the user to manage healthchecks,
and remove reservations if these healthchecks fail.

Some research will need to be done:

- Can there be a delayed execution (i.e. timer) for when the funds in a contract
are empty? Else, a special call can be created where a farmer can delete the contract
if there are no funds left (or maybe the farmer can cancel at any time?).
- Rpc calls (extrinsics) inclusion in block, and their cost ( in tokens, if needed).
- Space requirements of regular nodes
- Availability of light clients
- GUI?
- ...  
