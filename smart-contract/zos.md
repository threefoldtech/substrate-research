## Preparing zos

We integrated Zero-OS to work with this substrate node for provisioning and decomissioning of workloads. In order to test out the local setup you need the following:

- Install and run substrate-node
- Install [subkey](https://substrate.dev/docs/en/knowledgebase/integrate/subkey) and copy the binary to your zos node.
    - Note: Compile a static build for subkey! (`cargo build -p subkey --release --target x86_64-unknown-linux-musl`)
- Install client
- A farm on devnet (optional)
- Zos repo on branch **poc/substrate-contracts**

## Zos

When you checkout **poc/substrate-contracts** on the Zos repository you need to navigate to `pkg/provision/engine.go` and modify the websocket url (line 580 ish) for the client to your local ip address.

Run zos in a qemu:

```
cd qemu
/vm.sh -n node1 -c "runmode=dev farmer_id=YOUR_FARMER_ID"
```

### Copy the subkey binary to your zos node like this:

`scp subkey root@YOUR_NODE_IP:/bin`

### When the node succesfully booted you will see that provisiond logs something like:

`[+] provisiond: 2020-10-15T06:39:06Z info Node address: 5De2aCjDoJdGeGHX3CwyqNHwgt8TH6WKaw72CRLZazE2WsUg`

Copy this address, navigate to [https://polkadot.js.org/apps/#/explorer](https://polkadot.js.org/apps/#/explorer) in your browser. Next, click **Accounts** -> **Transfer**, transfer some funds from account **Alice** (any amount will do) to your address that your copied from your node.

This will active your node's address.

Zos is now ready to work with your local substrate node. Check out the client for available methods.