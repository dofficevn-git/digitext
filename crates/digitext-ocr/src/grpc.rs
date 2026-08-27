// Copyright (c) 2026 DigiText. All rights reserved.
use crate::error::WorkerError;
use crate::infer::{Tile, To6, Tok};
use crate::readiness::{Thj, Toj, Tol};
use opentelemetry::propagation::Injector;
use std::future::Future;
use tonic::metadata::{MetadataKey, MetadataMap, MetadataValue};
use tonic::transport::Channel;
use tracing_opentelemetry::OpenTelemetrySpanExt as _;
#[allow(clippy::all, clippy::pedantic, clippy::nursery)]
pub mod pb {
    tonic::include_proto!("inference");
}
use pb::grpc_inference_service_client::GrpcInferenceServiceClient;
const C_9L: &str = "tile";
const OUT_BOXES: &str = "boxes";
const OUT_TEXTS: &str = "texts";
const OUT_CONF: &str = "confidences";
fn f_1v4(endpoint: &str) -> Result<String, WorkerError> {
    let ep = endpoint.trim();
    if let Some(rest) = ep.strip_prefix("grpc://") {
        Ok(format!("http://{rest}"))
    } else if ep.starts_with("http://") || ep.starts_with("https://") {
        Ok(ep.to_owned())
    } else if ep.is_empty() {
        Err(WorkerError::ConfigInvalid(
            "endpoint gRPC rỗng".to_owned(),
        ))
    } else {
        Ok(format!("http://{ep}"))
    }
}
fn f_1mj(endpoint: &str) -> Result<Channel, WorkerError> {
    let r_388 = f_1v4(endpoint)?;
    let channel = Channel::from_shared(r_388)
        .map_err(|e| {
            WorkerError::ConfigInvalid(format!("endpoint gRPC không hợp lệ: {e}"))
        })?
        .connect_lazy();
    Ok(channel)
}
fn f_1qz(r_38a: &tonic::Status) -> WorkerError {
    use tonic::Code;
    let r_389 = format!("gRPC {:?}: {}", r_38a.code(), r_38a.message());
    match r_38a.code() {
        Code::ResourceExhausted => WorkerError::GpuOom(r_389),
        Code::DeadlineExceeded => WorkerError::Timeout(r_389),
        Code::Unavailable | Code::Internal | Code::Unknown | Code::Aborted => {
            WorkerError::TritonUnavailable(r_389)
        }
        Code::InvalidArgument | Code::OutOfRange | Code::FailedPrecondition => {
            WorkerError::CorruptInput(r_389)
        }
        Code::Unimplemented | Code::NotFound => WorkerError::ConfigInvalid(r_389),
        _ => WorkerError::TritonUnavailable(r_389),
    }
}
fn f_1z0(r_38b: &[u8]) -> Vec<f32> {
    r_38b
        .as_chunks::<4>()
        .0
        .iter()
        .copied()
        .map(f32::from_le_bytes)
        .collect()
}
fn f_1dl(r_38c: &[f32]) -> Vec<[f32; 4]> {
    r_38c.as_chunks::<4>().0.to_vec()
}
fn f_1zz(r_38d: &[u8]) -> Result<Vec<String>, WorkerError> {
    let mut r_38h = Vec::new();
    let mut r_38f = 0usize;
    while r_38f + 4 <= r_38d.len() {
        let r_38g = u32::from_le_bytes([
            r_38d[r_38f],
            r_38d[r_38f + 1],
            r_38d[r_38f + 2],
            r_38d[r_38f + 3],
        ]) as usize;
        r_38f += 4;
        let r_38e = r_38f
            .checked_add(r_38g)
            .filter(|&e| e <= r_38d.len())
            .ok_or_else(|| {
                WorkerError::CorruptInput(
                    "tensor STRING: độ dài vượt buffer".to_owned(),
                )
            })?;
        r_38h.push(String::from_utf8_lossy(&r_38d[r_38f..r_38e]).into_owned());
        r_38f = r_38e;
    }
    Ok(r_38h)
}
struct Tdd<'a>(&'a mut MetadataMap);
impl Injector for Tdd<'_> {
    fn set(&mut self, key: &str, r_38k: String) {
        if let (Ok(r_38i), Ok(r_38j)) = (
            MetadataKey::from_bytes(key.as_bytes()),
            MetadataValue::try_from(r_38k.as_str()),
        ) {
            self.0.insert(r_38i, r_38j);
        }
    }
}
fn f_1ia(r_38m: &mut MetadataMap) {
    let r_38l = tracing::Span::current().context();
    opentelemetry::global::get_text_map_propagator(|r_38n| {
        r_38n.inject_context(&r_38l, &mut Tdd(r_38m));
    });
}
fn f_vf(model_name: &str, tile: &Tile) -> pb::ModelInferRequest {
    use pb::model_infer_request::{InferInputTensor, InferRequestedOutputTensor};
    pb::ModelInferRequest {
        model_name: model_name.to_owned(),
        inputs: vec![InferInputTensor {
            name: C_9L.to_owned(),
            datatype: "UINT8".to_owned(),
            shape: vec![i64::from(tile.height), i64::from(tile.width), 3],
        }],
        outputs: [OUT_BOXES, OUT_TEXTS, OUT_CONF]
            .into_iter()
            .map(|r_38o| InferRequestedOutputTensor {
                name: r_38o.to_owned(),
            })
            .collect(),
        raw_input_contents: vec![tile.pixels.clone()],
        ..Default::default()
    }
}
fn f_1z5(r_38s: pb::ModelInferResponse) -> Result<To6, WorkerError> {
    let (mut boxes, mut texts, mut confidences) = (Vec::new(), Vec::new(), Vec::new());
    for (r_38p, r_38q) in r_38s.outputs.iter().enumerate() {
        let r_38r = r_38s
            .raw_output_contents
            .get(r_38p)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        match r_38q.name.as_str() {
            OUT_BOXES => boxes = f_1dl(&f_1z0(r_38r)),
            OUT_CONF => confidences = f_1z0(r_38r),
            OUT_TEXTS => texts = f_1zz(r_38r)?,
            _ => {}
        }
    }
    if boxes.len() != texts.len() || texts.len() != confidences.len() {
        return Err(
            WorkerError::CorruptInput(
                format!(
                    "output Triton lệch độ dài: {} box / {} text / {} conf (mỗi detection phải có đúng một bộ)",
                    boxes.len(), texts.len(), confidences.len()
                ),
            ),
        );
    }
    Ok(To6 {
        boxes,
        texts,
        confidences,
    })
}
#[derive(Clone)]
pub struct GrpcTritonInfer {
    channel: Channel,
    model_name: String,
}
impl GrpcTritonInfer {
    pub fn connect_lazy(endpoint: &str, model_name: &str) -> Result<Self, WorkerError> {
        Ok(GrpcTritonInfer {
            channel: f_1mj(endpoint)?,
            model_name: model_name.to_owned(),
        })
    }
}
impl Tok for GrpcTritonInfer {
    fn infer_tile(
        &self,
        tile: &Tile,
        _batch: usize,
    ) -> impl Future<Output = Result<To6, WorkerError>> + Send {
        let mut r_38t = GrpcInferenceServiceClient::new(self.channel.clone());
        let mut r_38u = tonic::Request::new(f_vf(&self.model_name, tile));
        f_1ia(r_38u.metadata_mut());
        async move {
            let r_38v = r_38t
                .model_infer(r_38u)
                .await
                .map_err(|r_38w| f_1qz(&r_38w))?;
            f_1z5(r_38v.into_inner())
        }
    }
}
#[derive(Clone)]
pub struct GrpcTritonProbe {
    channel: Channel,
}
impl GrpcTritonProbe {
    pub fn connect_lazy(endpoint: &str) -> Result<Self, WorkerError> {
        Ok(GrpcTritonProbe {
            channel: f_1mj(endpoint)?,
        })
    }
}
impl Tol for GrpcTritonProbe {
    fn probe(&self, r_393: &[String]) -> impl Future<Output = Result<Toj, Thj>> + Send {
        let mut r_38x = GrpcInferenceServiceClient::new(self.channel.clone());
        let r_38y = r_393.to_vec();
        async move {
            let r_395 = r_38x
                .server_ready(pb::ServerReadyRequest {})
                .await
                .map_err(|r_394| Thj(format!("ServerReady: {}", r_394.message())))?
                .into_inner()
                .ready;
            let mut r_390 = Vec::with_capacity(r_38y.len());
            for r_38z in &r_38y {
                let r_392 = r_38x
                    .model_ready(pb::ModelReadyRequest {
                        name: r_38z.clone(),
                        version: String::new(),
                    })
                    .await
                    .map(|r_391| r_391.into_inner().ready)
                    .unwrap_or(false);
                r_390.push((r_38z.clone(), r_392));
            }
            Ok(Toj {
                server_ready: r_395,
                models: r_390,
            })
        }
    }
}
