// Graph API URLs
pub const UNISWAP_V3_SUBGRAPH_URL: &str = "https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v3";
pub const ARBITRUM_SUBGRAPH_URL: &str = "https://api.thegraph.com/subgraphs/name/ianlapham/uniswap-arbitrum-one";
pub const OPTIMISM_SUBGRAPH_URL: &str = "https://api.thegraph.com/subgraphs/name/ianlapham/optimism-post-regenesis";

// Factory Addresses
pub const FACTORY_ADDRESS: &str = "0x1F98431c8aD98523631AE4a59f267346ea31F984";
pub const POSITION_MANAGER_ADDRESS: &str = "0xC36442b4a4522E871399CD717aBDD847Ab11FE88";

// Fee Tiers
pub const FEE_LOW: u32 = 500;      // 0.05%
pub const FEE_MEDIUM: u32 = 3000;   // 0.3%
pub const FEE_HIGH: u32 = 10000;    // 1%

// Event Topics
pub const SWAP_EVENT_TOPIC: &str = "0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67";
pub const MINT_EVENT_TOPIC: &str = "0x7a53080ba414158be7ec69b987b5fb7d07dee101fe85488f0853ae16239d0bde";
pub const BURN_EVENT_TOPIC: &str = "0x0c396cd989a39f4459b5fa1aed6a9a8dcdbc45908acfd67e028cd568da98982c";
pub const COLLECT_EVENT_TOPIC: &str = "0x70935338e69775456a85ddef226c395fb668b63fa0115f5f20610b388e6ca9c0";

// Price Precision
pub const PRICE_DECIMALS: u8 = 18;
pub const TICK_BASE: i32 = 1_0000; 