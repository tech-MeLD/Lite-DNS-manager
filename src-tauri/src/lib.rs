mod commands;
mod error;
mod models;
mod providers;
mod retry;
mod security;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::credentials::get_credentials,
            commands::credentials::save_credential,
            commands::credentials::delete_credential,
            commands::credentials::test_credential,
            commands::domains::list_domains,
            commands::domains::get_domain,
            commands::domains::get_domain_summary,
            commands::records::list_records,
            commands::records::create_record,
            commands::records::update_record,
            commands::records::delete_record,
            commands::records::search_records,
            commands::export::export_zone,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
