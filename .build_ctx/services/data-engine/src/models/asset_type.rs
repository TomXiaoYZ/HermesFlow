use serde::{Deserialize, Serialize};

/// Asset type categorization for different financial instruments
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AssetType {
    /// Spot trading (immediate settlement)
    Spot,
    /// Perpetual futures (no expiry)
    Perpetual,
    /// Futures contracts (with expiry date)
    Future,
    /// Options contracts
    Option,
    /// Stock/equity instruments
    Stock,
    /// Market indices
    Index,
    /// Cryptocurrency
    Crypto,
}

impl AssetType {
    /// Returns a string representation of the asset type
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetType::Spot => "Spot",
            AssetType::Perpetual => "Perpetual",
            AssetType::Future => "Future",
            AssetType::Option => "Option",
            AssetType::Stock => "Stock",
            AssetType::Index => "Index",
            AssetType::Crypto => "Crypto",
        }
    }
}

impl std::fmt::Display for AssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_type_serialization() {
        let asset = AssetType::Spot;
        let json = serde_json::to_string(&asset).unwrap();
        assert_eq!(json, "\"Spot\"");

        let deserialized: AssetType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, AssetType::Spot);
    }

    #[test]
    fn test_asset_type_display() {
        assert_eq!(AssetType::Spot.to_string(), "Spot");
        assert_eq!(AssetType::Perpetual.to_string(), "Perpetual");
        assert_eq!(AssetType::Future.to_string(), "Future");
    }

    #[test]
    fn test_asset_type_equality() {
        assert_eq!(AssetType::Spot, AssetType::Spot);
        assert_ne!(AssetType::Spot, AssetType::Perpetual);
    }
}
