// API module for the ICN system
pub mod routes;
pub mod handlers;

use crate::icn_types::IcnResult;

pub async fn initialize() -> IcnResult<()> {
    Ok(())
} 