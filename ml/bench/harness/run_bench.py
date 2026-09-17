#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import sys

def _load_manifest(path: str) -> list:
    p_13c = []
    with open(path, encoding="utf-8") as p_139:
        for p_13b, p_13a in enumerate(p_139, 1):
            p_13a = p_13a.strip()
            if p_13a:
                try:
                    p_13c.append(json.loads(p_13a))
                except json.JSONDecodeError as e:
                    raise SystemExit(f"❌ manifest dòng {p_13b}: {e}")
    return p_13c

def _as_text(t) -> str:
    if isinstance(t, (bytes, bytearray)):
        return t.decode("utf-8", "replace")
    return str(t)

def assemble_line(texts, confs) -> tuple[str, float]:
    p_13f = [p_13e for p_13e in (_as_text(t) for t in (texts if texts is not None else [])) if p_13e]
    hyp = " ".join(p_13f)
    p_13d = list(confs) if confs is not None else []
    conf = float(sum(p_13d) / len(p_13d)) if p_13d else 1.0
    return hyp, conf

def _infer_line(client, model: str, image):
    import numpy as np
    import tritonclient.grpc as grpcclient

    tile = np.ascontiguousarray(image, dtype=np.uint8)
    p_140 = grpcclient.InferInput("tile", list(tile.shape), "UINT8")
    p_140.set_data_from_numpy(tile)
    p_142 = [grpcclient.InferRequestedOutput(p_141) for p_141 in ("boxes", "texts", "confidences")]
    p_143 = client.infer(model_name=model, inputs=[p_140], outputs=p_142)
    return assemble_line(p_143.as_numpy("texts"), p_143.as_numpy("confidences"))


# --- backend HTTP (dịch vụ :5002 đang chạy sản xuất) ------------------------
# Cùng hợp đồng với ocr/ocr_routes.py: POST /api/ocr/tile {"image": <b64 PNG>}
# → {"boxes","texts","confidences"}. Lấy nguyên cách gọi ở
# ml/finetune/vietocr/build_rec_dataset.py để hai chỗ không lệch nhau.

def _infer_line_http(endpoint: str, api_key: str, image, timeout: int = 120):
    import base64
    import urllib.request

    import cv2

    ok, png = cv2.imencode(".png", image)
    if not ok:
        raise RuntimeError("không mã hoá được ảnh sang PNG")
    body = json.dumps({"image": base64.b64encode(png.tobytes()).decode()}).encode()
    headers = {"Content-Type": "application/json"}
    if api_key:
        headers["X-API-Key"] = api_key
    req = urllib.request.Request(
        endpoint.rstrip("/") + "/api/ocr/tile", data=body, headers=headers, method="POST"
    )
    with urllib.request.urlopen(req, timeout=timeout) as r:
        p_14a = json.loads(r.read().decode())
    return assemble_line(p_14a.get("texts"), p_14a.get("confidences"))


def _http_ready(endpoint: str, timeout: int = 10) -> bool:
    import urllib.request

    try:
        with urllib.request.urlopen(endpoint.rstrip("/") + "/api/ocr/health", timeout=timeout) as r:
            return r.status == 200
    except OSError:
        return False


def make_infer(backend: str, triton_url: str, model: str, endpoint: str, api_key: str, timeout: int):
    """Trả về hàm `infer(image) -> (hyp, conf)` đã gắn sẵn client. loadtest.py dùng chung."""
    if backend == "triton":
        import tritonclient.grpc as grpcclient

        client = grpcclient.InferenceServerClient(url=triton_url)
        if not client.is_server_ready():
            raise SystemExit(f"❌ Triton chưa sẵn sàng: {triton_url}")
        return lambda image: _infer_line(client, model, image)
    if backend == "http":
        if not _http_ready(endpoint):
            raise SystemExit(f"❌ dịch vụ OCR HTTP chưa sẵn sàng: {endpoint}/api/ocr/health")
        return lambda image: _infer_line_http(endpoint, api_key, image, timeout)
    raise SystemExit(f"❌ backend không hợp lệ: {backend}")


def add_backend_args(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--backend", choices=("http", "triton"), default="http",
                        help="http = dịch vụ :5002 (mặc định, đang chạy sản xuất); triton = BLS ocr_pipeline")
    parser.add_argument("--endpoint", default=os.environ.get("OCR_ENDPOINT", "http://127.0.0.1:5002"),
                        help="gốc URL dịch vụ OCR HTTP (env OCR_ENDPOINT)")
    parser.add_argument("--api-key", default=os.environ.get("OCR_API_KEY", ""),
                        help="X-API-Key của dịch vụ HTTP (env OCR_API_KEY)")
    parser.add_argument("--timeout", type=int, default=120, help="giây chờ mỗi ảnh (HTTP)")
    parser.add_argument("--triton", default="localhost:8001", help="địa chỉ Triton gRPC")
    parser.add_argument("--model", default="ocr_pipeline", help="tên model BLS (Triton)")


def run(manifest: list, infer, out_path: str) -> int:
    import cv2

    p_145 = 0
    p_144 = 0
    with open(out_path, "w", encoding="utf-8") as out:
        for p_146 in manifest:
            image = cv2.imread(p_146["image"])
            if image is None:
                p_144 += 1
                print(f"⚠ bỏ {p_146['id']}: không đọc được ảnh {p_146['image']}", file=sys.stderr)
                continue
            hyp, conf = infer(image)
            out.write(
                json.dumps({"id": p_146["id"], "hyp": hyp, "conf": conf}, ensure_ascii=False) + "\n"
            )
            p_145 += 1
    print(f"✅ {out_path}: {p_145} dòng" + (f" · ⚠ bỏ {p_144} (thiếu ảnh)" if p_144 else ""))
    return 0

def main(argv=None) -> int:
    p_147 = argparse.ArgumentParser(description="Chạy OCR golden → pred.jsonl (HTTP :5002 hoặc Triton)")
    p_147.add_argument("--manifest", required=True, help="manifest.jsonl {id, image, stratum}")
    p_147.add_argument("--out", required=True, help="pred.jsonl đầu ra")
    add_backend_args(p_147)
    p_148 = p_147.parse_args(argv)
    if not os.path.exists(p_148.manifest):
        raise SystemExit(f"❌ không thấy manifest: {p_148.manifest}")
    infer = make_infer(p_148.backend, p_148.triton, p_148.model, p_148.endpoint, p_148.api_key,
                       p_148.timeout)
    return run(_load_manifest(p_148.manifest), infer, p_148.out)

if __name__ == "__main__":
    sys.exit(main())
