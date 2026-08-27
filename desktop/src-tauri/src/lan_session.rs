// Copyright (c) 2026 DigiText. All rights reserved.
use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::Engine;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use x25519_dalek::{EphemeralSecret, PublicKey};
const SESS_LABEL: &str = "digitext-lan-sessv1:";
const HKDF_INFO: &str = "digitext-lan-session-v1";
const REQ_INFO: &str = "digitext-lan-req-v1:";
const SID_LABEL: &str = "digitext-lan-sid-v1:";
const C_22: &[u8] = b"digitext-lan-code-v1-salt";
const PROOF_CLIENT: &str = "digitext-lan-codeproof-client-v1:";
const PROOF_SERVER: &str = "digitext-lan-codeproof-server-v1:";
const C_2: &[u8] = b"digitext-lan-frame-v1";
const C_49: u8 = 1;
const C_4A: u8 = 2;
pub const C_7I: usize = 64 * 1024;
pub const C_MH: usize = 16;
pub const C_7H: usize = C_7I + C_MH;
const C_1W: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
const C_1X: usize = 30;
const C_1Y: usize = 5;
const C_21: usize = 26;
const C_20: usize = 12;
#[derive(Debug, PartialEq, Eq)]
pub enum SessionError {
    BadEphemeralKey,
    BadSignature,
    Kdf,
    Frame,
    FrameTooBig,
    BadFraming,
    CounterOverflow,
}
impl std::fmt::Display for SessionError {
    fn fmt(&self, r_24h: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let r_24i = match self {
            Self::BadEphemeralKey => {
                "khoá phiên (X25519) không hợp lệ"
            }
            Self::BadSignature => "chữ ký bắt tay không hợp lệ",
            Self::Kdf => "dẫn xuất khoá phiên lỗi",
            Self::Frame => {
                "khung dữ liệu không giải mã được (sai khoá/thứ tự/bị sửa)"
            }
            Self::FrameTooBig => {
                "khung dữ liệu vượt kích thước cho phép"
            }
            Self::BadFraming => {
                "thân dữ liệu không đúng khuôn khung (nghi bị cắt ngắn)"
            }
            Self::CounterOverflow => "bộ đếm khung tràn",
        };
        r_24h.write_str(r_24i)
    }
}
impl std::error::Error for SessionError {}
fn f_t9() -> base64::engine::general_purpose::GeneralPurpose {
    base64::engine::general_purpose::URL_SAFE_NO_PAD
}
fn b64() -> base64::engine::general_purpose::GeneralPurpose {
    base64::engine::general_purpose::STANDARD
}
pub fn f_1u2() -> String {
    let mut b = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut b);
    b.iter().map(|x| format!("{x:02x}")).collect()
}
pub fn f_2os(r_24j: &str) -> bool {
    (8..=128).contains(&r_24j.len()) && r_24j.bytes().all(|b| b.is_ascii_hexdigit())
}
pub fn f_2dv(nonce: &str, epk: &str, spk: &str, device_id: &str) -> String {
    format!("{SESS_LABEL}{nonce}:{epk}:{spk}:{device_id}")
}
pub fn session_id(nonce: &str, epk: &str, spk: &str, device_id: &str) -> String {
    let r_24k = format!("{SID_LABEL}{}", f_2dv(nonce, epk, spk, device_id));
    blake3::hash(r_24k.as_bytes()).to_hex().to_string()
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T90 {
    pub device_id: String,
    pub pubkey: String,
    pub spk: String,
    pub sig: String,
}
#[derive(Clone)]
pub struct Session {
    pub sid: String,
    pub device_id: String,
    pub pubkey: String,
    prk: [u8; 32],
    info_base: String,
}
impl std::fmt::Debug for Session {
    fn fmt(&self, r_24l: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        r_24l
            .debug_struct("Session")
            .field("sid", &self.sid)
            .field("device_id", &self.device_id)
            .finish_non_exhaustive()
    }
}
impl Session {
    pub fn request_keys(&self, rn: &str) -> Result<Tiy, SessionError> {
        let r_24o = hkdf::Hkdf::<Sha256>::from_prk(&self.prk).map_err(|_| SessionError::Kdf)?;
        let r_24p = format!("{}{REQ_INFO}{rn}", self.info_base);
        let mut r_24q = [0u8; 72];
        r_24o
            .expand(r_24p.as_bytes(), &mut r_24q)
            .map_err(|_| SessionError::Kdf)?;
        let mut r_24m = [0u8; 32];
        let mut r_24r = [0u8; 32];
        let mut r_24n = [0u8; 4];
        let mut r_24s = [0u8; 4];
        r_24m.copy_from_slice(&r_24q[0..32]);
        r_24n.copy_from_slice(&r_24q[32..36]);
        r_24r.copy_from_slice(&r_24q[36..68]);
        r_24s.copy_from_slice(&r_24q[68..72]);
        Ok(Tiy {
            c2s: T4r {
                key: r_24m,
                prefix: r_24n,
                dir: C_49,
            },
            s2c: T4r {
                key: r_24r,
                prefix: r_24s,
                dir: C_4A,
            },
        })
    }
}
pub struct ClientHello {
    secret: EphemeralSecret,
    epk: String,
    nonce: String,
}
impl ClientHello {
    pub fn new() -> Self {
        let secret = EphemeralSecret::random_from_rng(rand::rngs::OsRng);
        let epk = f_t9().encode(PublicKey::from(&secret).to_bytes());
        Self {
            secret,
            epk,
            nonce: f_1u2(),
        }
    }
    pub fn epk(&self) -> &str {
        &self.epk
    }
    pub fn nonce(&self) -> &str {
        &self.nonce
    }
    pub fn finish(self, r_24u: &T90) -> Result<Session, SessionError> {
        if r_24u.device_id.trim().is_empty() {
            return Err(SessionError::BadSignature);
        }
        let r_24t = f_2dv(&self.nonce, &self.epk, &r_24u.spk, &r_24u.device_id);
        if !f_2px(&r_24u.pubkey, r_24t.as_bytes(), &r_24u.sig) {
            return Err(SessionError::BadSignature);
        }
        let spk = f_13m(&r_24u.spk).ok_or(SessionError::BadEphemeralKey)?;
        let r_24v = self.secret.diffie_hellman(&spk);
        Ok(f_14q(
            r_24v.as_bytes(),
            &self.nonce,
            &self.epk,
            &r_24u.spk,
            &r_24u.device_id,
            &r_24u.pubkey,
        ))
    }
}
impl Default for ClientHello {
    fn default() -> Self {
        Self::new()
    }
}
pub fn f_29j(
    r_24z: &SigningKey,
    device_id: &str,
    r_24w: &str,
    nonce: &str,
) -> Result<(T90, Session), SessionError> {
    let epk = f_13m(r_24w).ok_or(SessionError::BadEphemeralKey)?;
    let secret = EphemeralSecret::random_from_rng(rand::rngs::OsRng);
    let r_250 = f_t9().encode(PublicKey::from(&secret).to_bytes());
    let pubkey = b64().encode(r_24z.verifying_key().to_bytes());
    let r_24x = f_2dv(nonce, r_24w, &r_250, device_id);
    let sig = b64().encode(r_24z.sign(r_24x.as_bytes()).to_bytes());
    let r_24y = secret.diffie_hellman(&epk);
    let session = f_14q(r_24y.as_bytes(), nonce, r_24w, &r_250, device_id, &pubkey);
    Ok((
        T90 {
            device_id: device_id.to_string(),
            pubkey,
            spk: r_250,
            sig,
        },
        session,
    ))
}
fn f_14q(
    r_252: &[u8; 32],
    nonce: &str,
    epk: &str,
    spk: &str,
    device_id: &str,
    pubkey: &str,
) -> Session {
    let (r_251, _) = hkdf::Hkdf::<Sha256>::extract(Some(nonce.as_bytes()), r_252);
    let mut prk = [0u8; 32];
    prk.copy_from_slice(&r_251);
    Session {
        sid: session_id(nonce, epk, spk, device_id),
        device_id: device_id.to_string(),
        pubkey: pubkey.to_string(),
        prk,
        info_base: format!("{HKDF_INFO}{device_id}{epk}{spk}"),
    }
}
fn f_13m(r_254: &str) -> Option<PublicKey> {
    let r_255 = f_t9().decode(r_254).ok()?;
    let r_253: [u8; 32] = r_255.try_into().ok()?;
    Some(PublicKey::from(r_253))
}
pub fn f_2ow(r_256: &str) -> bool {
    r_256.len() == 43 && f_13m(r_256).is_some()
}
fn f_2px(pubkey_b64: &str, r_257: &[u8], r_25b: &str) -> bool {
    let Ok(r_259) = b64().decode(pubkey_b64) else {
        return false;
    };
    let Ok(r_258): Result<[u8; 32], _> = r_259.try_into() else {
        return false;
    };
    let Ok(r_25d) = VerifyingKey::from_bytes(&r_258) else {
        return false;
    };
    let Ok(r_25c) = b64().decode(r_25b) else {
        return false;
    };
    let Ok(r_25a): Result<[u8; 64], _> = r_25c.try_into() else {
        return false;
    };
    r_25d.verify(r_257, &Signature::from_bytes(&r_25a)).is_ok()
}
struct T4r {
    key: [u8; 32],
    prefix: [u8; 4],
    dir: u8,
}
pub struct Tiy {
    c2s: T4r,
    s2c: T4r,
}
impl Tiy {
    pub fn client(self) -> (Sealer, Opener) {
        (Sealer::new(self.c2s), Opener::new(self.s2c))
    }
    pub fn server(self) -> (Opener, Sealer) {
        (Opener::new(self.c2s), Sealer::new(self.s2c))
    }
}
fn f_1dp(prefix: &[u8; 4], counter: u64) -> [u8; 12] {
    let mut r_25e = [0u8; 12];
    r_25e[..4].copy_from_slice(prefix);
    r_25e[4..].copy_from_slice(&counter.to_be_bytes());
    r_25e
}
fn f_1dm(dir: u8, r_25f: u64, r_25g: bool) -> Vec<u8> {
    let mut a = Vec::with_capacity(C_2.len() + 10);
    a.extend_from_slice(C_2);
    a.push(dir);
    a.extend_from_slice(&r_25f.to_be_bytes());
    a.push(u8::from(r_25g));
    a
}
pub struct Sealer {
    cipher: Aes256Gcm,
    prefix: [u8; 4],
    dir: u8,
    counter: u64,
}
impl Sealer {
    fn new(r_25h: T4r) -> Self {
        Self {
            cipher: Aes256Gcm::new(&r_25h.key.into()),
            prefix: r_25h.prefix,
            dir: r_25h.dir,
            counter: 0,
        }
    }
    pub fn seal(&mut self, r_25j: u64, r_25k: bool, r_25l: &[u8]) -> Result<Vec<u8>, SessionError> {
        if r_25l.len() > C_7I {
            return Err(SessionError::FrameTooBig);
        }
        let r_25i = self.counter;
        self.counter = r_25i.checked_add(1).ok_or(SessionError::CounterOverflow)?;
        self.cipher
            .encrypt(
                Nonce::from_slice(&f_1dp(&self.prefix, r_25i)),
                Payload {
                    msg: r_25l,
                    aad: &f_1dm(self.dir, r_25j, r_25k),
                },
            )
            .map_err(|_| SessionError::Frame)
    }
}
pub struct Opener {
    cipher: Aes256Gcm,
    prefix: [u8; 4],
    dir: u8,
    counter: u64,
}
impl Opener {
    fn new(r_25m: T4r) -> Self {
        Self {
            cipher: Aes256Gcm::new(&r_25m.key.into()),
            prefix: r_25m.prefix,
            dir: r_25m.dir,
            counter: 0,
        }
    }
    pub fn open(
        &mut self,
        r_25o: u64,
        r_25p: bool,
        cipher: &[u8],
    ) -> Result<Vec<u8>, SessionError> {
        let r_25n = self.counter;
        self.counter = r_25n.checked_add(1).ok_or(SessionError::CounterOverflow)?;
        self.cipher
            .decrypt(
                Nonce::from_slice(&f_1dp(&self.prefix, r_25n)),
                Payload {
                    msg: cipher,
                    aad: &f_1dm(self.dir, r_25o, r_25p),
                },
            )
            .map_err(|_| SessionError::Frame)
    }
}
pub fn f_1do(r_25q: u64) -> u64 {
    if r_25q == 0 {
        1
    } else {
        r_25q.div_ceil(C_7I as u64)
    }
}
pub fn f_zd(r_25r: u64) -> u64 {
    r_25r + f_1do(r_25r) * C_MH as u64
}
pub fn f_1md(r_25s: u64) -> u64 {
    f_1do(r_25s) - 1
}
pub fn f_ze(frame: u64) -> u64 {
    frame * C_7H as u64
}
pub fn f_1dn(r_25u: u64, frame: u64) -> usize {
    let r_25t = f_1md(r_25u);
    if frame < r_25t {
        C_7H
    } else {
        (r_25u - r_25t * C_7I as u64) as usize + C_MH
    }
}
pub fn f_2cl(sealer: &mut Sealer, r_25y: &[u8]) -> Result<Vec<u8>, SessionError> {
    let r_260 = r_25y.len() as u64;
    let r_25x = f_1md(r_260);
    let mut out = Vec::with_capacity(f_zd(r_260) as usize);
    for r_25w in 0..=r_25x {
        let r_25v = r_25w as usize * C_7I;
        let r_25z = ((r_25w as usize + 1) * C_7I).min(r_25y.len());
        out.extend_from_slice(&sealer.seal(r_25w, r_25w == r_25x, &r_25y[r_25v..r_25z])?);
    }
    Ok(out)
}
pub fn f_1w6(r_264: &mut Opener, cipher: &[u8]) -> Result<Vec<u8>, SessionError> {
    if cipher.len() < C_MH {
        return Err(SessionError::BadFraming);
    }
    let r_263 = cipher.len().div_ceil(C_7H);
    let mut out = Vec::with_capacity(cipher.len());
    for r_262 in 0..r_263 {
        let r_261 = r_262 * C_7H;
        let r_265 = ((r_262 + 1) * C_7H).min(cipher.len());
        if r_265 - r_261 < C_MH {
            return Err(SessionError::BadFraming);
        }
        out.extend_from_slice(&r_264.open(
            r_262 as u64,
            r_262 + 1 == r_263,
            &cipher[r_261..r_265],
        )?);
    }
    Ok(out)
}
pub struct SealingReader<R: std::io::Read> {
    inner: R,
    sealer: Sealer,
    next_frame: u64,
    last_frame: u64,
    out: Vec<u8>,
    out_pos: usize,
    done: bool,
}
impl<R: std::io::Read> SealingReader<R> {
    pub fn new(inner: R, sealer: Sealer, r_266: u64, last_frame: u64) -> Self {
        Self {
            inner,
            sealer,
            next_frame: r_266,
            last_frame,
            out: Vec::new(),
            out_pos: 0,
            done: false,
        }
    }
    fn fill(&mut self) -> std::io::Result<()> {
        let mut r_26a = vec![0u8; C_7I];
        let mut r_267 = 0usize;
        while r_267 < C_7I {
            match self.inner.read(&mut r_26a[r_267..])? {
                0 => break,
                r_269 => r_267 += r_269,
            }
        }
        r_26a.truncate(r_267);
        let r_268 = self.next_frame >= self.last_frame;
        self.out = self
            .sealer
            .seal(self.next_frame, r_268, &r_26a)
            .map_err(std::io::Error::other)?;
        self.out_pos = 0;
        self.next_frame += 1;
        if r_268 {
            self.done = true;
        }
        Ok(())
    }
}
impl<R: std::io::Read> std::io::Read for SealingReader<R> {
    fn read(&mut self, r_26b: &mut [u8]) -> std::io::Result<usize> {
        if self.out_pos >= self.out.len() {
            if self.done {
                return Ok(0);
            }
            self.fill()?;
        }
        let r_26c = r_26b.len().min(self.out.len() - self.out_pos);
        r_26b[..r_26c].copy_from_slice(&self.out[self.out_pos..self.out_pos + r_26c]);
        self.out_pos += r_26c;
        Ok(r_26c)
    }
}
pub fn code_key(code: &str) -> Result<[u8; 32], SessionError> {
    let mut out = [0u8; 32];
    Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(19456, 3, 1, Some(32)).map_err(|_| SessionError::Kdf)?,
    )
    .hash_password_into(code.as_bytes(), C_22, &mut out)
    .map_err(|_| SessionError::Kdf)?;
    Ok(out)
}
pub fn f_104(code_key: &[u8; 32], sid: &str) -> [u8; 32] {
    *blake3::keyed_hash(code_key, format!("{PROOF_CLIENT}{sid}").as_bytes()).as_bytes()
}
pub fn f_2dp(code_key: &[u8; 32], sid: &str) -> [u8; 32] {
    *blake3::keyed_hash(code_key, format!("{PROOF_SERVER}{sid}").as_bytes()).as_bytes()
}
pub fn f_24d(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.ct_eq(b).into()
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeStrength {
    Empty,
    Weak,
    Strong,
}
fn f_1v3(code: &str) -> String {
    code.chars()
        .filter(|c| !matches!(c, '-' | ' ' | '_' | '\t'))
        .flat_map(char::to_uppercase)
        .collect()
}
pub fn f_2ft(code: &str) -> CodeStrength {
    let r_26f = f_1v3(code.trim());
    if r_26f.is_empty() {
        return CodeStrength::Empty;
    }
    let r_26d = r_26f.bytes().all(|b| C_1W.contains(&b));
    let r_26e = r_26f
        .bytes()
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    if r_26f.len() >= C_21 && r_26d && r_26e >= C_20 {
        CodeStrength::Strong
    } else {
        CodeStrength::Weak
    }
}
pub fn f_x4(code: &str) -> Option<String> {
    if f_2ft(code) != CodeStrength::Strong {
        return None;
    }
    let r_26g = f_1v3(code.trim());
    Some(
        r_26g
            .as_bytes()
            .chunks(C_1Y)
            .map(|c| String::from_utf8_lossy(c).into_owned())
            .collect::<Vec<_>>()
            .join("-"),
    )
}
pub fn f_1e9() -> String {
    let r_26k = C_1W.len();
    let r_26j = 256 - (256 % r_26k);
    let mut r_26i = Vec::with_capacity(C_1X);
    let mut r_26h = [0u8; 64];
    while r_26i.len() < C_1X {
        rand::rngs::OsRng.fill_bytes(&mut r_26h);
        for b in r_26h {
            if r_26i.len() == C_1X {
                break;
            }
            if (b as usize) < r_26j {
                r_26i.push(C_1W[b as usize % r_26k]);
            }
        }
    }
    r_26i
        .chunks(C_1Y)
        .map(|c| String::from_utf8_lossy(c).into_owned())
        .collect::<Vec<_>>()
        .join("-")
}
pub fn device_fingerprint(pubkey_b64: &str) -> String {
    let r_26n = blake3::hash(format!("digitext-lan-fp-v1:{pubkey_b64}").as_bytes());
    let r_26l = &r_26n.as_bytes()[..10];
    let r_26m: Vec<u8> = r_26l
        .iter()
        .flat_map(|b| {
            [
                C_1W[(b >> 4) as usize % C_1W.len()],
                C_1W[(b & 0x0f) as usize % C_1W.len()],
            ]
        })
        .take(16)
        .collect();
    r_26m
        .chunks(4)
        .map(|c| String::from_utf8_lossy(c).into_owned())
        .collect::<Vec<_>>()
        .join("-")
}
