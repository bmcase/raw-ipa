mod distributions;
mod insecure;

#[cfg(any(test, feature = "test-fixture", feature = "cli"))]
pub use insecure::DiscreteDp as InsecureDiscreteDp;
use crate::ff::boolean_array::BooleanArray;
use crate::protocol::context::Context;
use crate::protocol::ipa_prf::OPRFIPAInputRow;


pub async fn apply_dp_padding<C, BK, TV, TS>(
    ctx: C,
    input: Vec<OPRFIPAInputRow<BK, TV, TS>>,
) -> Result<Vec<OPRFIPAInputRow<BK, TV, TS>>, Error>
where
    C: Context,
    BK: BooleanArray,
    TV: BooleanArray,
    TS: BooleanArray,
{



}