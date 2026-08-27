#!/usr/bin/env bash
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bundle="$here/ica-sign-trust-roots.pem"

VNRCA_URL="https://rootca.gov.vn/crt/vnrca256.p7b"
VNRCA_FP="bac8bf609ab420a8ee1780d74f4de5c7f7184959d2a7375b0a75e4c29f0908e7"
ICA_FP="253db450e9f2140b84d117c38a71f7860465a141f906558a4da9062582b4f0be"

sha256_der() {
  openssl x509 -outform der 2>/dev/null | {
    if command -v sha256sum >/dev/null 2>&1; then sha256sum; else shasum -a 256; fi
  } | cut -d' ' -f1 | tr 'A-F' 'a-f'
}

expect_fp() {
  local got; got="$(sha256_der <"$2")"
  [ "$got" = "$3" ] || { echo "❌ $1: vân tay KHÁC ghim (ghim=$3 fetch=$got)" >&2; exit 1; }
  echo "✓ $1: vân tay khớp ghim"
}

tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT

awk 'BEGIN{n=0} /BEGIN CERT/{n++} n>=1{print > ("'"$tmp"'/bundle" n ".pem")}' "$bundle"
nblk=$(ls "$tmp"/bundle*.pem 2>/dev/null | wc -l | tr -d ' ')
[ "$nblk" = 2 ] || { echo "❌ bundle phải có ĐÚNG 2 cert, thấy $nblk" >&2; exit 1; }

b_vnrca=""; b_ica=""
for f in "$tmp"/bundle*.pem; do
  case "$(sha256_der <"$f")" in
    "$VNRCA_FP") b_vnrca="$f";;
    "$ICA_FP")   b_ica="$f";;
    *) echo "❌ bundle chứa cert LẠ (vân tay ngoài ghim): $f" >&2; exit 1;;
  esac
done
[ -n "$b_vnrca" ] && [ -n "$b_ica" ] || { echo "❌ bundle thiếu VNRCA hoặc I-CA" >&2; exit 1; }
echo "✓ bundle: đúng 2 anchor ghim (VNRCA + I-CA SHA-256), không cert lạ"

echo "→ fetch VNRCA ($VNRCA_URL)"
curl -fsSL --max-time 30 --retry 1 "$VNRCA_URL" -o "$tmp/vnrca.p7b"
openssl pkcs7 -inform der -in "$tmp/vnrca.p7b" -print_certs -out "$tmp/vnrca.pem" 2>/dev/null \
  || openssl pkcs7 -inform pem -in "$tmp/vnrca.p7b" -print_certs -out "$tmp/vnrca.pem"
awk '/BEGIN CERT/{c++} c>=1{print} /END CERT/{if(c>=1) exit}' "$tmp/vnrca.pem" >"$tmp/vnrca1.pem"
expect_fp "VNRCA (nguồn chính thức)" "$tmp/vnrca1.pem" "$VNRCA_FP"
echo "✓ VNRCA trong bundle == VNRCA nguồn chính thức (cùng vân tay)"

if openssl verify -no_check_time -partial_chain -CAfile "$tmp/vnrca1.pem" "$b_ica" >/dev/null 2>&1; then
  echo "✓ I-CA SHA-256: openssl verify OK — do VNRCA (nguồn chính thức) ký"
else
  echo "❌ I-CA SHA-256 KHÔNG verify được theo VNRCA nguồn chính thức" >&2; exit 1
fi

echo
echo "✓ OK — bundle khớp nguồn chính thức + chuỗi tin cậy đúng."
