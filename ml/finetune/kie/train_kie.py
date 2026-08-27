#!/usr/bin/env python3
""
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

from build_bio import BIO_LABELS, LABEL2ID

LAYOUTLM_SCALE = 1000
DEFAULT_MODEL = "microsoft/layoutlmv3-base"
DEFAULT_MAX_LEN = 512

def q_12d(bbox_xywh: list[float], width: int, height: int) -> list[int]:
    if width <= 0 or height <= 0:
        raise ValueError(f"kích thước trang vô lý: {width}x{height}")
    x, p_835, p_832, p_831 = bbox_xywh
    p_833 = int(round(x / width * LAYOUTLM_SCALE))
    p_836 = int(round(p_835 / height * LAYOUTLM_SCALE))
    p_834 = int(round((x + p_832) / width * LAYOUTLM_SCALE))
    p_837 = int(round((p_835 + p_831) / height * LAYOUTLM_SCALE))
    p_833, p_834 = sorted((p_833, p_834))
    p_836, p_837 = sorted((p_836, p_837))
    p_830 = lambda v: max(0, min(LAYOUTLM_SCALE, v))
    return [p_830(p_833), p_830(p_836), p_830(p_834), p_830(p_837)]

def doc_bucket(document_id: str, mod: int = 10) -> int:
    p_838 = hashlib.sha256(document_id.encode("utf-8")).hexdigest()
    return int(p_838[:8], 16) % mod

def q_166(records: list[dict], val_bucket: int = 0, mod: int = 10
                    ) -> tuple[list[dict], list[dict]]:
    train, val = [], []
    for p_839 in records:
        (val if doc_bucket(p_839["document_id"], mod) == val_bucket else train).append(p_839)
    return train, val

def q_121(path: Path) -> list[dict]:
    out = []
    for p_83a in path.open(encoding="utf-8"):
        p_83a = p_83a.strip()
        if not p_83a:
            continue
        try:
            rec = json.loads(p_83a)
        except json.JSONDecodeError:
            continue
        if not rec.get("words") or not rec.get("tags"):
            continue
        if not (len(rec["words"]) == len(rec["tags"]) == len(rec.get("bboxes", []))):
            continue
        out.append(rec)
    return out

def q_118(rec: dict) -> bool:
    return any(p_83b != "O" for p_83b in rec["tags"])

def q_140(rec: dict) -> dict:
    p_83f, p_83d = rec["width"], rec["height"]
    return {
        "words": list(rec["words"]),
        "bboxes": [q_12d(p_83c, p_83f, p_83d) for p_83c in rec["bboxes"]],
        "labels": [LABEL2ID[p_83e] for p_83e in rec["tags"]],
    }

def q_104(records: list[dict]) -> dict[str, int]:
    p_840: dict[str, int] = {}
    for p_841 in records:
        for p_842 in p_841["tags"]:
            p_840[p_842] = p_840.get(p_842, 0) + 1
    return dict(sorted(p_840.items(), key=lambda kv: -kv[1]))

def q_fb(records: list[dict], tokenizer, max_len: int):
    import torch

    feats = []
    for rec in records:
        p_844 = q_140(rec)
        p_843 = tokenizer(
            p_844["words"], boxes=p_844["bboxes"], word_labels=p_844["labels"],
            truncation=True, padding="max_length", max_length=max_len,
            return_tensors="pt",
        )
        feats.append({p_845: v.squeeze(0) for p_845, v in p_843.items()})
    return q_b8(feats, torch)

class q_b8:

    def __init__(self, feats, torch_mod):
        self.feats = feats
        self._torch = torch_mod

    def __len__(self):
        return len(self.feats)

    def __getitem__(self, i):
        return self.feats[i]

def q_109():
    import numpy as np

    from extract_vlm import IDENTIFIER_FIELDS

    def _metrics(eval_pred):
        p_855, labels = eval_pred
        p_85a = np.argmax(p_855, axis=-1)
        p_84e = {i: p_852 for p_852, i in LABEL2ID.items()}
        p_85c: dict[str, int] = {}
        p_84d: dict[str, int] = {}
        p_84c: dict[str, int] = {}
        for p_row, p_851 in zip(p_85a, labels):
            for p_856, p_850 in zip(p_row, p_851):
                if p_850 == -100:
                    continue
                p_858, p_854 = p_84e[int(p_856)], p_84e[int(p_850)]
                p_857 = p_858.split("-", 1)[1] if p_858 != "O" else None
                p_853 = p_854.split("-", 1)[1] if p_854 != "O" else None
                if p_857 == p_853:
                    if p_853:
                        p_85c[p_853] = p_85c.get(p_853, 0) + 1
                    continue
                if p_857:
                    p_84d[p_857] = p_84d.get(p_857, 0) + 1
                if p_853:
                    p_84c[p_853] = p_84c.get(p_853, 0) + 1

        out: dict[str, float] = {}
        p_847, p_848, p_849 = [], [], []
        for p_84f in set(p_85c) | set(p_84d) | set(p_84c):
            p_85b, p_84b, p_84a = p_85c.get(p_84f, 0), p_84d.get(p_84f, 0), p_84c.get(p_84f, 0)
            p_859 = p_85b / (p_85b + p_84b) if p_85b + p_84b else 0.0
            rec = p_85b / (p_85b + p_84a) if p_85b + p_84a else 0.0
            p_846 = 2 * p_859 * rec / (p_859 + rec) if p_859 + rec else 0.0
            out[f"f1_{p_84f}"] = round(p_846, 4)
            p_847.append(p_846)
            (p_848 if p_84f in IDENTIFIER_FIELDS else p_849).append(p_846)
        out["f1_macro"] = round(sum(p_847) / len(p_847), 4) if p_847 else 0.0
        out["f1_identifier"] = round(sum(p_848) / len(p_848), 4) if p_848 else 0.0
        out["f1_non_identifier"] = round(sum(p_849) / len(p_849), 4) if p_849 else 0.0
        return out

    return _metrics

def train(args) -> int:
    import torch
    from transformers import (
        AutoConfig,
        AutoTokenizer,
        LayoutLMv3ForTokenClassification,
        Trainer,
        TrainingArguments,
    )

    records = [p_861 for p_861 in q_121(Path(args.bio_jsonl)) if q_118(p_861)]
    if not records:
        print("LỖI: không có trang nào mang nhãn khác O", file=sys.stderr)
        return 2
    p_865, p_867 = q_166(records, args.val_bucket, args.mod)
    if not p_867:
        print("LỖI: tập val rỗng — chỉnh --val-bucket/--mod", file=sys.stderr)
        return 2

    print(f"train: {len(p_865)} trang | val: {len(p_867)} trang "
          f"(chia theo document_id, hash ổn định)", file=sys.stderr)
    print(f"phân bố nhãn (train): {json.dumps(q_104(p_865), ensure_ascii=False)}",
          file=sys.stderr)

    p_864 = AutoTokenizer.from_pretrained(args.model, add_prefix_space=True)
    p_85d = AutoConfig.from_pretrained(
        args.model, num_labels=len(BIO_LABELS),
        id2label={i: p_85f for p_85f, i in LABEL2ID.items()}, label2id=dict(LABEL2ID),
    )
    p_85d.visual_embed = False
    model = LayoutLMv3ForTokenClassification.from_pretrained(
        args.model, config=p_85d, ignore_mismatched_sizes=True
    )

    p_863 = TrainingArguments(
        output_dir=args.out,
        num_train_epochs=args.epochs,
        per_device_train_batch_size=args.batch_size,
        per_device_eval_batch_size=args.batch_size,
        learning_rate=args.lr,
        eval_strategy="epoch",
        save_strategy="epoch",
        load_best_model_at_end=True,
        metric_for_best_model="f1_macro",
        greater_is_better=True,
        logging_steps=20,
        fp16=torch.cuda.is_available(),
        seed=args.seed,
        data_seed=args.seed,
        report_to=[],
    )
    p_866 = Trainer(
        model=model, args=p_863,
        train_dataset=q_fb(p_865, p_864, args.max_len),
        eval_dataset=q_fb(p_867, p_864, args.max_len),
        compute_metrics=q_109(),
    )
    p_866.train()
    metrics = p_866.evaluate()
    p_860 = Path(args.out)
    p_860.mkdir(parents=True, exist_ok=True)
    p_866.save_model(str(p_860))
    p_864.save_pretrained(str(p_860))
    p_862 = {
        "metrics": {p_85e: v for p_85e, v in metrics.items() if isinstance(v, (int, float))},
        "train_pages": len(p_865),
        "val_pages": len(p_867),
        "label_counts_train": q_104(p_865),
        "label_counts_val": q_104(p_867),
        "model": args.model,
        "note": (
            "Nhãn train là SILVER (VLM + luật lọc), KHÔNG phải gold người soát. Điểm số ở đây "
            "đo model KHỚP VLM đến đâu, KHÔNG phải model ĐÚNG đến đâu. Giá trị trường còn bị "
            "chặn trên bởi Triton OCR (xem build_bio.py)."
        ),
    }
    (p_860 / "train_report.json").write_text(
        json.dumps(p_862, ensure_ascii=False, indent=2), encoding="utf-8")
    print(json.dumps(p_862, ensure_ascii=False, indent=2), file=sys.stderr)
    return 0

def main(argv: list[str] | None = None) -> int:
    p_868 = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p_868.add_argument("bio_jsonl", help="data/bio.jsonl từ build_bio.py")
    p_868.add_argument("--out", default="output/kie")
    p_868.add_argument("--model", default=DEFAULT_MODEL)
    p_868.add_argument("--epochs", type=float, default=10)
    p_868.add_argument("--batch-size", type=int, default=4, help="16GB VRAM ⇒ giữ nhỏ")
    p_868.add_argument("--lr", type=float, default=5e-5)
    p_868.add_argument("--max-len", type=int, default=DEFAULT_MAX_LEN)
    p_868.add_argument("--val-bucket", type=int, default=0)
    p_868.add_argument("--mod", type=int, default=10, help="1/mod tài liệu vào val")
    p_868.add_argument("--seed", type=int, default=42)
    p_868.add_argument("--dry-run", action="store_true",
                    help="chỉ thống kê dữ liệu, không nạp torch")
    args = p_868.parse_args(argv)

    if args.dry_run:
        records = [p_869 for p_869 in q_121(Path(args.bio_jsonl)) if q_118(p_869)]
        p_86a, p_86b = q_166(records, args.val_bucket, args.mod)
        print(json.dumps({
            "pages_with_labels": len(records),
            "train_pages": len(p_86a), "val_pages": len(p_86b),
            "label_counts": q_104(records),
        }, ensure_ascii=False, indent=2))
        return 0
    return train(args)

if __name__ == "__main__":
    raise SystemExit(main())
