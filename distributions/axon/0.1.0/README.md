# Axon 0.1.0

Signed command-line client for Plexus RPC, including its ACP v1 stdio adapter.
The Linux x86-64 archive requires glibc 2.30 or newer.

Install using the release manifest and compare the trust key at <https://plexus.hypermemetic.ai/verify/>.

```sh
cargo binstall plexus-axon --manifest-path . --version 0.1.0 --only-signed --strategies crate-meta-data
axon --version
```

OpenPGP release fingerprint: `1886D62ABE00420F943787D76B4B765639FC1B52`.
