use os_terminal::{Terminal, font::BitmapFont};
use wuu::{Context, DropDownTerm, Panel, ScreenData, Status, TERMINAL, Wuu};

fn main() {
    let width_px = 800;
    let height_px = 480;
    let mut framebuffer = vec![0; width_px * height_px * 4];
    let screen = ScreenData {
        ptr: framebuffer.as_mut_ptr(),
        width: width_px as u64,
        height: height_px as u64,
        pitch: (width_px * 4) as u64,
    };
    let terminal = Terminal::new(screen, Box::new(BitmapFont));
    *TERMINAL.lock() = Some(terminal);

    wuu::println!("WUU OS demo");
    wuu::info!("Booted with {} MiB", 256);
    wuu::success!("Framebuffer initialized");
    wuu::warning!("This demo renders offscreen");
    wuu::error!("Example error message");

    let panels = [
        Panel::new(
            "Kernel",
            "Boot complete\nMemory manager ready",
            Status::Focused,
            DropDownTerm::new("kernel"),
        ),
        Panel::new(
            "Storage",
            "Filesystem mounted",
            Status::Normal,
            DropDownTerm::new("storage"),
        ),
    ];

    let ui = Wuu::new(Context::Server { cols: 80, rows: 24 });
    ui.render_dashboard("WUU OS", &panels);

    let _terminal = TERMINAL.lock().take();
    let changed_pixels = framebuffer.iter().filter(|pixel| **pixel != 0).count();
    println!("Dashboard rendered to framebuffer ({changed_pixels} non-zero bytes).");
}
