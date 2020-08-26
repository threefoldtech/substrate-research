# Basic node test

Small go program to demonstrate rpc calls to a custom substrate pallet.
Rather than using the official centrifuge gsrpc client, this module uses the
fork of Snowfork. At the time of writing, the centrifuge module only supports
supstrate v2.0.0-rc3, whereas the fork at Snowfork is fully compatible with
v2.0.0-rc4. This fork is set to be merged upstream in the near future.

In order for the example to work, a local substrate node is expected to be running
with the default websocket port. The custom pallet used is the one created in the
`poe dapp` tutorial. As such, the code won't be included here. Do note that if
you want to run it yourself, you will need to check out the repo at tag `v2.0.0-rc4`.

Furthermore we expect the node to be running in `dev` mode, since the default 
`Alice` account is used to sign the call. In dev mode, this account is preloaded
and funded.

The first time the program is run, the hash set in the code (actually just a byte
slice, no actual validation of any kind is done) will be claimed by the user. Subsequent
runs without resetting the chain or revoking the claim will cause the call to error.
The program will not show the error however. Reason being that the extrinsic will
be submitted successfully, but its execution will fail.

If you followed the tutorial and run the web ui as well, you will see a `ClaimCreated`
event appear in the ui. Subsequent runs, as said, will lead to a `ExtrinsicFailed`
error.

Keep in mind that the `subkey` tool needs to be installed at the correct version
(`v2.0.0-rc4`). Unfortunately, installing it from github does not seem to work,
since cargo does not honor the lockfile, and an import from libp2p will fail.
Instead, you must clone the substrate repo at the right tag, and then build the
subkey binary in the local repo. This will force cargo to correctly honor the lockfile.
