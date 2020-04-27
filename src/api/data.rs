use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::api::metadata::{PinMetadata, MetadataKeyValues, MetadataValue};

#[derive(Clone, Debug, Deserialize, Serialize)]
/// All the currently supported regions on Pinata
pub enum Region {
  /// Frankfurt, Germany (max 2 replications)
  FRA1,
  /// New York City, USA (max 2 replications)
  NYC1,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
/// Region and desired replication for that region
pub struct RegionPolicy {
  /// Region Id
  pub id: Region,
  /// Replication count for the region. Maximum of 2 in most regions
  pub desired_replication_count: u8,
}

#[derive(Debug, Deserialize, Serialize)]
/// Pinata Pin Policy Regions
pub struct PinPolicy {
  /// List of regions and their Policy
  pub regions: Vec<RegionPolicy>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
/// Represents a PinPolicy linked to a particular ipfs pinned hash
pub struct HashPinPolicy {
  ipfs_pin_hash: String,
  new_pin_policy: PinPolicy,
}

impl HashPinPolicy {
  /// Create a new HashPinPolicy.
  ///
  /// See the [pinata docs](https://pinata.cloud/documentation#HashPinPolicy) for more information.
  pub fn new<S>(ipfs_pin_hash: S, regions: Vec<RegionPolicy>) -> HashPinPolicy 
    where S: Into<String>
  {
    HashPinPolicy {
      ipfs_pin_hash: ipfs_pin_hash.into(),
      new_pin_policy: PinPolicy { regions },
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// Status of Jobs
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
  /// Pinata is running preliminary validations on your pin request.
  Prechecking,
  /// Pinata is actively searching for your content on the IPFS network.
  Searching,
  /// Pinata has located your content and is now in the process of retrieving it.
  Retrieving,
  /// Pinata wasn't able to find your content after a day of searching the IPFS network.
  Expired,
  /// Pinning this object would put you over the free tier limit. Please add a credit card 
  /// to continue.
  OverFreeLimit,
  /// This object is too large of an item to pin. If you're seeing this, please contact pinata 
  /// for a more custom solution.
  OverMaxSize,
  /// The object you're attempting to pin isn't readable by IPFS nodes.
  InvalidObject,
  /// You provided a host node that was either invalid or unreachable.
  BadHostNode,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
/// Represents response of a pinByHash request.
pub struct PinByHashResult {
  /// This is Pinata's ID for the pin job.
  pub id: String,
  /// This is the IPFS multi-hash provided to Pinata to pin.
  pub ipfs_hash: String,
  /// Current status of the pin job.
  pub status: JobStatus,
  /// The name of the pin (if provided initially)
  pub name: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
/// Used to add additional options when pinning by hash
pub struct PinOptions {
  /// multiaddresses of nodes your content is already stored on
  pub host_nodes: Option<Vec<String>>,
  /// Custom pin policy for the piece of content being pinned
  pub custom_pin_policy: Option<PinPolicy>,
  /// CID Version IPFS will use when creating a hash for your content
  pub cid_version: Option<u8>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
/// Request object to pin hash of an already existing IPFS hash to pinata.
/// 
/// ## Example
/// ```
/// # use pinata_sdk::{ApiError, PinataApi, PinByHash};
/// # async fn run() -> Result<(), ApiError> {
/// let api = PinataApi::new("api_key", "secret_api_key").unwrap();
/// 
/// let result = api.pin_by_hash(PinByHash::new("hash")).await;
/// 
/// if let Ok(pinning_job) = result {
///   // track result job here
/// }
/// # Ok(())
/// # }
/// ```
pub struct PinByHash {
  hash_to_pin: String,
  pinata_metadata: Option<PinMetadata>,
  pinata_option: Option<PinOptions>,
}

impl PinByHash {
  /// Create a new default PinByHash object with only the hash to pin set.
  /// 
  /// To set the pinata metadata and pinata options use the `set_metadata()` and 
  /// `set_options()` chainable function to set those values.
  pub fn new<S>(hash: S) -> PinByHash 
    where S: Into<String>
  {
    PinByHash {
      hash_to_pin: hash.into(),
      pinata_metadata: None,
      pinata_option: None,
    }
  }

  /// Consumes the current PinByHash and returns a new PinByHash with keyvalues metadata set
  pub fn set_metadata(self, keyvalues: MetadataKeyValues) -> PinByHash {
    PinByHash {
      hash_to_pin: self.hash_to_pin,
      pinata_metadata: Some(PinMetadata {
        keyvalues,
        name: None,
      }),
      pinata_option: self.pinata_option,
    }
  }

  /// Consumes the current PinByHash and returns a new PinByHash with metadata name and keyvalues set
  pub fn set_metadata_with_name<S>(self, name: S, keyvalues: HashMap<String, MetadataValue>) -> PinByHash 
    where S: Into<String>
  {
    PinByHash {
      hash_to_pin: self.hash_to_pin,
      pinata_metadata: Some(PinMetadata {
        keyvalues,
        name: Some(name.into()),
      }),
      pinata_option: self.pinata_option,
    }
  }

  /// Consumes the PinByHash and returns a new PinByHash with pinata options set.
  pub fn set_options(self, options: PinOptions) -> PinByHash {
    PinByHash {
      hash_to_pin: self.hash_to_pin,
      pinata_metadata: self.pinata_metadata,
      pinata_option: Some(options),
    }
  }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
/// Request object to pin json
/// 
/// ## Example
/// ```
/// # use pinata_sdk::{ApiError, PinataApi, PinByJson};
/// # use std::collections::HashMap;
/// # async fn run() -> Result<(), ApiError> {
/// let api = PinataApi::new("api_key", "secret_api_key").unwrap();
/// 
/// let mut json_data = HashMap::new();
/// json_data.insert("name", "user");
/// 
/// let result = api.pin_json(PinByJson::new(json_data)).await;
/// 
/// if let Ok(pinned_object) = result {
///   let hash = pinned_object.ipfs_hash;
/// }
/// # Ok(())
/// # }
/// ```
pub struct PinByJson<S: Serialize> {
  pinata_content: S,
  pinata_metadata: Option<PinMetadata>,
  pinata_option: Option<PinOptions>,
}

impl <S> PinByJson<S>
  where S: Serialize
{
  /// Create a new default PinByHash object with only the hash to pin set.
  /// 
  /// To set the pinata metadata and pinata options use the `set_metadata()` and 
  /// `set_options()` chainable function to set those values.
  pub fn new(json_data: S) -> PinByJson<S> {
    PinByJson {
      pinata_content: json_data,
      pinata_metadata: None,
      pinata_option: None,
    }
  }

  /// Consumes the current PinByJson<S> and returns a new PinByJson<S> with keyvalues metadata set
  pub fn set_metadata(mut self, keyvalues: MetadataKeyValues) -> PinByJson<S> {
    self.pinata_metadata = Some(PinMetadata {
      name: None,
      keyvalues,
    });
    self
  }

  /// Consumes the current PinByJson<S> and returns a new PinByJson<S> with keyvalues metadata set
  pub fn set_metadata_with_name<IntoStr>(
    mut self, name: IntoStr,
    keyvalues: MetadataKeyValues
  ) -> PinByJson<S> 
    where IntoStr: Into<String>
  {
    self.pinata_metadata = Some(PinMetadata {
      name: Some(name.into()),
      keyvalues,
    });
    self
  }

  /// Consumes the PinByHash and returns a new PinByHash with pinata options set.
  pub fn set_options(mut self, options: PinOptions) -> PinByJson<S> {
    self.pinata_option = Some(options);
    self
  }
}

#[derive(Clone)]
///  Internal structure use to know how to read a file or structure
pub(crate) struct FileData {
  pub(crate) file_path: String,
}

/// Request object to pin a file
/// 
/// ## Example
/// ```
/// # use pinata_sdk::{ApiError, PinataApi, PinByFile};
/// # async fn run() -> Result<(), ApiError> {
/// let api = PinataApi::new("api_key", "secret_api_key").unwrap();
/// 
/// let result = api.pin_file(PinByFile::new("file_or_dir_path")).await;
/// 
/// if let Ok(pinned_object) = result {
///   let hash = pinned_object.ipfs_hash;
/// }
/// # Ok(())
/// # }
/// ```
pub struct PinByFile {
  pub(crate) files: Vec<FileData>,
  pub(crate) pinata_metadata: Option<PinMetadata>,
  pub(crate) pinata_option: Option<PinOptions>,
}

impl PinByFile {
  /// Create a PinByFile object.
  /// 
  /// `file_or_dir_path` can be path to a file or to a directory.
  /// If a directory is provided
  pub fn new<S: Into<String>>(file_or_dir_path: S) -> PinByFile {
    let owned_file_path = file_or_dir_path.into();
    PinByFile {
      files: [
        FileData { file_path: owned_file_path }
      ].to_vec(),
      pinata_metadata: None,
      pinata_option: None,
    }
  }

  /// Consumes the current PinByFile and returns a new PinByFile with keyvalues metadata set
  pub fn set_metadata(mut self, keyvalues: MetadataKeyValues) -> PinByFile {
    self.pinata_metadata = Some(PinMetadata {
      name: None,
      keyvalues,
    });
    self
  }

  /// Consumes the current PinByFile and returns a new PinByFile with keyvalues metadata set
  pub fn set_metadata_with_name<IntoStr>(
    mut self, name: IntoStr,
    keyvalues: MetadataKeyValues
  ) -> PinByFile
    where IntoStr: Into<String>
  {
    self.pinata_metadata = Some(PinMetadata {
      name: Some(name.into()),
      keyvalues,
    });
    self
  }

  /// Consumes the PinByHash and returns a new PinByHash with pinata options set.
  pub fn set_options(mut self, options: PinOptions) -> PinByFile {
    self.pinata_option = Some(options);
    self
  }
}

#[derive(Clone, Serialize)]
/// Sort Direction
pub enum SortDirection {
  /// Sort by ascending dates
  ASC,
  /// Sort by descending dates
  DESC,
}

#[derive(Clone, Default, Serialize)]
/// Filter parameters for fetching PinJobs
/// 
/// Example of how to use this:
/// ```
/// use pinata_sdk::{PinJobsFilter, SortDirection, JobStatus};
///
/// let filters = PinJobsFilter::new()
///   .set_sort(SortDirection::ASC)
///   .set_status(JobStatus::Prechecking);
///   // and change other possible filter set methods
/// ```
pub struct PinJobsFilter {
  sort: Option<SortDirection>,
  status: Option<JobStatus>,
  ipfs_pin_hash: Option<String>,
  limit: Option<u16>,
  offset: Option<u64>,
}

impl PinJobsFilter {
  /// Create a new PinJobsFilter.
  /// 
  /// It initialize all possible filters to None (the default)
  pub fn new() -> PinJobsFilter {
    PinJobsFilter::default()
  }

  /// Set a sort direction on the PinJobsFilter
  pub fn set_sort(mut self, direction: SortDirection) -> PinJobsFilter {
    self.sort = Some(direction);
    self
  }

  /// Set a status on the PinJobsFilter
  pub fn set_status(mut self, status: JobStatus) -> PinJobsFilter {
    self.status = Some(status);
    self
  }

  /// Set a IPFS pin hash on the PinJobsFilter
  pub fn set_ipfs_pin_hash<S: Into<String>>(mut self, hash: S) -> PinJobsFilter {
    self.ipfs_pin_hash = Some(hash.into());
    self
  }

  /// Set limit on the amount of results per page
  pub fn set_limit(mut self, limit: u16) -> PinJobsFilter {
    self.limit = Some(limit);
    self
  }

  /// Set the record offset for records returned. This is how to retrieve additional pages
  pub fn set_offset(mut self, offset: u64) -> PinJobsFilter {
    self.offset = Some(offset);
    self
  }
}

#[derive(Debug, Deserialize)]
/// Pin Job Record
pub struct PinJob {
  /// The id for the pin job record
  pub id: String,
  /// The IPFS mult-hash for the content pinned.
  pub ipfs_pin_hash: String,
  /// The date hash was initially queued. Represented in ISO8601 format
  pub date_queued: String,
  /// The current status for the pin job
  pub status: JobStatus,
  /// Optional name passed for hash
  pub name: Option<String>,
  /// Optional keyvalues metadata passsed for hash
  pub keyvalues: Option<HashMap<String, String>>,
  /// Optional list of host nodes passed for the hash
  pub host_nodes: Option<Vec<String>>,
  /// PinPolicy applied to content once it is found
  pub pin_policy: Option<PinPolicy>,
}

#[derive(Debug, Deserialize)]
/// Represents a list of pin job records for a set of filters.
pub struct PinJobs {
  /// Total number of pin job records that exist for the PinJobsFilter used
  pub count: u64,
  /// Each item in the rows represents a pin job record
  pub rows: Vec<PinJob>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
/// Represents a PinnedObject
pub struct PinnedObject {
  /// IPFS multi-hash provided back for your content
  pub ipfs_hash: String,
  /// This is how large (in bytes) the content you just pinned is
  pub pin_size: u64,
  /// Timestamp for your content pinning in ISO8601 format
  pub timestamp: String
}

#[derive(Debug, Deserialize)]
/// Results of a call to get total users pinned data
pub struct TotalPinnedData {
  /// The number of pins you currently have pinned with Pinata
  pub pin_count: u128,
  /// The total size of all unique content you have pinned with Pinata (expressed in bytes)
  pub pin_size_total: String,
  /// The total size of all content you have pinned with Pinata. This value is derived by multiplying the size of each piece of unique content by the number of times that content is replicated.
  pub pin_size_with_replications_total: String,
}