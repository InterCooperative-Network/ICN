use crate::icn_types::IcnResult;

pub async fn health_check() -> IcnResult<&'static str> {
    Ok("OK")
}
