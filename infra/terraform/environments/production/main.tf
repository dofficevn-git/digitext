
module "network" {
  source = "../../modules/network"
  name   = "digitext-prod-net"
}

module "pki" {
  source = "../../modules/pki"
  name   = "digitext-prod-pki"
}

module "ingest_app" {
  source = "../../modules/ingest_app"
  name   = "ingest-app"
  count_ = 2
}

module "compute_node" {
  source = "../../modules/compute_node"
  name   = "gpu-ocr"
  count_ = 4
}

module "minio_pool" {
  source = "../../modules/minio_pool"
  name   = "minio"
  count_ = 6
}

module "postgres_cluster" {
  source = "../../modules/postgres_cluster"
  name   = "pg"
  count_ = 2
}

module "meilisearch" {
  source = "../../modules/meilisearch"
  name   = "meili"
  count_ = 3
}

module "observability" {
  source = "../../modules/observability"
  name   = "obs"
}

module "dr_offsite" {
  source = "../../modules/dr_offsite"
  name   = "dr"
}
