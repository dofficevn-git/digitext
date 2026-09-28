"""Nhập dữ liệu dạy bóc trường của doc_extract_pla (plakie / Vintern) vào DigiText.

Nguồn: D:/doc_extract_pla/exports/{tu,tu_kie,vnkie}/{train,val}.jsonl + images/*_header.png
(ảnh phần đầu văn bản + nhãn 8 trường, định dạng hội thoại InternVL).

Ra: ml/finetune/kie/data/plakie/
  images/<tên ảnh>            — chép nguyên ảnh
  examples.jsonl              — MỖI ẢNH MỘT DÒNG (bỏ trùng, ưu tiên bản xuất mới nhất):
                                {"id","image","split":"train|heldout","fields":{khoá DigiText: giá trị},
                                 "source": "plakie/<bộ>"}
  heldout = tu/val (47 ảnh) — KHÔNG dùng làm ví dụ, chỉ để đo.

Đổi tên trường (plakie → DigiText):
  document_type → loaiVanBan · document_number_and_symbol → soKyHieu · issue_date → ngayThang ·
  issuing_authority → coQuanBanHanh · summary → trichYeu ·
  supervising_authority → coQuanChuQuan · signer_name → nguoiKy · handwritten_number → soVietTay
  (3 trường cuối chưa có trong 16 trường DigiText — giữ lại để không mất công đã gán nhãn).
"""
import json
import shutil
import sys
from pathlib import Path

SRC = Path(sys.argv[1] if len(sys.argv) > 1 else "D:/doc_extract_pla/exports")
OUT = Path(__file__).parent / "data" / "plakie"
MAP = {
    "document_type": "loaiVanBan",
    "document_number_and_symbol": "soKyHieu",
    "issue_date": "ngayThang",
    "issuing_authority": "coQuanBanHanh",
    "summary": "trichYeu",
    "supervising_authority": "coQuanChuQuan",
    "signer_name": "nguoiKy",
    "handwritten_number": "soVietTay",
}
# Thứ tự ưu tiên khi một ảnh có ở nhiều bộ: bộ xuất sau cùng thắng.
SETS = sorted(
    (d for d in SRC.iterdir() if (d / "train.jsonl").exists()),
    key=lambda d: json.loads((d / "metadata.json").read_text(encoding="utf-8")).get("export_date", "")
    if (d / "metadata.json").exists() else "",
)


def main() -> None:
    (OUT / "images").mkdir(parents=True, exist_ok=True)
    rows: dict[str, dict] = {}
    heldout: set[str] = set()
    bad = 0
    for d in SETS:
        for split in ("train", "val"):
            f = d / f"{split}.jsonl"
            if not f.exists():
                continue
            for line in f.open(encoding="utf-8"):
                line = line.strip()
                if not line:
                    continue
                r = json.loads(line)
                try:
                    raw = json.loads(r["conversations"][1]["value"])
                except (KeyError, IndexError, json.JSONDecodeError):
                    bad += 1
                    continue
                img = d / "images" / r["image"]
                if not img.exists():
                    bad += 1
                    continue
                fields = {MAP[k]: (v or "").strip() for k, v in raw.items() if k in MAP}
                if split == "val":
                    heldout.add(r["image"])
                rows[r["image"]] = {"id": r["id"], "image": r["image"], "fields": fields, "source": f"plakie/{d.name}", "_src": img}
    with (OUT / "examples.jsonl").open("w", encoding="utf-8") as w:
        for name, r in sorted(rows.items()):
            shutil.copy2(r.pop("_src"), OUT / "images" / name)
            r["split"] = "heldout" if name in heldout else "train"
            w.write(json.dumps(r, ensure_ascii=False) + "\n")
    n_h = sum(1 for n in rows if n in heldout)
    print(f"bộ nguồn: {[d.name for d in SETS]}")
    print(f"ảnh khác nhau: {len(rows)} · làm ví dụ: {len(rows) - n_h} · để đo (heldout): {n_h} · dòng lỗi/thiếu ảnh: {bad}")


if __name__ == "__main__":
    main()
