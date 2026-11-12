use hmac::{Hmac, Mac};
use sha2::Sha256;
use uuid::Uuid;

pub fn random_token() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}
pub(crate) fn decode(value: &str) -> Option<Vec<u8>> {
    if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    (0..value.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&value[i..i + 2], 16).ok())
        .collect()
}
pub fn proof(
    key: &str,
    role: &str,
    artifact: &str,
    instance: &str,
    pid: u32,
    client: &str,
    server: &str,
) -> Option<String> {
    let mut mac = Hmac::<Sha256>::new_from_slice(&decode(key)?).ok()?;
    let message = serde_json::to_vec(&(
        "injectra-bootstrap-v1",
        role,
        artifact,
        instance,
        pid,
        client,
        server,
    ))
    .ok()?;
    mac.update(&message);
    Some(
        mac.finalize()
            .into_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect(),
    )
}
pub(crate) fn verify(expected: &str, actual: &str) -> bool {
    let Some(key) = decode(expected) else {
        return false;
    };
    let Some(actual) = decode(actual) else {
        return false;
    };
    // A keyed comparison through Mac::verify_slice avoids ordinary string equality.
    let mut a = Hmac::<Sha256>::new_from_slice(&key).expect("HMAC key");
    a.update(&actual);
    let mut b = Hmac::<Sha256>::new_from_slice(&key).expect("HMAC key");
    b.update(&key);
    a.verify_slice(&b.finalize().into_bytes()).is_ok()
}
