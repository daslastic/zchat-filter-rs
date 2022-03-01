fn main() {
    let app = zchat_interperter::ZoomApp::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}