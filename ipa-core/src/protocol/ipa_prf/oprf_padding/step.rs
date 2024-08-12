use ipa_step_derive::CompactStep;

#[derive(CompactStep)]
pub(crate) enum PaddingDpStep {
    // #[step(child = SendFakeNumRecords)]
    PaddingDp,
    SendFakeNumRecords,
}
