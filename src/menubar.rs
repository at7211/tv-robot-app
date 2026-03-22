use std::sync::Once;

use cocoa::appkit::{
    NSApp, NSApplication, NSApplicationActivationPolicyAccessory, NSMenu,
    NSMenuItem, NSStatusBar,
};
use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};

static mut URL_STRING: Option<String> = None;

fn nsstring(s: &str) -> id {
    unsafe { NSString::alloc(nil).init_str(s) }
}

fn generate_qr_png(url: &str) -> Vec<u8> {
    use qrcode::QrCode;

    let qr = QrCode::new(url.as_bytes()).expect("Failed to create QR code");
    let modules = qr.to_colors();
    let qr_width = qr.width() as u32;
    let scale: u32 = 8;
    let border: u32 = 2;
    let img_size = (qr_width + border * 2) * scale;

    // Build raw RGBA pixels
    let mut pixels = vec![255u8; (img_size * img_size * 4) as usize];
    for qy in 0..qr_width {
        for qx in 0..qr_width {
            let is_dark = modules[(qy * qr_width + qx) as usize] == qrcode::types::Color::Dark;
            if is_dark {
                for py in 0..scale {
                    for px in 0..scale {
                        let x = (qx + border) * scale + px;
                        let y = (qy + border) * scale + py;
                        let idx = ((y * img_size + x) * 4) as usize;
                        pixels[idx] = 0;     // R
                        pixels[idx + 1] = 0; // G
                        pixels[idx + 2] = 0; // B
                        // A stays 255
                    }
                }
            }
        }
    }

    // Encode as BMP (simpler than PNG, no extra crate needed)
    // Actually, we'll pass raw RGBA to NSBitmapImageRep directly
    // Return the raw pixels + metadata
    // We'll encode width in the first 4 bytes
    let mut data = Vec::new();
    data.extend_from_slice(&img_size.to_le_bytes());
    data.extend_from_slice(&pixels);
    data
}

unsafe fn create_qr_nsimage(url: &str) -> id {
    let raw = generate_qr_png(url);
    let img_size = u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]);
    let pixels = &raw[4..];

    // Create NSBitmapImageRep from raw RGBA
    let rep: id = msg_send![class!(NSBitmapImageRep), alloc];
    let rep: id = msg_send![rep,
        initWithBitmapDataPlanes: std::ptr::null_mut::<*mut u8>()
        pixelsWide: img_size as i64
        pixelsHigh: img_size as i64
        bitsPerSample: 8i64
        samplesPerPixel: 4i64
        hasAlpha: YES
        isPlanar: NO
        colorSpaceName: nsstring("NSDeviceRGBColorSpace")
        bytesPerRow: (img_size * 4) as i64
        bitsPerPixel: 32i64
    ];

    // Copy pixel data into the rep
    let bitmap_data: *mut u8 = msg_send![rep, bitmapData];
    std::ptr::copy_nonoverlapping(pixels.as_ptr(), bitmap_data, pixels.len());

    // Create NSImage from rep
    let size = NSSize::new(200.0, 200.0);
    let image: id = msg_send![class!(NSImage), alloc];
    let image: id = msg_send![image, initWithSize: size];
    let _: () = msg_send![image, addRepresentation: rep];

    image
}

extern "C" fn copy_url_action(_this: &Object, _cmd: Sel, _sender: id) {
    unsafe {
        if let Some(ref url) = URL_STRING {
            let pasteboard: id = msg_send![class!(NSPasteboard), generalPasteboard];
            let _: () = msg_send![pasteboard, clearContents];
            let ns_url = nsstring(url);
            let _: () = msg_send![pasteboard, setString: ns_url forType: nsstring("public.utf8-plain-text")];
        }
    }
}

extern "C" fn quit_action(_this: &Object, _cmd: Sel, _sender: id) {
    unsafe {
        let app = NSApp();
        let _: () = msg_send![app, terminate: nil];
    }
}

static REGISTER_CLASS: Once = Once::new();

fn get_action_class() -> &'static Class {
    REGISTER_CLASS.call_once(|| {
        let superclass = class!(NSObject);
        let mut decl = ClassDecl::new("TVRobotActions", superclass).unwrap();
        unsafe {
            decl.add_method(
                sel!(copyUrl:),
                copy_url_action as extern "C" fn(&Object, Sel, id),
            );
            decl.add_method(
                sel!(quitApp:),
                quit_action as extern "C" fn(&Object, Sel, id),
            );
        }
        decl.register();
    });
    Class::get("TVRobotActions").unwrap()
}

pub fn run(url: &str) {
    unsafe {
        URL_STRING = Some(url.to_string());

        let _pool = NSAutoreleasePool::new(nil);
        let app = NSApp();
        app.setActivationPolicy_(NSApplicationActivationPolicyAccessory);

        // Create status bar item
        let status_bar = NSStatusBar::systemStatusBar(nil);
        let status_item: id = msg_send![status_bar, statusItemWithLength: -1.0f64]; // NSVariableStatusItemLength
        let _: () = msg_send![status_item, retain];

        let button: id = msg_send![status_item, button];
        let icon: id = msg_send![class!(NSImage), imageWithSystemSymbolName: nsstring("gamecontroller.fill") accessibilityDescription: nil];
        let _: () = msg_send![button, setImage: icon];

        // Create menu
        let menu = NSMenu::new(nil);

        // Custom view menu item
        let padding: f64 = 20.0;
        let qr_size: f64 = 200.0;
        let view_width: f64 = qr_size + padding * 2.0;
        let view_height: f64 = 300.0;

        let custom_view: id = msg_send![class!(NSView), alloc];
        let frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(view_width, view_height));
        let custom_view: id = msg_send![custom_view, initWithFrame: frame];

        // Title label
        let title_label: id = msg_send![class!(NSTextField), alloc];
        let title_frame = NSRect::new(
            NSPoint::new(padding, view_height - 35.0),
            NSSize::new(qr_size, 25.0),
        );
        let title_label: id = msg_send![title_label, initWithFrame: title_frame];
        let _: () = msg_send![title_label, setStringValue: nsstring("肥豬電腦遙控器")];
        let _: () = msg_send![title_label, setBezeled: NO];
        let _: () = msg_send![title_label, setDrawsBackground: NO];
        let _: () = msg_send![title_label, setEditable: NO];
        let _: () = msg_send![title_label, setSelectable: NO];
        let _: () = msg_send![title_label, setAlignment: 1i64]; // NSTextAlignmentCenter
        let font: id = msg_send![class!(NSFont), boldSystemFontOfSize: 16.0f64];
        let _: () = msg_send![title_label, setFont: font];
        let _: () = msg_send![custom_view, addSubview: title_label];

        // QR code image
        let qr_image = create_qr_nsimage(url);
        let qr_view: id = msg_send![class!(NSImageView), alloc];
        let qr_frame = NSRect::new(
            NSPoint::new(padding, view_height - 245.0),
            NSSize::new(qr_size, qr_size),
        );
        let qr_view: id = msg_send![qr_view, initWithFrame: qr_frame];
        let _: () = msg_send![qr_view, setImage: qr_image];
        let _: () = msg_send![custom_view, addSubview: qr_view];

        // Action target
        let action_class = get_action_class();
        let action_target: id = msg_send![action_class, new];

        // Copy URL button
        let btn_width: f64 = 100.0;
        let btn_gap: f64 = 10.0;
        let btns_total: f64 = btn_width * 2.0 + btn_gap;
        let btn_x: f64 = (view_width - btns_total) / 2.0;

        let copy_btn: id = msg_send![class!(NSButton), alloc];
        let copy_frame = NSRect::new(
            NSPoint::new(btn_x, 15.0),
            NSSize::new(btn_width, 32.0),
        );
        let copy_btn: id = msg_send![copy_btn, initWithFrame: copy_frame];
        let _: () = msg_send![copy_btn, setTitle: nsstring("複製網址")];
        let _: () = msg_send![copy_btn, setBezelStyle: 1i64]; // NSRoundedBezelStyle
        let _: () = msg_send![copy_btn, setTarget: action_target];
        let _: () = msg_send![copy_btn, setAction: sel!(copyUrl:)];
        let _: () = msg_send![custom_view, addSubview: copy_btn];

        // Quit button
        let quit_btn: id = msg_send![class!(NSButton), alloc];
        let quit_frame = NSRect::new(
            NSPoint::new(btn_x + btn_width + btn_gap, 15.0),
            NSSize::new(btn_width, 32.0),
        );
        let quit_btn: id = msg_send![quit_btn, initWithFrame: quit_frame];
        let _: () = msg_send![quit_btn, setTitle: nsstring("結束")];
        let _: () = msg_send![quit_btn, setBezelStyle: 1i64];
        let _: () = msg_send![quit_btn, setTarget: action_target];
        let _: () = msg_send![quit_btn, setAction: sel!(quitApp:)];
        let _: () = msg_send![custom_view, addSubview: quit_btn];

        // Add custom view to menu item
        let menu_item = NSMenuItem::new(nil);
        let _: () = msg_send![menu_item, setView: custom_view];
        menu.addItem_(menu_item);

        // Set menu on status item
        let _: () = msg_send![status_item, setMenu: menu];

        // Run the app
        app.run();
    }
}
