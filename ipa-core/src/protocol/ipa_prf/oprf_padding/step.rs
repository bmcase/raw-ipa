use ipa_step_derive::CompactStep;

#[derive(CompactStep)]
pub(crate) enum PaddingDpStep {
    PaddingDp,
    H1Send,
}
