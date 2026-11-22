use std::time::Duration;
use x11_dl::xlib::*;
unsafe fn atom(dpy:*mut Display, s:&[u8]) -> Atom {
    XInternAtom(dpy, s.as_ptr() as *const _, 0)
}
fn main() {
    unsafe {
        let dpy = XOpenDisplay(std::ptr::null());
        if dpy.is_null() { eprintln!("no display"); return; }
        let root = XDefaultRootWindow(dpy);
        let win = XCreateSimpleWindow(dpy, root, 0,0,16,16,1,0,0xffffff);
        // _XEMBED_INFO property
        let xembed = atom(dpy, b"_XEMBED_INFO\0");
        let data: [i64;2] = [0,1];
        XChangeProperty(dpy, win, xembed, XA_CARDINAL, 32, PropModeReplace, data.as_ptr() as *const _, 2);
        XSelectInput(dpy, win, StructureNotifyMask);
        let sel = atom(dpy, b"_NET_SYSTEM_TRAY_S0\0");
        let owner = XGetSelectionOwner(dpy, sel);
        println!("tray owner = {:?}", owner);
        if owner != 0 {
            let req = atom(dpy, b"_NET_SYSTEM_TRAY_REQUEST_DOCK\0");
            let mut e: XEvent = std::mem::zeroed();
            let cm = e.client_message;
            // cannot modify union field directly? Use AsMut? We recreate:
            e = XEvent::from(XClientMessageEvent {
                type_: ClientMessage,
                serial: 0,
                send_event: 1,
                display: dpy,
                window: owner,
                message_type: req,
                format: 32,
                data: ClientMessageData::CLong([0, win as _, 0, 0, 0]),
            });
            XSendEvent(dpy, owner, 0, NoEventMask, &mut e);
            XFlush(dpy);
        }
        XMapWindow(dpy, win);
        XFlush(dpy);
        println!("mapped");
        let mut e: XEvent = std::mem::zeroed();
        loop {
            XNextEvent(dpy, &mut e);
            println!("event: {}", e.get_type());
        }
    }
}
