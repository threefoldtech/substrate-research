package sdb

import (
	"bytes"
	"fmt"

	"github.com/pkg/errors"

	"github.com/threefoldtech/0-stor/client/metastor/db"

	gsrpc "github.com/leesmet/go-substrate-rpc-client"
	"github.com/leesmet/go-substrate-rpc-client/hash"
	"github.com/leesmet/go-substrate-rpc-client/scale"
	"github.com/leesmet/go-substrate-rpc-client/signature"
	"github.com/leesmet/go-substrate-rpc-client/types"
)

// Substrate connector, forwarding calls to a substrate node
type Substrate struct {
	api         *gsrpc.SubstrateAPI
	meta        *types.Metadata
	genesisHash types.Hash
	rv          *types.RuntimeVersion
}

// New substrate api
func New() *Substrate {
	// TODO: customization
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

	s := &Substrate{
		api:         api,
		meta:        meta,
		genesisHash: genesisHash,
		rv:          rv,
	}

	return s
}

// Set implements db.DB
func (s *Substrate) Set(namespace, key, metadata []byte) error {
	c, err := types.NewCall(s.meta, "TemplateModule.set_metadata", namespace, key, metadata)
	if err != nil {
		return errors.Wrap(err, "could not create call")
	}

	ext := types.NewExtrinsic(c)

	nonce, err := s.getNonce()
	if err != nil {
		errors.Wrap(err, "could not get nonce")
	}

	sigOpts := s.getSigOpts(uint64(nonce))

	if err = ext.Sign(signature.TestKeyringPairAlice, sigOpts); err != nil {
		return errors.Wrap(err, "could not sign extrinsic")
	}

	_, err = s.api.RPC.Author.SubmitExtrinsic(ext)

	return errors.Wrap(err, "could not submit extrinsic")
}

type Meta struct {
	Data      []byte
	AccountID types.AccountID
}

// Get implements db.DB
func (s *Substrate) Get(namespace, key []byte) ([]byte, error) {
	// hasher, err := hash.NewBlake2b128(namespace)
	// if err != nil {
	// 	return nil, errors.Wrap(err, "failed to create hash")
	// }

	// hash := hasher.Sum(nil)

	// hash = append(hash, key...)
	// storageKey, err := types.CreateStorageKey(s.meta, "TemplateModule", "MetaStor", hash, nil)
	// if err != nil {
	// 	return nil, errors.Wrap(err, "could not create storage key")
	// }

	// var meta Meta
	// ok, err := s.api.RPC.State.GetStorageLatest(storageKey, &meta)
	// if err != nil || !ok {
	// 	return nil, errors.Wrap(err, "could not get latest storage key")
	// }

	// return meta.Data, nil

	fmt.Printf("%s %s \n", string(namespace), string(key))

	hasher, err := hash.NewBlake2b128(namespace)
	if err != nil {
		return nil, errors.Wrap(err, "failed to create hash")
	}

	hash := hasher.Sum(nil)

	hash = append(hash, key...)
	fmt.Printf("%x \n", hash)

	buf := bytes.NewBuffer(nil)
	enc := scale.NewEncoder(buf)
	err = enc.Encode(hash)
	if err != nil {
		panic(err)
	}
	hash = buf.Bytes()
	fmt.Printf("%x \n", hash)

	storageKey, err := types.CreateStorageKey(s.meta, "TemplateModule", "MetaStor", []byte{}, nil)
	if err != nil {
		return nil, errors.Wrap(err, "could not create storage key")
	}

	fmt.Printf("%x \n", storageKey)

	keys, err := s.api.RPC.State.GetKeysLatest(storageKey)
	if err != nil {
		return nil, errors.Wrap(err, "could not get latest storage key")
	}

	fmt.Println("storage keys:")
	for _, key := range keys {
		fmt.Printf("%x \n", key)
	}

	return nil, nil
}

// Delete implements db.DB
func (s *Substrate) Delete(namespace, key []byte) error {
	c, err := types.NewCall(s.meta, "TemplateModule.delete_metadata", namespace, key)
	if err != nil {
		return errors.Wrap(err, "could not create call")
	}

	ext := types.NewExtrinsic(c)

	nonce, err := s.getNonce()
	if err != nil {
		errors.Wrap(err, "could not get nonce")
	}

	sigOpts := s.getSigOpts(uint64(nonce))

	if err = ext.Sign(signature.TestKeyringPairAlice, sigOpts); err != nil {
		return errors.Wrap(err, "could not sign extrinsic")
	}

	_, err = s.api.RPC.Author.SubmitExtrinsic(ext)

	return errors.Wrap(err, "could not submit extrinsic")
}

// Update implements db.DB
func (s *Substrate) Update(namespace, key []byte, cb db.UpdateCallback) error {
	return nil
}

// ListKeys implements db.DB
func (s *Substrate) ListKeys(namespace []byte, cb db.ListCallback) error {
	fmt.Println("Listkeys")
	return nil
}

// Close implements db.DB
func (s *Substrate) Close() error {
	fmt.Println("Close")
	return nil
}

func (s *Substrate) getNonce() (uint32, error) {
	// Get the nonce for Alice
	key, err := types.CreateStorageKey(s.meta, "System", "Account", signature.TestKeyringPairAlice.PublicKey, nil)
	if err != nil {
		return 0, errors.Wrap(err, "could not create storage key")
	}

	var accountInfo types.AccountInfo
	ok, err := s.api.RPC.State.GetStorageLatest(key, &accountInfo)
	if err != nil || !ok {
		return 0, errors.Wrap(err, "could not get latest storage key")
	}

	nonce := uint32(accountInfo.Nonce)

	return nonce, nil
}

func (s *Substrate) getSigOpts(nonce uint64) types.SignatureOptions {
	return types.SignatureOptions{
		BlockHash:   s.genesisHash,
		Era:         types.ExtrinsicEra{IsMortalEra: false}, // TODO: really figure out what this means
		GenesisHash: s.genesisHash,
		Nonce:       types.NewUCompactFromUInt(nonce),
		SpecVersion: s.rv.SpecVersion,
		TxVersion:   1,
		Tip:         types.NewUCompactFromUInt(0), // still cheap
	}
}

var _ db.DB = (*Substrate)(nil)
