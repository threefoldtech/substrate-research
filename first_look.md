# First look

After a first look at substrate and its ecosystem, the following results are shown:

- Version 2.0

Substrate is currently working heavily towards its v2.0 release. It has recently
tagged v2.0.0-rc6 on github. Between release candidates, there can be (quite some)
breaking changes. This should not be a problem, as we can easily specify exact
depency versions.

- Golang api client

It seems that parity do not maintain an api client in any language themselves.
There is a golang api, maintained by `centrifuge` (https://github.com/centrifuge/go-substrate-rpc-client).
Being developed by an external party, this api is however not strictly up to date
with the latest rc. In fact, the linked api is compatible only with rc3 at the time
of writing. An rc4 compatible client (forked from the centrifuge one) is available
as well and set to be merged into the upstream shortly. At the time of writing,
substrate rc6 has been released though, so there is still some work to catch up here.

The golang client also has no implementation for signatures. Instead, it offloads
this to the `subkey` tool (maintained in the substrate main repo). Since subkey
itself is also subject to breaking changes during the rc process, it must be
installed with the same version supported by the api.

- key management

Substrate chains can use 3 different signatures. EcDsa signatures, familiar ed25519
signatures (which we use already in a lot of applications), and the related sr25519
signatures. By default, the later are used. The key management tool, subkey, can
operate on the other types by passing a command line flag. Furthermore, HDKD
is supported, along key trees to be generated. Both hard and soft trees are supported
(though this probably isn't immediately relevant)

- nightly rust requirement

Due to the compilation stages of substrate, a nightly compiler toolchain is required
(next to the possible standard one). This does raise a small problem. The nightly
compiler upstream is updated daily, based on the merged PR's on the rust language
repository. It is possible for compilation to fail at some point. It seems the
location of the failure is always the same, though it is currently not known what
causes this. Downgrading the nightly toolchain to a known working one solves this
issue. The error, if it occurs, also does not indicate in any way that it is
compiler related. A way to solve this issue is to note the current version of
the nightly compiler before upgrading, reverting if the upgraded compiler should
fail to compile the code.

- modifying state requires payment

All calls to runtime modules must go through rpc functions, which have a weight.
Based on the weight, a runtime cost is computed which must be paid by the caller.
This gives economic protection for the chain, as to avoid spamming garbage. As
mentioned previously, any valid looking call is included in the chain, regardless
of whether or not the call is actually successful. This does mean that it will
be hard for nodes to directly push their result, since they would need a funded
account. Some workarounds could be:

	- make certain calls free. This mimics the current case where the explorer
  just accepts any calls, though the explorer will not actually consume any for
  bogus calls.

	- we can automatically set up a wallet for a node. This is not ideal since
  it means the farmer then needs to add some funds to the wallet, which is a
  manual step which might go wrong.

	- there is some talk about a `proxy` pallet, which would allow an account to
  make certain calls on behalf of someone else. We need to check if this is
  a good solution, and if it makes the call free for the caller (and offloads
  the costs to the authorized person).
