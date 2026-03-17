#![forbid(unsafe_code)]

#[derive(Clone, Debug)]
pub struct IdentityConfig {
    pub primary_bostrom: String,
    pub alt_bostrom: String,
    pub safe_alt_zeta: String,
    pub safe_alt_hex: String, // e.g. 0x519f... ERC‑20 compatible
}

impl IdentityConfig {
    pub fn phoenix_default() -> Self {
        Self {
            primary_bostrom:
                "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7".to_string(),
            alt_bostrom:
                "bostrom1ldgmtf20d6604a24ztr0jxht7xt7az4jhkmsrc".to_string(),
            safe_alt_zeta:
                "zeta12x0up66pzyeretzyku8p4ccuxrjqtqpdc4y4x8".to_string(),
            safe_alt_hex:
                "0x519fC0eB4111323Cac44b70e1aE31c30e405802D".to_string(),
        }
    }
}
