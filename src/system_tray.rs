use std::time::{Duration, Instant};

use tray_icon::menu::{Menu, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::metrics::SystemSnapshot;

pub enum TrayAction {
    Restore,
    Quit,
}

pub struct TrayState {
    _tray: TrayIcon,
    cpu_item: MenuItem,
    ram_item: MenuItem,
    restore_item: MenuItem,
    quit_item: MenuItem,
    last_update: Instant,
}

impl TrayState {
    pub fn new() -> Option<Self> {
        let _ = gtk::init();
        let icon = load_icon().ok()?;

        let cpu_item = MenuItem::new("CPU: n/a", false, None);
        let ram_item = MenuItem::new("RAM: n/a", false, None);
        let restore_item = MenuItem::new("Restaurar/maximizar", true, None);
        let quit_item = MenuItem::new("Salir", true, None);

        let menu = Menu::new();
        menu.append(&cpu_item).ok()?;
        menu.append(&ram_item).ok()?;
        menu.append(&PredefinedMenuItem::separator()).ok()?;
        menu.append(&restore_item).ok()?;
        menu.append(&quit_item).ok()?;

        let tray = TrayIconBuilder::new()
            .with_tooltip("HiraraMonitor")
            .with_icon(icon)
            .with_menu(Box::new(menu))
            .build()
            .ok()?;

        Some(Self {
            _tray: tray,
            cpu_item,
            ram_item,
            restore_item,
            quit_item,
            last_update: Instant::now(),
        })
    }

    pub fn update_metrics(&mut self, snapshot: &SystemSnapshot) {
        if self.last_update.elapsed() < Duration::from_millis(750) {
            return;
        }

        self.cpu_item
            .set_text(format!("CPU: {:.1}%", snapshot.cpu_global));
        self.ram_item
            .set_text(format!("RAM: {:.1}%", snapshot.memory_used_percent()));
        self.last_update = Instant::now();
    }

    pub fn poll_action(&self) -> Option<TrayAction> {
        while let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
            let restore_id = self.restore_item.id();
            let quit_id = self.quit_item.id();

            if event.id() == restore_id {
                return Some(TrayAction::Restore);
            }

            if event.id() == quit_id {
                return Some(TrayAction::Quit);
            }
        }

        None
    }
}

fn load_icon() -> Result<Icon, std::io::Error> {
    let bytes = include_bytes!("../assets/tray.png");
    let reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(std::io::Error::other)?;
    let decoded = reader.decode().map_err(std::io::Error::other)?;
    let rgba = decoded.to_rgba8();

    Icon::from_rgba(rgba.into_raw(), decoded.width(), decoded.height())
        .map_err(std::io::Error::other)
}
