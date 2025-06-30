use image::Luma;
use miette::IntoDiagnostic;
use qrcode::{QrCode, render::unicode::Dense1x2};
use std::io::Cursor;
use zeroize::Zeroizing;

pub(crate) fn encode_png(data: Zeroizing<Vec<u8>>) -> miette::Result<Zeroizing<Vec<u8>>> {
    let code = QrCode::new(data).into_diagnostic()?;

    let image = code.render::<Luma<u8>>().build();

    let mut bytes = Zeroizing::new(Vec::new());
    image
        .write_to(&mut Cursor::new(&mut *bytes), image::ImageFormat::Png)
        .into_diagnostic()?;

    Ok(bytes)
}

pub(crate) fn encode_ascii(data: Zeroizing<Vec<u8>>) -> miette::Result<Zeroizing<String>> {
    let code = QrCode::new(data).into_diagnostic()?;

    let qr = Zeroizing::new(
        code.render::<char>()
            .dark_color('#')
            .module_dimensions(2, 1)
            .build(),
    );

    Ok(qr)
}

pub(crate) fn encode_unicode(data: Zeroizing<Vec<u8>>) -> miette::Result<Zeroizing<String>> {
    let code = QrCode::new(data).into_diagnostic()?;

    let qr = Zeroizing::new(
        code.render::<Dense1x2>()
            .dark_color(Dense1x2::Light)
            .light_color(Dense1x2::Dark)
            .build(),
    );

    Ok(qr)
}
