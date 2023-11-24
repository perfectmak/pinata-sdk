# pinata-sdk

![Rust](https://github.com/perfectmak/pinata-sdk/workflows/Rust/badge.svg)
![pinata-sdk](https://docs.rs/pinata-sdk/badge.svg)

The `pinata_sdk` provides the easieset path for interacting with the [Pinata API](https://pinata.cloud/documentation#GettingStarted).

### Setup

Add the crate as a dependency to your codebase

```toml
[dependencies]
pinata-sdk = "1.1.0"
```

### Initializing the API
```rust
use pinata_sdk::PinataApi;

let api = PinataApi::new("api_key", "secret_api_key").unwrap();

// test that you can connect to the API:
let result = api.test_authentication().await;
if let Ok(_) = result {
  // credentials are correct and other api calls can be made
}
```

### Usage

#### 1. Pinning a file

Send a file to pinata for direct pinning to IPFS.

```rust
use pinata_sdk::{ApiError, PinataApi, PinByFile};

let api = PinataApi::new("api_key", "secret_api_key").unwrap();

let result = api.pin_file(PinByFile::new("file_or_dir_path")).await;

if let Ok(pinned_object) = result {
  let hash = pinned_object.ipfs_hash;
}
```

If a directory path is used to construct `PinByFile`, then `pin_file()` will upload all the contents
of the file to be pinned on pinata.

#### 2. Pinning a JSON object

You can send a JSON serializable to pinata for direct pinning to IPFS.

```rust
use pinata_sdk::{ApiError, PinataApi, PinByJson};
use std::collections::HashMap;

let api = PinataApi::new("api_key", "secret_api_key").unwrap();

// HashMap derives serde::Serialize
let mut json_data = HashMap::new();
json_data.insert("name", "user");

let result = api.pin_json(PinByJson::new(json_data)).await;

if let Ok(pinned_object) = result {
  let hash = pinned_object.ipfs_hash;
}
```

#### 3. Unpinning

You can unpin using the `PinataApi::unpin()` function by passing in the CID hash of the already
pinned content.


## Contribution Guide

Feel free to contribute. Please ensure that an issue is exists that describes the feature or bugfix you are planning to contribute.

Also, this README is generated using the [cargo-readme](https://github.com/livioribeiro/cargo-readme) crate from the `README.tpl` file, so 
update that file and run `cargo readme > README.md` to update the README's content. (This process can definitely be improved by running this 
step in a build script).

## License

MIT OR Apache-2.0
