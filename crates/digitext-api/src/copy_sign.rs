// Copyright (c) 2026 DigiText. All rights reserved.
use rsa::pkcs8::DecodePrivateKey;
use rsa::signature::hazmat::PrehashSigner;
use rsa::signature::SignatureEncoding;
pub fn f_1ov() -> Result<Option<(rsa::RsaPrivateKey, Vec<u8>)>, String> {
    let (Ok(key_path), Ok(cert_path)) = (
        std::env::var("DIGITEXT_COPY_SIGN_KEY"),
        std::env::var("DIGITEXT_COPY_SIGN_CERT"),
    ) else {
        return Ok(None);
    };
    let r_3l = std::fs::read_to_string(&key_path)
        .map_err(|e| format!("đọc khóa ký bản sao {key_path}: {e}"))?;
    let r_3k = rsa::RsaPrivateKey::from_pkcs8_pem(&r_3l).map_err(|e| {
        format!("khóa ký bản sao không phải PKCS#8 PEM: {e}")
    })?;
    let r_3j = std::fs::read_to_string(&cert_path)
        .map_err(|e| format!("đọc chứng thư ký bản sao {cert_path}: {e}"))?;
    let r_3i =
        f_222(&r_3j).ok_or("chứng thư ký bản sao không phải PEM")?;
    Ok(Some((r_3k, r_3i)))
}
fn f_222(r_3o: &str) -> Option<Vec<u8>> {
    use base64::Engine as _;
    let r_3m: String = r_3o
        .lines()
        .filter(|r_3n| !r_3n.starts_with("-----"))
        .collect::<Vec<_>>()
        .join("");
    base64::engine::general_purpose::STANDARD.decode(r_3m).ok()
}
pub fn f_2f1(r_3s: &[u8], r_3q: &rsa::RsaPrivateKey, r_3p: &[u8]) -> Result<Vec<u8>, String> {
    let r_3r = chrono::Utc::now().timestamp().max(0) as u64;
    let r_3t = digitext_sign_verify::pdf::prepare(r_3s, r_3p, r_3r)
        .map_err(|e| format!("chuẩn bị ký bản sao: {e}"))?;
    let r_3v = rsa::pkcs1v15::SigningKey::<rsa::sha2::Sha256>::new(r_3q.clone());
    let r_3u = r_3v
        .sign_prehash(&r_3t.tbs_digest)
        .map_err(|e| format!("ký digest bản sao: {e}"))?;
    digitext_sign_verify::pdf::finalize(&r_3t, &r_3u.to_bytes(), r_3p)
        .map_err(|e| format!("lắp CMS bản sao: {e}"))
}
