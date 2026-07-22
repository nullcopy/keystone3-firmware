use alloc::string::{String, ToString};

use alloc::vec::Vec;

use app_utils::impl_public_struct;
use ur_registry::{
    error::URResult,
    zcash::{
        zcash_accounts::ZcashAccounts, zcash_unified_full_viewing_key::ZcashUnifiedFullViewingKey,
    },
};

impl_public_struct!(UFVKInfo {
    key_text: String,
    key_name: String,
    index: u32
});

pub fn generate_sync_ur(
    key_infos: Vec<UFVKInfo>,
    seed_fingerprint: [u8; 32],
    device_version: Option<&str>,
) -> URResult<ZcashAccounts> {
    let keys = key_infos
        .iter()
        .map(|info| {
            Ok(ZcashUnifiedFullViewingKey::new(
                info.key_text.clone(),
                info.index,
                Some(info.key_name.clone()),
            ))
        })
        .collect::<URResult<Vec<ZcashUnifiedFullViewingKey>>>()?;
    let mut accounts = ZcashAccounts::new(seed_fingerprint.to_vec(), keys);
    if let Some(version) = device_version {
        accounts.set_device_version(version.to_string());
    }
    Ok(accounts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn test_generate_sync_ur() {
        let seed_fingerprint = [1u8; 32];
        let key_infos = vec![
            UFVKInfo {
                key_text: "uview1vmle95235860km865468566554".to_string(),
                key_name: "Account 0".to_string(),
                index: 0,
            },
            UFVKInfo {
                key_text: "uview1vmle95235860km865468566555".to_string(),
                key_name: "Account 1".to_string(),
                index: 1,
            },
        ];

        let result = generate_sync_ur(key_infos, seed_fingerprint, Some("1.2.3"));
        assert!(result.is_ok());

        let accounts = result.unwrap();
        let cbor: Vec<u8> = accounts.try_into().unwrap();
        assert!(!cbor.is_empty());
    }

    /// Pins the fact that `device_version` does NOT reach the wallet.
    ///
    /// `generate_sync_ur` sets it, but `ur-registry`'s `ZcashAccounts` encoder
    /// reports `map_size() == 2` and only ever writes `seed_fingerprint` and
    /// `accounts`; `DEVICE_VERSION` (key 3) exists in the decoder only. So the
    /// setter is a no-op on the wire and a wallet can never read the value.
    ///
    /// This is an upstream `ur-registry` gap, not something this crate can fix:
    /// either the encoder must emit key 3 (and bump `map_size`), or the
    /// firmware-side plumbing should be removed as dead. When the SDK starts
    /// encoding it, this test fails and should become the positive assertion.
    #[test]
    fn test_device_version_is_not_encoded_upstream_gap() {
        let seed_fingerprint = [1u8; 32];
        let key_infos = vec![UFVKInfo {
            key_text: "uview1vmle95235860km865468566554".to_string(),
            key_name: "Account 0".to_string(),
            index: 0,
        }];

        let accounts = generate_sync_ur(key_infos, seed_fingerprint, Some("1.2.3")).unwrap();
        assert_eq!(accounts.get_device_version(), Some("1.2.3".to_string()));

        let cbor: Vec<u8> = accounts.try_into().unwrap();
        let decoded = ZcashAccounts::try_from(cbor).unwrap();
        assert_eq!(
            decoded.get_device_version(),
            None,
            "device_version unexpectedly survived the round trip - the upstream \
             encoder gap is fixed; make this a positive assertion and confirm the \
             wallet-facing zcash-accounts UR is intentionally changing shape"
        );
    }
}
