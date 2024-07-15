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
            breakdown_cardinalities: breakdown_cardinalities,
            num_breakdowns: num_breakdowns,
        };
        let mut mk_counter = MatchkeyCounter{
            mkcard: 0,
            mkcount: 0,
            matchkey_cardinalities: matchkey_cardinalities,
            matchkey_cardinality_cap: matchkey_cardinality_cap,
        };
        while bk_counter.remaining() && mk_counter.remaining() {
            // maybe shouldn't add breakdowns when mkcard = 0, or these may never
            // make it to be revealed for aggregation

            dummy_mks.push(mk_counter.current_mk());
            dummy_breakdowns.push(bk_counter.current_bk());

            mk_counter.next();
            bk_counter.next();
        }


    }
    pub struct BreakdownCounter{
        pub bk: u32,
        pub bkcount: u32,
        pub breakdown_cardinalities: Vec<u32>,
        pub num_breakdowns: u32,
    }
    impl BreakdownCounter {
        fn remaining(&self) -> bool {
            self.bk < self.num_breakdowns  && self.bkcount < self.breakdown_cardinalities[self.num_breakdowns-1]
        }
        fn next(&mut self) {
            if self.remaining() {
                if self.bkcount < self.breakdown_cardinalities[self.bk -1] {
                    self.bkcount += 1;
                } else {
                    self.bk += 1;
                    self.bkcount = 0;
                }
            }
        }
        fn current_bk(&self) -> u32{
            if self.remaining() {
                self.bk
            } else {
                0_u32
            }
        }
        }
    }



    pub struct MatchkeyCounter{
        pub mkcard: u32,
        pub mkcount: u32,
        pub matchkey_cardinalities: Vec<u32>,
        pub matchkey_cardinality_cap: u32,
        pub current_mk: u64,
    }
    impl MatchkeyCounter {
        fn remaining(&self) -> bool {
            self.mkcard < self.matchkey_cardinality_cap && self.mkcount < self.matchkey_cardinalities[self.matchkey_cardinality_cap - 1]
        }
        fn next(&mut self) {
            if self.remaining() {
                if self.mkcount < self.matchkey_cardinalities[self.mkcard - 1] {
                    self.mkcount += 1;
                } else {
                    self.mkcard += 1;
                    self.mkcount = 0;
                    self.current_mk = rand; // todo
                }
            }
        }
        fn current_mk()
    }

    // H1 and H2 will generate the dummies







}
