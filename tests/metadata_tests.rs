#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_valid_metadata() {
        let mut contract = MetadataContract::default();
        let mut data = HashMap::new();
        data.insert("type".to_string(), "payment".to_string());
        data.insert("amount".to_string(), "100".to_string());

        let msg = StoreMetadataMsg {
            tx_id: "tx123".to_string(),
            metadata: Metadata { data },
        };

        assert!(contract.store_metadata(msg.clone()).is_ok());

        let retrieved = contract.get_metadata("tx123".to_string());
        assert_eq!(retrieved.unwrap(), msg.metadata);
    }

    #[test]
    fn test_metadata_size_limit() {
        let mut contract = MetadataContract::default();

        let mut data = HashMap::new();
        data.insert("big".to_string(), "x".repeat(MAX_METADATA_SIZE + 1));

        let msg = StoreMetadataMsg {
            tx_id: "tx_big".to_string(),
            metadata: Metadata { data },
        };

        let result = contract.store_metadata(msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_metadata_format() {
        let mut contract = MetadataContract::default();

        let mut data = HashMap::new();
        data.insert("".to_string(), "value".to_string()); // invalid key

        let msg = StoreMetadataMsg {
            tx_id: "tx_invalid".to_string(),
            metadata: Metadata { data },
        };

        assert!(contract.store_metadata(msg).is_err());
    }
}
