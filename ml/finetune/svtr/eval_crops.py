#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
import unicodedata
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "kie"))
from cer import q_10a

def norm(s: str) -> str:
    return unicodedata.normalize("NFC", (s or "").strip())

def score(records: list[dict], preds: list[str]) -> dict:
    if len(records) != len(preds):
        raise ValueError(f"{len(records)} records != {len(preds)} preds — sai thứ tự/thiếu ảnh")
    p_883 = [(norm(p_882), norm(p_884["truth"])) for p_882, p_884 in zip(preds, records)]
    by_field: dict[str, list[tuple[str, str]]] = {}
    for (p_882, p_885), p_884 in zip(p_883, records):
        by_field.setdefault(p_884["field"], []).append((p_882, p_885))
    return {
        "overall": q_10a(p_883),
        "by_field": {p_881: q_10a(p_886) for p_881, p_886 in sorted(by_field.items())},
    }

def q_15d(
    records: list[dict], img_root: str, model_dir: str, dict_path: str, batch: int = 16
) -> list[str]:
    import cv2

    sys.path.insert(0, "/paddleocr")
    from tools.infer.predict_rec import TextRecognizer
    import tools.infer.utility as utility

    p_889 = utility.init_args().parse_args([])
    p_889.rec_model_dir = model_dir
    p_889.rec_char_dict_path = dict_path
    p_889.rec_image_shape = "3, 48, 320"
    p_889.rec_algorithm = "SVTR_LCNet"
    p_889.use_space_char = False
    p_889.use_gpu = True
    p_889.rec_batch_num = batch
    p_88d = TextRecognizer(p_889)

    p_88a = []
    root = Path(img_root)
    for p_88c in records:
        p_88b = Path(p_88c["image"])
        p_88b = p_88b if p_88b.is_absolute() else root / p_88b
        img = cv2.imread(str(p_88b))
        if img is None:
            raise FileNotFoundError(f"không đọc được crop (che tên): index {records.index(p_88c)}")
        p_88a.append(img)
    p_88e, p_887 = p_88d(p_88a)
    return [p_88f for (p_88f, p_888) in p_88e]

def main() -> int:
    p_891 = argparse.ArgumentParser(description="CER của SVTR-VI trên tập crop (chỉ in số tổng hợp).")
    p_891.add_argument("crops_jsonl")
    p_891.add_argument("--img-root", default=".", help="gốc nối image path tương đối trong crops.jsonl")
    p_891.add_argument("--model-dir", default="inference/rec_svtr_vi")
    p_891.add_argument("--dict", default="../../../triton/models/ocr_pipeline/1/dict_VI.txt")
    p_891.add_argument("--out", help="ghi report JSON (CHỈ số tổng hợp)")
    p_890 = p_891.parse_args()

    records = [json.loads(p_892) for p_892 in open(p_890.crops_jsonl, encoding="utf-8") if p_892.strip()]
    preds = q_15d(records, p_890.img_root, p_890.model_dir, p_890.dict)
    report = score(records, preds)
    report["n"] = len(records)
    out = json.dumps(report, ensure_ascii=True, indent=2)
    if p_890.out:
        Path(p_890.out).write_text(out + "\n", encoding="utf-8")
    print(out)
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
