#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${PDFA_GATE_OUT:-$ROOT/target/pdfa-gate}"
case "$OUT" in
  /private/* | /tmp/*) OUT="$HOME/.cache/digitext-pdfa-gate" ;;
esac
IMG="${VERAPDF_IMAGE:-verapdf/cli:v1.30.2}"

if ! command -v docker >/dev/null 2>&1; then
  echo "❌ cần docker để chạy veraPDF ($IMG)" >&2
  exit 2
fi

mkdir -p "$OUT"
rm -f "$OUT"/*.pdf

echo "▸ sinh mẫu artifact lưu trữ…"
DIGITEXT_PDFA_SAMPLE="$OUT/warm.pdf" cargo test -q -p digitext-pdf \
  xuat_mau_pdfa_cho_verapdf -- --ignored >/dev/null
DIGITEXT_SIGNED_SAMPLE="$OUT/signed.pdf" cargo test -q -p digitext-sign-verify \
  xuat_mau_pdf_da_ky_cho_verapdf -- --ignored >/dev/null
DIGITEXT_LT_SAMPLE="$OUT/lt.pdf" cargo test -q -p digitext-sign-verify \
  xuat_mau_pdf_pades_lt_cho_verapdf -- --ignored >/dev/null
DIGITEXT_STAMP_SAMPLE="$OUT/stamp.pdf" cargo test -q -p digitext-sign-verify \
  xuat_mau_pdf_dau_tron_cho_verapdf -- --ignored >/dev/null
DIGITEXT_CERT_SAMPLE="$OUT/cert.pdf" cargo test -q -p digitext-pdf \
  xuat_mau_van_ban_xac_thuc_cho_verapdf -- --ignored >/dev/null

for f in warm signed lt stamp cert; do
  test -s "$OUT/$f.pdf" || {
    echo "❌ không sinh được $f.pdf" >&2
    exit 1
  }
done

echo "▸ veraPDF -f 2b (PDF/A-2b)…"
if ! docker run --rm -v "$OUT":/data "$IMG" --format text -f 2b \
  /data/warm.pdf /data/signed.pdf /data/lt.pdf /data/stamp.pdf /data/cert.pdf; then
  echo "❌ có tệp KHÔNG đạt PDF/A-2b — chi tiết:" >&2
  docker run --rm -v "$OUT":/data "$IMG" --format mrr -f 2b \
    /data/warm.pdf /data/signed.pdf /data/lt.pdf /data/stamp.pdf /data/cert.pdf 2>/dev/null |
    grep -E '<name>|clause=|errorMessage' >&2 || true
  exit 1
fi

echo "✅ cả 5 artifact đạt PDF/A-2b"
