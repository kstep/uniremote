use std::net::SocketAddr;

use qrcode::{QrCode, render::unicode};

use crate::auth::AuthToken;

pub fn print_qr_code(addr: SocketAddr, auth_token: &AuthToken) {
    let url = format!("http://{addr}/login/{auth_token}");

    if addr.ip().is_loopback() {
        println!("Visit: {url}");
        return;
    }

    match QrCode::new(&url) {
        Ok(code) => {
            let string = code
                .render::<unicode::Dense1x2>()
                .dark_color(unicode::Dense1x2::Dark)
                .light_color(unicode::Dense1x2::Light)
                .quiet_zone(false)
                .build();
            println!("\n{string}\n");
            println!("Scan QR code or visit: {url}");
        }
        Err(error) => {
            tracing::warn!("failed to generate qr code: {error}");
            println!("Visit: {url}");
        }
    }
}
