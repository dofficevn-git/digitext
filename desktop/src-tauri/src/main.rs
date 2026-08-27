// Copyright (c) 2026 DigiText. All rights reserved.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod access;
mod archive;
mod auth;
mod catalog;
mod collect;
mod collect_progress;
mod config;
mod data_share;
mod db;
mod disposal;
mod docs;
mod dossier;
mod driver;
mod error;
mod forward_progress;
mod infoops;
mod khaithac;
mod lan;
mod lan_identity;
mod lan_session;
mod live;
mod ocr;
mod packaging;
mod records;
mod report;
mod scanner;
mod signing;
mod stats;
mod storage;
mod submission;
mod sync;
mod telemetry;
use tauri::Manager;
fn main() {
    scanner::f_2ib();
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(scanner::Tkx::default())
        .manage(auth::AuthState::default())
        .manage(auth::captcha::CaptchaStore::default())
        .manage(lan::LanState::default())
        .manage(data_share::T4j::default())
        .manage(collect::SyncGuards::default())
        .manage(forward_progress::ForwardControl::default())
        .manage(forward_progress::CollectControl::default())
        .setup(|r_29j| {
            if let Ok(r_29m) = r_29j.path().app_log_dir() {
                telemetry::init(&r_29m);
            }
            tracing::info!(
                version = env!("CARGO_PKG_VERSION"),
                "RGlnaVRleHQgZGVza3RvcCBraOG7n2kgxJHhu5luZw=="
            );
            let r_29l = r_29j.path().app_data_dir()?;
            std::fs::create_dir_all(&r_29l)?;
            let r_29k = db::open(&r_29l.join(config::DB_FILE))?;
            r_29j.manage(db::Db(std::sync::Mutex::new(r_29k)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            live::changes_poll,
            auth::commands::auth_captcha_new,
            auth::commands::auth_login,
            auth::commands::auth_logout,
            auth::commands::auth_lock,
            auth::commands::auth_unlock,
            auth::commands::auth_switch_user,
            auth::commands::auth_current,
            auth::commands::auth_list_accounts,
            auth::commands::auth_get_server_url,
            auth::commands::server_ping,
            auth::commands::auth_set_server_url,
            auth::commands::auth_validate_online,
            auth::admin::users_list,
            auth::admin::user_create,
            auth::admin::user_update,
            auth::admin::user_set_active,
            auth::admin::user_reset_password,
            auth::admin::user_permissions_get,
            auth::admin::user_permissions_set,
            auth::admin::projects_list,
            auth::admin::project_create,
            auth::admin::project_set_admins,
            auth::admin::project_add_admin,
            auth::admin::project_remove_admin,
            auth::admin::upload_batches_list,
            auth::admin::upload_batches_matrix,
            sync::tree_publish,
            sync::tree_pull,
            lan::lan_publish_start,
            lan::lan_publish_stop,
            lan::lan_status,
            lan::lan_discover,
            lan::lan_pull,
            stats::stats_local,
            stats::stats_push,
            stats::stats_summary,
            docs::docs_list,
            docs::docs_tree,
            docs::docs_download,
            docs::docs_pdf_for_view,
            docs::docs_upload_local,
            docs::docs_ocr_all,
            docs::docs_make_pdf2_all,
            docs::docs_extract_fields_all,
            docs::docs_progress,
            packaging::packages_list,
            packaging::package_create,
            packaging::package_download,
            submission::submissions_list,
            submission::submission_get,
            submission::submission_register,
            submission::submission_action,
            submission::submission_review_item,
            submission::submission_approve,
            submission::submission_returned,
            submission::notifications_list,
            submission::notification_read,
            submission::notifications_read_all,
            submission::submission_report,
            submission::submission_dossier,
            records::records_list,
            records::record_create,
            records::record_download,
            access::access_list,
            access::access_get,
            access::access_create,
            access::access_review_item,
            access::access_action,
            access::access_read,
            access::access_download,
            access::access_fees_get,
            access::access_fees_set,
            disposal::disposal_entries,
            disposal::disposal_sweep,
            disposal::disposal_set_retention,
            disposal::disposal_entry_action,
            disposal::disposal_batches,
            disposal::disposal_batch_get,
            disposal::disposal_batch_create,
            disposal::disposal_batch_action,
            infoops::info_list,
            infoops::info_create,
            infoops::info_action,
            infoops::info_download,
            infoops::ops_integrity_report,
            infoops::ops_integrity_run,
            infoops::ops_backups,
            infoops::ops_backup_report,
            docs::docs_pdfa_check,
            docs::docs_rebuild_non_pdfa,
            docs::docs_report,
            docs::docs_review,
            docs::docs_review_bulk,
            ocr::ocr_fields_list,
            ocr::ocr_field_correct,
            signing::sign_list_jobs,
            signing::sign_detect_modules,
            signing::sign_list_tokens,
            signing::sign_execute,
            data_share::data_share_status,
            data_share::data_share_restart,
            data_share::lan_site_code_get,
            data_share::lan_site_code_set,
            data_share::lan_site_code_generate,
            data_share::lan_share_port_get,
            data_share::lan_share_port_set,
            collect::lan_manual_peers_get,
            collect::lan_manual_peers_set,
            collect::data_collect,
            collect::data_collect_cancel,
            collect::data_forward,
            collect::data_forward_cancel,
            collect::collected_list,
            collect::collected_retry_rejected,
            collect::collect_add_files,
            scanner::list_scanners,
            scanner::start_scan_batch,
            scanner::cancel_scan_batch,
            scanner::process_scan_page,
            scanner::reset_scan_page,
            scanner::undo_scan_page,
            scanner::redo_scan_page,
            scanner::scan_page_history,
            driver::scanner_driver_extensions,
            driver::install_scanner_driver,
            archive::import_ho_so,
            archive::create_folder_tree,
            archive::finalize_ho_so,
            archive::merge_scanned_pages_pdf,
            archive::open_ho_so_for_edit,
            archive::open_pdf_for_edit,
            archive::render_pdf_pages,
            archive::rebuild_pdf,
            archive::insert_pages,
            archive::list_ho_so,
            archive::ho_so_delete,
            archive::ho_so_update,
            archive::list_phong,
            archive::open_pdf,
            archive::save_export,
            archive::get_storage_root,
            archive::set_storage_root,
            archive::storage_root_status,
            archive::delete_path,
            archive::rename_path,
            archive::create_subfolder,
            archive::move_path,
            archive::read_dir,
            archive::ho_so_at,
            dossier::dossier_list,
            dossier::dossier_get,
            dossier::dossier_create,
            dossier::dossier_sync_from_kho,
            dossier::dossier_update,
            dossier::dossier_delete,
            dossier::van_ban_list,
            dossier::van_ban_create,
            dossier::van_ban_update,
            dossier::van_ban_delete,
            dossier::dossier_systematize,
            dossier::dossier_set_boxing,
            dossier::dossier_bulk_upsert,
            dossier::van_ban_bulk_insert,
            dossier::dossier_history,
            dossier::dossier_incomplete_van_ban,
            dossier::dossier_delete_incomplete_van_ban,
            dossier::dossier_scan_files,
            dossier::dossier_scan_pdf_files,
            dossier::get_ocr_url,
            dossier::set_ocr_url,
            dossier::open_url,
            catalog::catalog_list,
            catalog::phong_ban_upsert,
            catalog::phong_ban_delete,
            catalog::mang_upsert,
            catalog::mang_delete,
            catalog::thbq_upsert,
            catalog::thbq_delete,
            catalog::phong_upsert,
            catalog::phong_delete,
            catalog::ngon_ngu_upsert,
            catalog::ngon_ngu_delete,
            catalog::loai_van_ban_upsert,
            catalog::loai_van_ban_delete,
            khaithac::khai_thac_search,
            khaithac::khai_thac_rows,
            khaithac::khai_thac_van_ban,
            khaithac::kho_list,
            khaithac::gia_list,
            khaithac::kho_list_detail,
            khaithac::kho_upsert,
            khaithac::kho_delete,
            khaithac::gia_list_detail,
            khaithac::gia_upsert,
            khaithac::gia_delete,
            khaithac::dossier_assign_gia,
            khaithac::dossier_assign_gia_by_range,
            report::report_so_hoa,
            report::report_chinh_ly,
            report::report_phan_mem,
            report::report_nhan_vien,
        ])
        .run(tauri::generate_context!())
        .expect("error while running DigiText desktop");
}
