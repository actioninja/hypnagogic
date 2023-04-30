#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Hypnastic",
        native_options,
        Box::new(|cc| Box::new(hypnastic::Hypnastic::new(cc))),
    )
}
