#![deny(missing_docs)]
//! ## Initializing the API
//! ```
//! use pinata_sdk::PinataApi;
//! # use pinata_sdk::ApiError;
//! 
//! # async fn run() -> Result<(), ApiError> {
//! let api = PinataApi::new("api_key", "secret_api_key").unwrap();
//! 
//! // test that you can connect to the API:
//! let result = api.test_authentication().await;
//! if let Ok(_) = result {
//!   // credentials are correct and other api calls can be made
//! }
//! # Ok(())
//! # }
//! ```
//! 
//! ## Usage
//! 
//! ### 1. Pinning a file
//! 
//! Send a file to pinata for direct pinning to IPFS.
//! 
//! ```
//! use pinata_sdk::{ApiError, PinataApi, PinByFile};
//! 
//! # async fn run() -> Result<(), ApiError> {
//! let api = PinataApi::new("api_key", "secret_api_key").unwrap();
//! 
//! let result = api.pin_file(PinByFile::new("file_or_dir_path")).await;
//! 
//! if let Ok(pinned_object) = result {
//!   let hash = pinned_object.ipfs_hash;
//! }
//! # Ok(())
//! # }
//! ```
//! 
//! If a directory path is used to construct `PinByFile`, then `pin_file()` will upload all the contents
//! of the file to be pinned on pinata.
//! 
//! ### 2. Pinning a JSON object
//! 
//! You can send a JSON serializable to pinata for direct pinning to IPFS.
//! 
//! ```
//! use pinata_sdk::{ApiError, PinataApi, PinByJson};
//! use std::collections::HashMap;
//! 
//! # async fn run() -> Result<(), ApiError> {
//! let api = PinataApi::new("api_key", "secret_api_key").unwrap();
//! 
//! // HashMap derives serde::Serialize
//! let mut json_data = HashMap::new();
//! json_data.insert("name", "user");
//! 
//! let result = api.pin_json(PinByJson::new(json_data)).await;
//! 
//! if let Ok(pinned_object) = result {
//!   let hash = pinned_object.ipfs_hash;
//! }
//! # Ok(())
//! # }
//! ```
//! 
//! ### 3. Unpinning
//!
//! You can unpin using the `PinataApi::unpin()` function by passing in the CID hash of the already
//! pinned content.
//! 

#[cfg_attr(test, macro_use)]
extern crate log;
extern crate derive_builder;

use std::fs;
use std::path::Path;
use reqwest::{Client, ClientBuilder, header::HeaderMap, multipart::{Form, Part}, Response};
use walkdir::WalkDir;
use serde::{Serialize};
use serde::de::DeserializeOwned;
use errors::Error;
use utils::api_url;
use api::internal::*;

pub use api::data::*;
pub use api::metadata::*;
pub use errors::ApiError;

mod api;
mod utils;
mod errors;

/// API struct. Exposes functions to interact with the Pinata API
pub struct PinataApi {
  client: Client,
}

impl PinataApi {
  /// Creates a new instance of PinataApi using the provided keys.
  /// This function panics if api_key or secret_api_key's are empty/blank
  pub fn new<S: Into<String>>(api_key: S, secret_api_key: S) -> Result<PinataApi, Error> {
    let owned_key = api_key.into();
    let owned_secret = secret_api_key.into();

    utils::validate_keys(&owned_key, &owned_secret)?;

    let mut default_headers = HeaderMap::new();
    default_headers.insert("pinata_api_key", (&owned_key).parse().unwrap());
    default_headers.insert("pinata_secret_api_key", (&owned_secret).parse().unwrap());

    let client = ClientBuilder::new()
      .default_headers(default_headers)
      .build()?;

    Ok(PinataApi {
      client,
    })
  }

  /// Test if your credentials are corrects. It returns an error if credentials are not correct
  pub async fn test_authentication(&self) -> Result<(), ApiError> {
    let response = self.client.get(&api_url("/data/testAuthentication"))
      .send()
      .await?;

    self.parse_ok_result(response).await
  }

  /// Change the pin policy for an individual piece of content.
  ///
  /// Changes made via this function only affect the content for the hash passed in. They do not affect a user's account level pin policy.
  ///
  /// To read more about pin policies, please check out the [Regions and Replications](https://pinata.cloud/documentation#RegionsAndReplications) documentation
  pub async fn set_hash_pin_policy(&self, policy: HashPinPolicy) -> Result<(), ApiError> {
    let response = self.client.put(&api_url("/pinning/hashPinPolicy"))
      .json(&policy)
      .send()
      .await?;

    self.parse_ok_result(response).await
  }

  /// Add a hash to Pinata for asynchronous pinning.
  /// 
  /// Content added through this function is pinned in the background. Fpr this operation to succeed, the 
  /// content for the hash provided must already be pinned by another node on the IPFS network.
  pub async fn pin_by_hash(&self, hash: PinByHash) -> Result<PinByHashResult, ApiError> {
    let response = self.client.post(&api_url("/pinning/pinByHash"))
      .json(&hash)
      .send()
      .await?;

    self.parse_result(response).await
  }

  /// Retrieve a list of all the pins that are currently in the pin queue for your user
  pub async fn get_pin_jobs(&self, filters: PinJobsFilter) -> Result<PinJobs, ApiError> {
    let response = self.client.get(&api_url("/pinning/pinJobs"))
      .query(&filters)
      .send()
      .await?;

    self.parse_result(response).await
  }

  /// Pin any JSON serializable object to Pinata IPFS nodes.
  pub async fn pin_json<S>(&self, pin_data: PinByJson<S>) -> Result<PinnedObject, ApiError> 
    where S: Serialize
  {
    let response = self.client.post(&api_url("/pinning/pinJSONToIPFS"))
      .json(&pin_data)
      .send()
      .await?;

    self.parse_result(response).await
  }

  /// Pin any file or folder to Pinata's IPFS nodes.
  /// 
  /// To upload a file use `PinByFile::new("file_path")`. If file_path is a directory, all the content
  /// of the directory will be uploaded to IPFS and the hash of the parent directory is returned.
  ///
  /// If the file cannot be read or directory cannot be read an error will be returned.
  pub async fn pin_file(&self, pin_data: PinByFile) -> Result<PinnedObject, ApiError> {
    let mut form = Form::new();

    for file_data in pin_data.files {
      let base_path = Path::new(&file_data.file_path);
      if base_path.is_dir() {
        // recursively read the directory
        for entry_result in WalkDir::new(base_path) {
          let entry = entry_result?;
          let path = entry.path();

          // not interested in reading directory
          if path.is_dir() { continue }

          let path_name = path.strip_prefix(base_path)?;
          let part_file_name = format!(
            "{}/{}", 
            base_path.file_name().unwrap().to_str().unwrap(),
            path_name.to_str().unwrap()
          );
          
          let part = Part::bytes(fs::read(path)?)
            .file_name(part_file_name);
          form = form.part("file", part);
        }
      } else {
        let file_name = base_path.file_name().unwrap().to_str().unwrap();
        let part = Part::bytes(fs::read(base_path)?);
        form = form.part("file", part.file_name(String::from(file_name)));
      }
    }
    
    if let Some(metadata) = pin_data.pinata_metadata {
      form = form.text("pinataMetadata", serde_json::to_string(&metadata).unwrap());
    }
    
    if let Some(option) = pin_data.pinata_option {
      form = form.text("pinataOptions", serde_json::to_string(&option).unwrap());
    }
    
    let response = self.client.post(&api_url("/pinning/pinFileToIPFS"))
      .multipart(form)
      .send()
      .await?;

    self.parse_result(response).await
  }

  /// Unpin content previously uploaded to the Pinata's IPFS nodes.
  pub async fn unpin(&self, hash: &str) -> Result<(), ApiError> {
    let response = self.client.delete(&api_url(&format!("/pinning/unpin/{}", hash)))
      .send()
      .await?;

    self.parse_ok_result(response).await
  }

  /// Change name and custom key values associated for a piece of content stored on Pinata.
  pub async fn change_hash_metadata(&self, change: ChangePinMetadata) -> Result<(), ApiError> {
    let response = self.client.put(&api_url("/pinning/hashMetadata"))
      .json(&change)
      .send()
      .await?;

    self.parse_ok_result(response).await
  }

  /// This endpoint returns the total combined size for all content that you've pinned through Pinata
  pub async fn get_total_user_pinned_data(&self) ->  Result<TotalPinnedData, ApiError> {
    let response = self.client.get(&api_url("/data/userPinnedDataTotal"))
      .send()
      .await?;

    self.parse_result(response).await
  }

  /// This returns data on what content the sender has pinned to IPFS from pinata
  /// 
  /// The purpose of this endpoint is to provide insight into what is being pinned, and how
  /// long it has been pinned. The results of this call can be filtered using [PinListFilter](struct.PinListFilter.html).
  pub async fn get_pin_list(&self, filters: PinListFilter) -> Result<PinList, ApiError> {
    let response = self.client.get(&api_url("/data/pinList"))
      .query(&filters)
      .send()
      .await?;

    self.parse_result(response).await
  }

  async fn parse_result<R>(&self, response: Response) -> Result<R, ApiError> 
    where R: DeserializeOwned
  {
    if response.status().is_success() {
      let result = response.json::<R>().await?;
      Ok(result)
    } else {
      let error = response.json::<PinataApiError>().await?;
      Err(ApiError::GenericError(error.message()))
    }
  }

  async fn parse_ok_result(&self, response: Response) -> Result<(), ApiError> {
    if response.status().is_success() {
      Ok(())
    } else {
      let error = response.json::<PinataApiError>().await?;
      Err(ApiError::GenericError(error.message()))
    }
  }
}

#[cfg(test)]
mod tests;
