use zeroize::Zeroizing;

pub(crate) fn copy(data: Zeroizing<Vec<u8>>) -> miette::Result<()> {
    // Other clipboard backends can be added and sensibly selected here.
    wayland::copy(data)
}

mod wayland {
    use miette::IntoDiagnostic;
    use wl_clipboard_rs::copy::{self, ClipboardType, MimeType, ServeRequests, Source};
    use zeroize::Zeroizing;

    /// Copies something to the Wayland clipboard, and allow it to be pasted only once.
    pub(super) fn copy(data: Zeroizing<Vec<u8>>) -> miette::Result<()> {
        let source = Source::Bytes(data.to_vec().into());

        let mut opts = copy::Options::new();
        let _ = opts
            .serve_requests(ServeRequests::Only(1))
            .foreground(true)
            .clipboard(ClipboardType::Regular)
            .trim_newline(false)
            .seat(Default::default());

        let copy = opts
            .prepare_copy(source, MimeType::Text)
            .into_diagnostic()?;

        copy.serve().into_diagnostic()?;

        Ok(())
    }
}
