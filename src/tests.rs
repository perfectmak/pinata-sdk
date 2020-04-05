use serde::Serialize;
use std::collections::HashMap;
use super::{PinataApi, HashPinPolicy, RegionPolicy, Region, PinByHash};
use super::{PinJobsFilter, SortDirection, JobStatus, PinByJson, PinByFile};

fn get_api() -> PinataApi {
  let api_key = std::env::var("API_KEY").expect("API_KEY env required to run test");
  let secret_api_key = std::env::var("SECRET_API_KEY").expect("SECRET_API_KEY env required to run test");
  super::PinataApi::new(api_key, secret_api_key).unwrap()
}

#[tokio::test]
async fn test_authentication_works() {
  let result = get_api().test_authentication().await;
  match result {
    Ok(_) => assert!(true),
    Err(_) => assert!(false),
  }
}

#[tokio::test]
async fn test_set_hash_pin_policy_works() {
  // Note the hash provided
  let result = get_api().set_hash_pin_policy(HashPinPolicy::new(
    "Qmbsjf1f3Z2AUX6H4PcbyUSdzJ7YZrZfzF246iaikYZja7",
    [
      RegionPolicy {
        id: Region::FRA1,
        desired_replication_count: 1,
      }
    ].to_vec()
  )).await;

  match result {
    Ok(_) => assert!(true),
    Err(_) => assert!(false),
  }
}

#[tokio::test]
async fn test_pin_by_hash_works() {
  let result = get_api().pin_by_hash(
    PinByHash::new("Qmbsjf1f3Z2AUX6H4PcbyUSdzJ7YZrZfzF246iaikYZja7")
  ).await;

  match result {
    Ok(data) => {
      debug!("{:?}", data);
      assert!(true)
    },
    Err(e) => assert!(false, "{}", e),
  }
}

#[tokio::test]
async fn test_get_pin_jobs() {
  let result = get_api().get_pin_jobs(PinJobsFilter::new()
    .set_sort(SortDirection::ASC)
    .set_status(JobStatus::Prechecking)
    .set_ipfs_pin_hash("Qmbsjf1f3Z2AUX6H4PcbyUSdzJ7YZrZfzF246iaikYZja7")
    .set_limit(1)
  ).await;

  match result {
    Ok(data) => {
      debug!("{:?}", data);
      assert_eq!(data.count, 0, "There should be 0 pinned jobs");
      assert_eq!(data.rows.len(), 0, "Zero count should also return zero rows");
    }
    Err(e) => assert!(false, "{}", e),
  }
}

#[tokio::test]
async fn test_pin_json_to_ipfs() {
  #[derive(Serialize)]
  struct TestData {
    name: String,
    package: String,
  }

  let result = get_api().pin_json(
    PinByJson::new(TestData {
      name: "Perfect Makanju".to_string(),
      package: "pinata_sdk".to_string(),
    })
  ).await;

  match result {
    Ok(data) => {
      debug!("{:?}", data);
      assert_eq!(data.ipfs_hash, "QmcDRRZ8Sy2QrpN8VySimHH5SToSPScW8yP8VmkZ2gDEJv");
      assert_eq!(data.pin_size, 57);
    }
    Err(e) => assert!(false, "{}", e),
  }
}

#[tokio::test]
async fn test_pin_json_to_ipfs_with_metadata() {
  #[derive(Serialize)]
  struct TestData {
    name: String,
    package: String,
  }

  let result = get_api().pin_json(
    PinByJson::new(TestData {
      name: "Perfect".to_string(),
      package: "pinata_sdk".to_string(),
    })
    .set_metadata_with_name("TaggedName", HashMap::new())
  ).await;

  match result {
    Ok(data) => {
      debug!("{:?}", data);
      assert_eq!(data.ipfs_hash, "QmScaKE4777guCGczz6giVSMX2QJhxAdBeXCqPnsivQE8f");
      assert_eq!(data.pin_size, 49);
    }
    Err(e) => assert!(false, "{}", e),
  }
}

#[tokio::test]
async fn test_pin_file_to_ipfs() {
  let result = get_api().pin_file(
    PinByFile::new("./test-file.txt")
  ).await;

  match result {
    Ok(data) => {
      debug!("{:?}", data);
      assert_eq!(data.ipfs_hash, "QmYW6YYCco35LGEmpm6oyJVijjTR5fPxvKxRULEauefXNH");
      assert_eq!(data.pin_size, 73);
    }
    Err(e) => assert!(false, "{}", e),
  }
}

#[tokio::test]
async fn test_pin_directory_to_ipfs() {
  let result = get_api().pin_file(
    PinByFile::new("./test-dir")
  ).await;

  match result {
    Ok(data) => {
      debug!("{:?}", data);
      assert_eq!(data.ipfs_hash, "QmYTyd2A15snZbRbWi2cbZkis45DzDdPSdzdF3wXdMEWVk");
      assert_eq!(data.pin_size, 291);
    }
    Err(e) => assert!(false, "{}", e),
  }
}

#[tokio::test]
async fn test_unpin() {
  #[derive(Serialize)]
  struct PinData {
    random: &'static str
  }
  let api = get_api();

  let pin_result = api.pin_json(PinByJson::new(PinData { random: "Tell me" }))
    .await
    .unwrap();

  let result = api.unpin(&pin_result.ipfs_hash).await;

  match result {
    Ok(_) => assert!(true),
    Err(e) => assert!(false, "{}", e),
  }
}