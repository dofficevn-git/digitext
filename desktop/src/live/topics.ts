// Copyright (c) 2026 DigiText. All rights reserved.

export const V3x8 = "data://changed";

export const V3x9 = [
  "session",

  "users",

  "devices",

  "projects",

  "catalog",

  "hoso",

  "dossier",

  "khogia",

  "scan",

  "collected",

  "lan",

  "tree",

  "docs",

  "ocr",

  "ocrFields",
  "dashboard",

  "submissions",

  "notifications",

  "records",

  "access",

  "disposal",

  "info",

  "packages",

  "sign",

  "stats",

  "ops",

  "settings",

  "uploadBatches",

  "chinhLy",

  "clSync",

  "work",
] as const;

export type V3xa = (typeof V3x9)[number];

const V3xb: ReadonlySet<string> = new Set(V3x9);

export function isTopic(s: string): s is V3xa {
  return V3xb.has(s);
}

export const V3xc: readonly string[] = [
  "auth_captcha_new",
  "auth_current",
  "auth_list_accounts",
  "auth_get_server_url",
  "server_ping",

  "changes_poll",

  "dashboard_overview",
  "dashboard_project",
  "dashboard_staff",
  "dashboard_me",
  "dashboard_kpi_settings_get",

  "users_list",
  "user_permissions_get",
  "projects_list",
  "upload_batches_list",
  "upload_batches_matrix",

  "lan_status",
  "lan_discover",
  "lan_manual_peers_get",
  "data_share_status",
  "lan_site_code_get",
  "lan_share_port_get",

  "stats_local",
  "stats_summary",
  "report_so_hoa",
  "report_chinh_ly",
  "report_phan_mem",
  "report_nhan_vien",

  "docs_list",
  "docs_tree",
  "docs_download",
  "docs_pdf_for_view",
  "docs_progress",
  "docs_report",
  "docs_pdfa_check",

  "packages_list",
  "package_download",
  "submissions_list",
  "submission_get",
  "submission_report",
  "submission_dossier",
  "notifications_list",
  "records_list",
  "record_get",
  "record_download",
  "access_list",
  "access_get",
  "access_download",
  "access_fees_get",
  "disposal_entries",
  "disposal_batches",
  "disposal_batch_get",
  "info_list",
  "info_download",
  "ops_integrity_report",
  "ops_backups",
  "ops_backup_report",
  "ocr_fields_list",
  "sign_list_jobs",
  "sign_detect_modules",
  "sign_list_tokens",

  "collected_list",

  "list_scanners",
  "scan_page_history",
  "scanner_driver_extensions",

  "list_ho_so",
  "list_phong",
  "ho_so_at",
  "read_dir",
  "render_pdf_pages",
  "get_storage_root",
  "storage_root_status",
  "open_pdf",
  "save_export",
  "open_url",

  "dossier_list",
  "dossier_get",
  "dossier_history",
  "dossier_incomplete_van_ban",
  "dossier_scan_files",
  "dossier_scan_pdf_files",
  "van_ban_list",
  "get_ocr_url",

  "catalog_list",
  "khai_thac_search",
  "khai_thac_rows",
  "khai_thac_van_ban",
  "kho_list",
  "gia_list",
  "kho_list_detail",
  "gia_list_detail",

  "cl_catalog_list",
  "cl_rows",
  "cl_row",
  "cl_ho_so_list",
  "cl_van_ban_list",
  "cl_next_stt",
  "cl_sync_status",
  "cl_conflicts",
  "cl_outbox",
  "cl_migrate_preview",
  "cl_project_get",
  "work_list_packages",
  "work_get_package",
  "work_my_packages",
  "work_rejected_items",
  "work_next_item",
  "work_navigate",
];

export const V3xd: Readonly<Record<string, readonly V3xa[]>> = {
  auth_login: ["session"],
  auth_logout: ["session"],
  auth_lock: ["session"],
  auth_unlock: ["session"],
  auth_switch_user: ["session"],
  auth_validate_online: ["session"],
  auth_set_server_url: ["settings"],

  user_create: ["users"],
  user_update: ["users"],
  user_set_active: ["users"],
  user_reset_password: ["users"],

  user_permissions_set: ["users", "session"],
  project_create: ["projects"],
  project_set_admins: ["projects"],
  project_add_admin: ["projects"],
  project_remove_admin: ["projects"],

  tree_publish: ["tree"],

  tree_pull: ["tree", "hoso", "catalog", "dossier"],
  lan_pull: ["tree", "hoso", "catalog", "dossier"],
  lan_publish_start: ["lan"],
  lan_publish_stop: ["lan"],
  lan_manual_peers_set: ["lan"],
  data_share_restart: ["lan"],
  lan_site_code_set: ["lan"],
  lan_site_code_generate: ["lan"],
  lan_share_port_set: ["lan"],

  data_collect: ["collected", "hoso"],
  data_collect_cancel: ["collected"],
  data_forward: ["collected", "docs", "uploadBatches"],
  data_forward_cancel: ["collected"],
  collected_retry_rejected: ["collected"],
  collect_add_files: ["collected", "hoso"],

  docs_upload_local: ["docs", "uploadBatches"],
  docs_ocr_all: ["docs", "ocr"],
  docs_make_pdf2_all: ["docs", "ocr"],
  docs_extract_fields_all: ["docs", "ocrFields"],
  docs_rebuild_non_pdfa: ["docs"],
  docs_review: ["docs"],
  docs_review_bulk: ["docs"],
  ocr_field_correct: ["ocrFields"],
  kie_example_set_active: ["ocrFields"],
  dashboard_kpi_settings_set: ["dashboard", "stats", "work"],
  ocr_run_page: ["ocr"],
  ocr_save_boxes: ["ocr"],
  sign_execute: ["sign", "docs"],
  package_create: ["packages", "docs"],
  stats_push: ["stats"],

  submission_register: ["submissions", "notifications"],
  submission_action: ["submissions", "notifications"],
  submission_review_item: ["submissions"],
  submission_approve: ["submissions", "notifications", "docs"],
  submission_returned: ["submissions", "notifications"],
  notification_read: ["notifications"],
  notifications_read_all: ["notifications"],
  record_create: ["records"],
  access_create: ["access", "notifications"],
  access_review_item: ["access"],
  access_action: ["access", "notifications"],
  access_read: ["access"],
  access_fees_set: ["access"],
  disposal_sweep: ["disposal"],
  disposal_set_retention: ["disposal"],
  disposal_entry_action: ["disposal"],
  disposal_batch_create: ["disposal"],
  disposal_batch_action: ["disposal", "docs"],
  info_create: ["info", "notifications"],
  info_action: ["info", "notifications"],
  ops_integrity_run: ["ops"],

  start_scan_batch: ["scan"],
  cancel_scan_batch: ["scan"],
  process_scan_page: ["scan"],
  reset_scan_page: ["scan"],
  undo_scan_page: ["scan"],
  redo_scan_page: ["scan"],
  install_scanner_driver: ["scan"],

  import_ho_so: ["hoso", "dossier"],
  create_folder_tree: ["hoso", "tree", "dossier"],
  finalize_ho_so: ["hoso", "scan"],
  merge_scanned_pages_pdf: ["hoso"],
  open_ho_so_for_edit: ["scan"],
  open_pdf_for_edit: ["scan"],
  insert_pages: ["scan"],
  rebuild_pdf: ["hoso"],
  ho_so_delete: ["hoso", "dossier"],
  ho_so_update: ["hoso"],
  delete_path: ["hoso"],
  rename_path: ["hoso"],
  create_subfolder: ["hoso"],
  move_path: ["hoso"],
  set_storage_root: ["settings", "hoso"],
  set_ocr_url: ["settings"],

  dossier_create: ["dossier"],
  dossier_sync_from_kho: ["dossier"],
  dossier_update: ["dossier"],
  dossier_delete: ["dossier"],
  dossier_systematize: ["dossier"],
  dossier_set_boxing: ["dossier"],
  dossier_bulk_upsert: ["dossier"],
  dossier_delete_incomplete_van_ban: ["dossier"],
  van_ban_create: ["dossier"],
  van_ban_update: ["dossier"],
  van_ban_delete: ["dossier"],
  van_ban_bulk_insert: ["dossier"],

  phong_ban_upsert: ["catalog"],
  phong_ban_delete: ["catalog"],
  mang_upsert: ["catalog"],
  mang_delete: ["catalog"],
  thbq_upsert: ["catalog"],
  thbq_delete: ["catalog"],
  ngon_ngu_upsert: ["catalog"],
  ngon_ngu_delete: ["catalog"],
  loai_van_ban_upsert: ["catalog"],
  loai_van_ban_delete: ["catalog"],

  phong_upsert: ["catalog", "hoso"],
  phong_delete: ["catalog", "hoso"],

  kho_upsert: ["khogia"],
  kho_delete: ["khogia"],
  gia_upsert: ["khogia"],
  gia_delete: ["khogia"],
  dossier_assign_gia: ["khogia", "dossier"],
  dossier_assign_gia_by_range: ["khogia", "dossier"],

  cl_save: ["chinhLy", "clSync"],
  cl_delete: ["chinhLy", "clSync"],
  cl_sync_now: ["chinhLy", "clSync", "catalog", "khogia"],
  cl_conflict_resolve: ["chinhLy", "clSync"],
  cl_outbox_retry: ["clSync"],
  cl_outbox_discard: ["chinhLy", "clSync"],
  cl_remote: ["chinhLy"],
  cl_migrate_run: ["chinhLy", "clSync", "dossier", "catalog"],
  cl_project_set: ["chinhLy", "clSync", "dossier", "catalog", "khogia", "work"],
  work_create_package: ["work"],
  work_patch_package: ["work"],
  work_set_members: ["work"],
  work_review: ["work"],
  work_close_package: ["work"],
  work_claim_item: ["work"],
  work_release: ["work"],
  work_done: ["work"],
  work_check: ["work"],
};

const V3xe: ReadonlySet<string> = new Set(V3xc);

export function V3xf(V0: string): readonly V3xa[] {
  if (V3xe.has(V0)) return [];
  return V3xd[V0] ?? V3x9;
}
