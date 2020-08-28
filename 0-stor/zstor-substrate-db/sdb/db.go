package sdb

import (
	"github.com/threefoldtech/0-stor/client/metastor/db"
)

// Substrate connector, forwarding calls to a substrate node
type Substrate struct {
}

// Set implements db.DB
func (*Substrate) Set(namespace, key, metadata []byte) error {
	return nil
}

// Get implements db.DB
func (*Substrate) Get(namespace, key []byte) ([]byte, error) {
	return nil, nil
}

// Delete implements db.DB
func (*Substrate) Delete(namespace, key []byte) error {
	return nil
}

// Update implements db.DB
func (*Substrate) Update(namespace, key []byte, cb db.UpdateCallback) error {
	return nil
}

// ListKeys implements db.DB
func (*Substrate) ListKeys(namespace []byte, cb db.ListCallback) error {
	return nil
}

// Close implements db.DB
func (*Substrate) Close() error {
	return nil
}

var _ db.DB = (*Substrate)(nil)
