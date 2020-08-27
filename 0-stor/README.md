# 0-stor metadata replicated on substrate chain

The aim of this project is to provide a poc of a substrate based chain to replicate
metadata for 0-stors. The goal is to upload a file on a 0-stor on 1 pc, which
will then save the metadata on a local substrate node. On another pc with a
connected substrate node, a local 0-stor will then download the file.

Since the main idea is not to test the replication of substrate, a single
node can also be used. In essence, the idea is to prove that metadata can be
stored and fetched from substrate, so a minimal sample would have just a single
local node, with a single 0-stor, where the 0-stor first uploads a file, then
later uses the key to download it.

## 0-stor limitation

If run with multiple 0-stor instances, they will operate on a shared config, as
the metadata alone is currently not enough for 0-stor to discover and connect to
the data shards.
