//! Default ports, payments, and product constants.

pub const DEFAULT_APP_ADDR: &str = "0.0.0.0";
pub const DEFAULT_APP_PORT: u16 = 8080;
pub const DEFAULT_RPC_URL: &str = "http://127.0.0.1:11101";
pub const DEFAULT_SSE_URL: &str = "http://127.0.0.1:18101/events";
pub const DEFAULT_CHAIN_NAME: &str = "casper-net-1";

/// Default payment (motes) for native transfer / light calls.
pub const DEFAULT_CALL_PAYMENT: &str = "5000000000";
/// Default payment (motes) for contract install.
pub const DEFAULT_INSTALL_PAYMENT: &str = "400000000000";
