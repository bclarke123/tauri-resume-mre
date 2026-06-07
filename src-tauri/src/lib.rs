// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, _event| {
            // MRE for the mobile Resumed/Suspended RefCell borrow bug.
            //
            // The op must be one the runtime dispatches INLINE on the main
            // thread and that reaches `windows.0.borrow_mut()`. Creating a
            // window qualifies (`build()` -> create_window -> send_user_message
            // runs inline on the main thread -> handle_user_message CreateWindow
            // -> borrow_mut). NOTE: `WebviewWindow::close()` does NOT work — it
            // goes through the deferred proxy queue, not the inline path.
            //
            // On the buggy runtime this re-borrows while the Resumed/Suspended
            // branch still holds `borrow()` -> `already borrowed: BorrowMutError`.
            //
            // IMPORTANT: the mobile lifecycle resume/suspend surfaces as a
            // `RunEvent::WindowEvent { event: WindowEvent::Resumed/Suspended }`
            // (dispatched under the held borrow). `RunEvent::Resumed` is a
            // desktop poll-tick with NO borrow held — matching it does NOT
            // reproduce.
            #[cfg(mobile)]
            if let tauri::RunEvent::WindowEvent { event, .. } = &_event {
                if matches!(
                    event,
                    tauri::WindowEvent::Resumed | tauri::WindowEvent::Suspended
                ) {
                    use tauri::WebviewUrl;
                    eprintln!("[MRE] lifecycle WindowEvent (resume/suspend) -> creating a window");
                    match tauri::WebviewWindowBuilder::new(
                        _app,
                        "mre-child",
                        WebviewUrl::App("index.html".into()),
                    )
                    .build()
                    {
                        Ok(_) => eprintln!("[MRE] window built — NO crash (fixed, or op deferred)"),
                        Err(e) => eprintln!("[MRE] build error (borrow_mut not reached): {e}"),
                    }
                }
            }
        });
}
