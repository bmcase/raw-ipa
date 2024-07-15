mod distributions;
mod insecure;

#[cfg(any(test, feature = "test-fixture", feature = "cli"))]
pub use insecure::DiscreteDp as InsecureDiscreteDp;
use crate::ff::boolean_array::BooleanArray;
use crate::helpers::Role;
use crate::protocol::context::Context;
use crate::protocol::ipa_prf::oprf_padding::insecure::OPRFPaddingDp;
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

    match ctx.role() {
        Role::H1 => padding_h1(&ctx,input).await,
        // Role::H2 => padding_h2(&ctx,input).await,
        // Role::H3 => padding_h3(&ctx,input).await,

    }


}

pub async fn padding_h1<C, BK, TV, TS>(
    ctx: C,
    input: Vec<OPRFIPAInputRow<BK, TV, TS>>,
) -> Result<Vec<OPRFIPAInputRow<BK, TV, TS>>, Error>
where
    C: Context,
    BK: BooleanArray,
    TV: BooleanArray,
    TS: BooleanArray,
{
    // H1 samples how many dummies to create

    // padding for aggregation
    if ctx.role() == Role::H1 { // how much should be in here ?

        let aggregation_padding_sensitivity  = 10; // document how set
        let aggregation_padding = OPRFPaddingDp::new(1.0, 1e-6, aggregation_padding_sensitivity);
        let mut rng = rand::thread_rng();
        let mut total_fake_breakdownkeys = 0;
        let num_breakdowns = 8; // todo need to get what this is
        let mut breakdown_cardinalities: Vec<_> = vec![];
        for i in 0..num_breakdowns {
            let sample = aggregation_padding.unwrap().sample(&mut rng);
            breakdown_cardinalities.push(sample);
            total_fake_breakdownkeys += sample;
        }


        // padding for oprf
        let mut total_fake_matchkey = 0;
        let matchkey_cardinality_cap = 10;
        let oprf_padding_sensitivity = 1; // document how set
        let oprf_padding = OPRFPaddingDp::new(1.0, 1e-6, oprf_padding_sensitivity);
        let mut matchkey_cardinalities: Vec<_> = vec![];
        let mut total_fake_matchkeys = 0;

        for i in 0..matchkey_cardinality_cap {
            let sample = oprf_padding.unwrap().sample(&mut rng);
            matchkey_cardinalities.push(sample);
        }

        // H1 creates a flattened list of the dummies to be added.  Then reshares them with
        // the help of H2.
        let mut dummy_mks  = vec![];
        let mut dummy_breakdowns = vec![];
        let num_dummies_to_add = max(total_fake_breakdownkeys,total_fake_matchkeys);
        let mut bk_counter = BreakdownCounter{
            bk:  0,
            bkcount: 0,
        };
        let mut mk_counter = MatchkeyCounter{
            mkcard: 0,
            mkcount: 0,
        };
        while bk_counter.bk < num_breakdowns && bk_counter.bkcount < breakdown_cardinalities[num_breakdowns-1] &&
            mk_counter.mkcard < matchkey_cardinality_cap && mk_counter.mkcount < matchkey_cardinalities[matchkey_cardinality_cap-1]{



        }


    }
    struct BreakdownCounter{
        bk: u32,
        bkcount: u32,

    }
    struct MatchkeyCounter{
        mkcard: u32,
        mkcount: u32,
    }

    // H1 and H2 will generate the dummies







}
