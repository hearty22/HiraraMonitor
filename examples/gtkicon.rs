use gtk::prelude::*;
use std::time::Duration;
fn main() {
    gtk::init().expect("gtk init");
    let icon = gtk::StatusIcon::new();
    icon.set_visible(true);
    // Try a standard icon name available in most themes:
    if !icon.set_icon_name(Some("utilities-system-monitor")) {
        icon.set_icon_name(Some("system-run"));
    }
    icon.set_tooltip_text(Some("HiraraMonitor (XEmbed)"));
    icon.connect_button_press_event(|icon, event| {
        println!("button pressed {:?}", event.get_button());
        true
    });
    // Run GTK event loop in a separate thread so caller doesn't block.
    std::thread::spawn(|| gtk::main());
    std::thread::sleep(Duration::from_secs(10));
    println!("done");
    // Can't easily quit; timeout kills.
}
