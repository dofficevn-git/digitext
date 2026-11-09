# Image máy chủ DigiText Lưu trữ (API + worker chế độ lưu trữ) — bản Lưu trữ GĐ2, 9/11/2026.
# Dựng từ gốc repo:  docker build -f infra/compose/luutru/Dockerfile -t digitext-luutru:local .
# (infra/compose/luutru/cai-dat.sh tự dựng.)
FROM rust:1-bookworm AS build
WORKDIR /src
COPY . .
RUN cargo build --release -p digitext-api -p digitext-ocr \
 && strip target/release/digitext-api target/release/digitext-ocr

FROM debian:bookworm-slim
# fonts-dejavu-core: watermark bản đọc / văn bản xác thực DIP cần font TTF có dấu tiếng Việt.
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates fonts-dejavu-core tzdata \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --system --uid 10001 --home /var/lib/digitext digitext \
 && mkdir -p /var/lib/digitext /opt/digitext/updates \
 && chown digitext /var/lib/digitext
COPY --from=build /src/target/release/digitext-api /src/target/release/digitext-ocr /usr/local/bin/
COPY migrations /opt/digitext/migrations
ENV TZ=Asia/Ho_Chi_Minh \
    DIGITEXT_MIGRATIONS_DIR=/opt/digitext/migrations \
    DIGITEXT_UPDATES_DIR=/opt/digitext/updates \
    DIGITEXT_DATA_DIR=/var/lib/digitext \
    DIGITEXT_JWT_KEY_PATH=/var/lib/digitext/jwt-ed25519.pem \
    DIGITEXT_CERT_FONT=/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf
USER digitext
WORKDIR /var/lib/digitext
EXPOSE 8080
CMD ["digitext-api", "serve"]
