#!/usr/bin/env python3
""
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

from train_kie import (
    DEFAULT_MAX_LEN,
    q_104,
    doc_bucket,
    q_118,
    q_121,
    q_166,
)

def doc_rank(document_id: str) -> float:
    p_47d = hashlib.sha256(f"curve:{document_id}".encode("utf-8")).hexdigest()
    return int(p_47d[:8], 16) / 0xFFFFFFFF

def q_16b(records: list[dict], frac: float) -> list[dict]:
    if not 0.0 < frac <= 1.0:
        raise ValueError(f"ZnJhY3Rpb24gcGjhuqNpIHRyb25nICgwLDFdLCBuaOG6rW4g{frac}")
    return [p_47e for p_47e in records if doc_rank(p_47e["document_id"]) < frac]

def q_136(s: str) -> list[float]:
    out = []
    for p_480 in s.split(","):
        p_480 = p_480.strip()
        if not p_480:
            continue
        p_47f = float(p_480)
        if not 0.0 < p_47f <= 1.0:
            raise ValueError(f"ZnJhY3Rpb24gcGjhuqNpIHRyb25nICgwLDFdLCBuaOG6rW4g{p_47f}")
        out.append(p_47f)
    if not out:
        raise ValueError("Y+G6p24gw610IG5o4bqldCBt4buZdCBmcmFjdGlvbg==")
    return sorted(set(out))

def q_11e(points: list[dict], key: str = "f1_macro", eps: float = 0.01) -> bool:
    if len(points) < 2:
        return False
    return abs(points[-1][key] - points[-2][key]) < eps

def q_fa(points: list[dict], meta: dict) -> dict:
    p_481 = q_11e(points)
    return {
        "points": points,
        "saturated": p_481,
        "verdict": (
            "ĐI NGANG — thêm data CÙNG LOẠI gần như không giúp; ưu tiên gold người soát "
            "hoặc data ĐA DẠNG hơn (cơ quan/layout khác)."
            if p_481 else
            "Q8OSTiBE4buQQyDigJQgdGjDqm0gZGF0YSBjw7JuIGdpw7pwOyBuaMawbmcgduG6q24gYuG7iyBjaOG6t24gdHLDqm4gYuG7n2kgY2jhuqV0IGzGsOG7o25nIFZMTSArIFRyaXRvbiBPQ1Iu"
        ),
        "meta": meta,
        "note": (
            "Nhãn là SILVER ⇒ đường cong đo model BẮT CHƯỚC VLM tốt lên đến đâu, KHÔNG phải "
            "ĐÚNG lên đến đâu. Đi ngang = đã bắt chước hết mức, không phải = đã đủ tốt."
        ),
    }

def q_176(train_recs: list[dict], val_recs: list[dict], args) -> dict:
    import torch
    from transformers import (
        AutoConfig,
        AutoTokenizer,
        LayoutLMv3ForTokenClassification,
        Trainer,
        TrainingArguments,
    )

    from build_bio import BIO_LABELS, LABEL2ID
    from train_kie import q_fb, q_109

    p_488 = AutoTokenizer.from_pretrained(args.model, add_prefix_space=True)
    p_482 = AutoConfig.from_pretrained(
        args.model, num_labels=len(BIO_LABELS),
        id2label={p_483: p_485 for p_485, p_483 in LABEL2ID.items()}, label2id=dict(LABEL2ID),
    )
    p_482.visual_embed = False
    model = LayoutLMv3ForTokenClassification.from_pretrained(
        args.model, config=p_482, ignore_mismatched_sizes=True
    )
    p_487 = TrainingArguments(
        output_dir=f"{args.out_dir}/tmp",
        num_train_epochs=args.epochs,
        per_device_train_batch_size=args.batch_size,
        per_device_eval_batch_size=args.batch_size,
        learning_rate=args.lr,
        eval_strategy="no",
        save_strategy="no",
        logging_steps=100,
        fp16=torch.cuda.is_available(),
        seed=args.seed,
        data_seed=args.seed,
        report_to=[],
    )
    p_489 = Trainer(
        model=model, args=p_487,
        train_dataset=q_fb(train_recs, p_488, args.max_len),
        eval_dataset=q_fb(val_recs, p_488, args.max_len),
        compute_metrics=q_109(),
    )
    p_489.train()
    p_486 = p_489.evaluate()
    del model, p_489
    torch.cuda.empty_cache()
    return {p_484.replace("eval_", ""): p_48a for p_484, p_48a in p_486.items() if isinstance(p_48a, (int, float))}

def main(argv: list[str] | None = None) -> int:
    p_48b = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p_48b.add_argument("bio_jsonl")
    p_48b.add_argument("--fractions", default="0.25,0.5,0.75,1.0")
    p_48b.add_argument("--out", default="data/curve_report.json")
    p_48b.add_argument("--out-dir", default="output/curve")
    p_48b.add_argument("--model", default="microsoft/layoutlmv3-base")
    p_48b.add_argument("--epochs", type=float, default=10)
    p_48b.add_argument("--batch-size", type=int, default=4)
    p_48b.add_argument("--lr", type=float, default=5e-5)
    p_48b.add_argument("--max-len", type=int, default=DEFAULT_MAX_LEN)
    p_48b.add_argument("--val-bucket", type=int, default=0)
    p_48b.add_argument("--mod", type=int, default=10)
    p_48b.add_argument("--seed", type=int, default=42)
    p_48b.add_argument("--dry-run", action="store_true", help="Y2jhu4kgeGVtIGNoaWEgYmFvIG5oacOqdSwga2jDtG5nIGPhuqduIHRvcmNo")
    args = p_48b.parse_args(argv)

    try:
        p_48d = q_136(args.fractions)
    except ValueError as e:
        print(f"TOG7lkk6IA=={e}", file=sys.stderr)
        return 2

    p_495 = Path(args.bio_jsonl)
    if not p_495.exists():
        print(f"TOG7lkk6IGtow7RuZyB0aOG6pXkg{p_495}", file=sys.stderr)
        return 2
    records = [p_492 for p_492 in q_121(p_495) if q_118(p_492)]
    if not records:
        print("TOG7lkk6IGtow7RuZyBjw7MgdHJhbmcgbsOgbyBtYW5nIG5ow6NuIGtow6FjIE8=", file=sys.stderr)
        return 2

    p_48e, val_recs = q_166(records, args.val_bucket, args.mod)
    if not val_recs:
        print("TOG7lkk6IHThuq1wIHZhbCBy4buXbmcg4oCUIGNo4buJbmggLS12YWwtYnVja2V0Ly0tbW9k", file=sys.stderr)
        return 2

    if args.dry_run:
        p_494 = []
        for p_48c in p_48d:
            p_496 = q_16b(p_48e, p_48c)
            p_494.append({
                "fraction": p_48c, "train_pages": len(p_496),
                "train_docs": len({p_492["document_id"] for p_492 in p_496}),
                "label_counts": q_104(p_496),
            })
        print(json.dumps({
            "val_pages": len(val_recs),
            "val_docs": len({p_492["document_id"] for p_492 in val_recs}),
            "levels": p_494,
        }, ensure_ascii=False, indent=2))
        return 0

    points = []
    for p_48c in p_48d:
        p_496 = q_16b(p_48e, p_48c)
        if not p_496:
            print(f"YuG7jyBt4bupYyA={p_48c}OiBzdWJzZXQgcuG7l25n", file=sys.stderr)
            continue
        p_491 = len({p_492["document_id"] for p_492 in p_496})
        print(f"Cj09PSBt4bupYyA={p_48c:.0%}: {len(p_496)} trang / {p_491}IHTDoGkgbGnhu4d1ID09PQ==", file=sys.stderr)
        p_490 = q_176(p_496, val_recs, args)
        points.append({
            "fraction": p_48c, "train_pages": len(p_496), "train_docs": p_491,
            "f1_macro": round(p_490.get("f1_macro", 0.0), 6),
            "f1_identifier": round(p_490.get("f1_identifier", 0.0), 6),
            "f1_non_identifier": round(p_490.get("f1_non_identifier", 0.0), 6),
            "by_field": {p_48f: p_497 for p_48f, p_497 in p_490.items() if p_48f.startswith("f1_")
                         and p_48f not in ("f1_macro", "f1_identifier", "f1_non_identifier")},
        })
        print(f"  → f1_macro={points[-1]['f1_macro']:.4f} "
              f"f1_identifier={points[-1]['f1_identifier']:.4f}", file=sys.stderr)

    p_493 = q_fa(points, {
        "val_pages": len(val_recs),
        "val_docs": len({p_492["document_id"] for p_492 in val_recs}),
        "full_train_pages": len(p_48e),
        "epochs": args.epochs,
        "model": args.model,
    })
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(p_493, ensure_ascii=False, indent=2), encoding="utf-8")
    print(json.dumps(p_493, ensure_ascii=False, indent=2), file=sys.stderr)
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
