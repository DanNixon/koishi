use miette::{IntoDiagnostic, miette};
use skim::{Skim, SkimItemReceiver, prelude::SkimOptionsBuilder};

pub(crate) fn pick_single_item<T: Clone + 'static>(
    item_rx: SkimItemReceiver,
) -> miette::Result<Option<T>> {
    let options = SkimOptionsBuilder::default()
        .bind(vec!["tab:up".into(), "btab:down".into()])
        .build()
        .into_diagnostic()?;

    let skim_output = Skim::run_with(&options, Some(item_rx))
        .ok_or_else(|| miette!("Failed to run fuzzy picker"))?;

    if skim_output.is_abort {
        return Ok(None);
    }

    match skim_output.selected_items.len() {
        0 => Ok(None),
        1 => {
            let selected_record: T = skim_output
                .selected_items
                .first()
                .unwrap()
                .as_any()
                .downcast_ref::<T>()
                .ok_or_else(|| miette!("Failed to get selected item"))?
                .clone();
            Ok(Some(selected_record))
        }
        _ => Err(miette!("Multiple records selected, this is not supported.")),
    }
}
