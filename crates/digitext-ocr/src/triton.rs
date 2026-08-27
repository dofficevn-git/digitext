// Copyright (c) 2026 DigiText. All rights reserved.
use crate::readiness::{Thj, Toj, Tol};
use std::future::Future;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
#[derive(Debug, Clone)]
pub struct HttpTritonProbe {
    host: String,
    port: u16,
    timeout: Duration,
}
impl HttpTritonProbe {
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(2);
    pub fn from_endpoint(endpoint: &str) -> Result<Self, Thj> {
        let r_3fv = endpoint
            .strip_prefix("http://")
            .ok_or_else(|| {
                Thj(
                    format!(
                        "Y2jhu4kgaOG7lyB0cuG7oyBodHRwOi8vIGNobyBwcm9iZSBu4buZaSBi4buZLCBuaOG6rW4g{endpoint:?}"
                    ),
                )
            })?;
        let r_3fu = r_3fv.split('/').next().unwrap_or(r_3fv);
        let (host, port) = r_3fu
            .rsplit_once(':')
            .ok_or_else(|| Thj(format!("dGhp4bq/dSBj4buVbmcgdHJvbmcg{endpoint:?}")))?;
        if host.is_empty() {
            return Err(Thj(format!("dGhp4bq/dSBob3N0IHRyb25nIA=={endpoint:?}")));
        }
        let port: u16 = port
            .parse()
            .map_err(|_| Thj(format!("Y+G7lW5nIGtow7RuZyBo4bujcCBs4buHOiA={port:?}")))?;
        Ok(HttpTritonProbe {
            host: host.to_owned(),
            port,
            timeout: Self::DEFAULT_TIMEOUT,
        })
    }
    async fn get_ok(&self, path: &str) -> Result<bool, Thj> {
        let addr = format!("{}:{}", self.host, self.port);
        let r_3g4 = async {
            let mut r_3g2 = TcpStream::connect(&addr)
                .await
                .map_err(|e| Thj(format!("connect {addr}: {e}")))?;
            let r_3fz = format!(
                "GET {path} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
                self.host
            );
            r_3g2
                .write_all(r_3fz.as_bytes())
                .await
                .map_err(|e| Thj(format!("write {path}: {e}")))?;
            let mut r_3fw = Vec::with_capacity(256);
            let mut r_3g3 = [0u8; 256];
            loop {
                let n = r_3g2
                    .read(&mut r_3g3)
                    .await
                    .map_err(|e| Thj(format!("read {path}: {e}")))?;
                if n == 0 {
                    break;
                }
                r_3fw.extend_from_slice(&r_3g3[..n]);
                if r_3fw.contains(&b'\n') || r_3fw.len() > 1024 {
                    break;
                }
            }
            let r_3fx = String::from_utf8_lossy(&r_3fw);
            let r_3g1 = r_3fx
                .lines()
                .next()
                .and_then(|r_3fy| r_3fy.split_whitespace().nth(1));
            Ok(r_3g1 == Some("200"))
        };
        match tokio::time::timeout(self.timeout, r_3g4).await {
            Ok(r_3g0) => r_3g0,
            Err(_) => Err(Thj(format!("dGltZW91dCBkw7Ig{path}"))),
        }
    }
}
impl Tol for HttpTritonProbe {
    fn probe(&self, r_3g6: &[String]) -> impl Future<Output = Result<Toj, Thj>> + Send {
        let models = r_3g6.to_vec();
        async move {
            let r_3g7 = self.get_ok("/v2/health/ready").await?;
            let mut r_3g5 = Vec::with_capacity(models.len());
            for m in &models {
                let ready = self
                    .get_ok(&format!("/v2/models/{m}/ready"))
                    .await
                    .unwrap_or(false);
                r_3g5.push((m.clone(), ready));
            }
            Ok(Toj {
                server_ready: r_3g7,
                models: r_3g5,
            })
        }
    }
}
