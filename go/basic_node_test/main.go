package main

import (
	"fmt"

	gsrpc "github.com/Snowfork/go-substrate-rpc-client"
	"github.com/Snowfork/go-substrate-rpc-client/signature"
	"github.com/Snowfork/go-substrate-rpc-client/types"
)

func main() {
	api, err := gsrpc.NewSubstrateAPI("ws://localhost:9944")
	if err != nil {
		panic(err)
	}

	meta, err := api.RPC.State.GetMetadataLatest()
	if err != nil {
		panic(err)
	}

	genesisHash, err := api.RPC.Chain.GetBlockHash(0)
	if err != nil {
		panic(err)
	}

	rv, err := api.RPC.State.GetRuntimeVersionLatest()
	if err != nil {
		panic(err)
	}
	fmt.Printf("spec version: %d\n", rv.SpecVersion)

	ext := claimHashExtrinsic(meta, rv)

	nonce := getNonce(meta, api)
	fmt.Printf("nonce: %d\n", uint64(nonce))

	o := types.SignatureOptions{
		BlockHash:   genesisHash,
		Era:         types.ExtrinsicEra{IsMortalEra: false}, //TODO: whats this
		GenesisHash: genesisHash,
		Nonce:       types.NewUCompactFromUInt(uint64(nonce)),
		SpecVersion: rv.SpecVersion,
		TxVersion:   1,
		Tip:         types.NewUCompactFromUInt(0), // we are cheapo
	}

	// this requires the `subkey` tool to be installed
	err = ext.Sign(signature.TestKeyringPairAlice, o)
	if err != nil {
		panic(err)
	}

	fmt.Println("ext signed")

	// make claim and track real time status
	sub, err := api.RPC.Author.SubmitAndWatchExtrinsic(ext)
	if err != nil {
		panic(err)
	}
	defer sub.Unsubscribe()

	for {
		status := <-sub.Chan()
		fmt.Printf("Transaction status: %#v\n", status)

		if status.IsInBlock {
			fmt.Printf("Completed at block hash: %#x\n", status.AsInBlock)
			return
		}
	}

}

func claimHashExtrinsic(meta *types.Metadata, rv *types.RuntimeVersion) types.Extrinsic {
	c, err := types.NewCall(meta, "TemplateModule.create_claim", []byte("this is my unchecked hash"))
	if err != nil {
		panic(err)
	}

	ext := types.NewExtrinsic(c)

	return ext
}

func getNonce(meta *types.Metadata, api *gsrpc.SubstrateAPI) uint32 {
	// Get the nonce for Alice
	key, err := types.CreateStorageKey(meta, "System", "Account", signature.TestKeyringPairAlice.PublicKey, nil)
	if err != nil {
		panic(err)
	}

	var accountInfo types.AccountInfo
	ok, err := api.RPC.State.GetStorageLatest(key, &accountInfo)
	if err != nil || !ok {
		panic(err)
	}

	nonce := uint32(accountInfo.Nonce)

	return nonce
}
