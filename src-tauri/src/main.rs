// Prevents additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod notes;
mod workbench;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            use tauri::Manager;
            // 已有实例时：把它已有的便签窗口显示到最前，本进程随后退出。
            if let Some(win) = app.get_webview_window("notes") {
                let _ = win.show();
                let _ = win.unminimize();
                let _ = win.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            notes::setup(app)?;
            workbench::setup(app);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            notes::commands::get_config_dir,
            notes::commands::read_seasons_config,
            notes::commands::create_page,
            notes::commands::list_pages,
            notes::commands::get_page,
            notes::commands::update_page,
            notes::commands::soft_delete,
            notes::commands::restore_page,
            notes::commands::purge_page,
            notes::commands::search,
            notes::commands::get_stats,
            notes::commands::get_meta_index,
            notes::commands::delete_tag,
            notes::commands::list_tags,
            notes::commands::create_tag,
            notes::commands::rename_tag,
            notes::commands::set_tag_description,
            notes::commands::restore_tag,
            notes::commands::merge_tags,
            notes::commands::pick_project_dir,
            notes::commands::list_project_folders,
            notes::commands::list_outlines,
            notes::commands::save_outline,
            notes::commands::set_hit_rects,
            workbench::action::toggle_notes,
            workbench::action::open_notes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
