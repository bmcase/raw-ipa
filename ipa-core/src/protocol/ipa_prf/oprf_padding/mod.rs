mod distributions;
mod insecure;

use std::{cmp::max, f64};

#[cfg(any(test, feature = "test-fixture", feature = "cli"))]
pub use insecure::DiscreteDp as InsecureDiscreteDp;
use rand::{rngs::ThreadRng, Rng};

use crate::{
    helpers::{BytesStream, Role},
    protocol::{
        basics::Reshare,
        context::prss::InstrumentedSequentialSharedRandomness,
        ipa_prf::{
            boolean_ops::expand_shared_array_in_place, oprf_padding::insecure::OPRFPaddingDp,
            shuffle::shuffled_to_oprfreport, OPRFIPAInputRow,
        },
    },
    secret_sharing::replicated::ReplicatedSecretSharing,
};
pub mod step;

use futures_util::{stream, StreamExt};

use crate::{
    error::{Error, LengthError},
    ff::{
        boolean::Boolean,
        boolean_array::{BooleanArray, BA112, BA64},
        ArrayAccess, U128Conversions,
    },
    helpers::TotalRecords,
    protocol::{
        boolean::step::SixteenBitStep,
        context::Context,
        dp::step::DPStep,
        ipa_prf::{aggregation::aggregate_values, boolean_ops::addition_sequential::integer_add},
        prss::{FromPrss, SharedRandomness},
        BooleanProtocols, RecordId,
    },
    secret_sharing::{
        replicated::semi_honest::{AdditiveShare as Replicated, AdditiveShare},
        BitDecomposed, FieldSimd, SharedValue, TransposeFrom, Vectorizable,
    },
};

pub async fn apply_dp_padding<C, BK, TV, TS, const B: usize>(
    ctx: C,
    mut input: Vec<OPRFIPAInputRow<BK, TV, TS>>,
) -> Result<Vec<OPRFIPAInputRow<BK, TV, TS>>, Error>
where
    C: Context,
    BK: BooleanArray,
    TV: BooleanArray,
    TS: BooleanArray,
{
    // H1 and H2 will need to use PRSS to establish a common shared secret
    // then they will use this to see the rng which needs to be passed in to the
    // OPRF padding struct.  They will then both generate the same random noise for padding.
    // For the values they will follow a convension to get the values into secret shares (maybe
    // also using PRSS values). H3 will set to zero.
    let (mut left, mut right) = ctx.prss_rng();
    let rng = if ctx.role() == Role::H1 {
        &mut right
    } else if ctx.role() == Role::H2 {
        &mut left
    } else {
        return Ok(input);
    }; // TODO H3's behavior

    // H_i samples how many dummies to create
    // padding for aggregation
    let aggregation_padding_sensitivity = 10; // document how set
    let aggregation_padding = OPRFPaddingDp::new(1.0, 1e-6, aggregation_padding_sensitivity)?;

    let mut total_fake_breakdownkeys = 0;
    let num_breakdowns = B;
    let mut breakdown_cardinalities: Vec<_> = vec![];
    // for every breakdown, sample how many dummies will be added
    for _ in 0..num_breakdowns {
        let sample = aggregation_padding.sample(rng);
        breakdown_cardinalities.push(sample);
        total_fake_breakdownkeys += sample;
    }

    // padding for oprf
    let mut number_unique_fake_matchkeys = 0;
    let matchkey_cardinality_cap = 10; // set by assumptions on capping that either happens on the device or is heuristic in IPA.
    let oprf_padding_sensitivity = 2; // document how set
    let oprf_padding = OPRFPaddingDp::new(1.0, 1e-6, oprf_padding_sensitivity)?;
    let mut matchkey_cardinalities: Vec<_> = vec![];

    for _ in 0..matchkey_cardinality_cap {
        let sample = oprf_padding.sample(rng);
        matchkey_cardinalities.push(sample);
        number_unique_fake_matchkeys += sample;
    }

    // generate what the random matchkeys will be
    let mut dummy_mks: Vec<u64> = vec![];
    for _ in 0..number_unique_fake_matchkeys {
        dummy_mks.push(rng.gen())
    }

    // H_i and H_{i+1} will generate the dummies
    // using reshare H_i will know the values and will reshare them with H_{i+1} (with H3 also generating PRSS shares as part
    // of reshare)

    let mut padding_input_rows: Vec<OPRFIPAInputRow<BK, TV, TS>> = Vec::new();
    for cardinality in 0..matchkey_cardinality_cap {
        for i in 0..cardinality {
            // create an additive share of a boolean array
            let mut y = AdditiveShare::new(BA112::ZERO, BA112::ZERO); // where YS=BA112 is the length of the boolean array needed to fit
                                                                      // everything in an OPRFIPAInputRow

            let mut boolean_array_of_mk = vec![];
            if ctx.role() == Role::H1 || ctx.role() == Role::H2 {
                boolean_array_of_mk = [dummy_mks[i]].map(Boolean::from).to_vec();
            } else {
                // Role::H3
                boolean_array_of_mk = [0_u64].map(Boolean::from).to_vec();
            }
            expand_shared_array_in_place(&mut y, &boolean_array_of_mk, 0); // Q: do I need to do something so that this is just known to H_i and H_{i+1}?

            let mut offset = BA64::BITS as usize;

            y.set(offset, &TV::ZERO); // set trigger value to 0
            offset += 1;

            expand_shared_array_in_place(&mut y, &BK::ZERO, offset);

            offset += BK::BITS as usize;
            expand_shared_array_in_place(&mut y, &TV::ZERO, offset);

            offset += TV::BITS as usize;
            expand_shared_array_in_place(&mut y, &TS::ZERO, offset);

            // then use shuffled_to_oprfreport to convert to an OPRFIPAInputRow
            let oprf_input_row: OPRFIPAInputRow<BK, TV, TS> =
                shuffled_to_oprfreport::<BA112, BK, TV, TS>(y);
            padding_input_rows.push(oprf_input_row);
        }
    }
    Ok(input.extend(padding_input_rows))
}

pub async fn sample_shared_randomness<C>(ctx: C) -> Result<u32, insecure::Error>
where
    C: Context,
{
    println!("In sample_shared_randomness");

    // let (instrumented_seq_shared_rng_a
    //     ,instrumented_seq_shared_rng_b) = ctx.prss_rng();
    // println!("ctx.role() {:?}", ctx.role());
    // println!("a.role {:?}", instrumented_seq_shared_rng_a.role );
    // println!("b.role {:?}", instrumented_seq_shared_rng_b.role );
    // let mut seq_shared_rng = instrumented_seq_shared_rng_a.inner;
    let oprf_padding = OPRFPaddingDp::new(1.0, 1e-6, 10_u32)?;
    // let sample = oprf_padding.sample(&mut seq_shared_rng);

    // let (mut left, mut right ) = ctx.prss_rng();
    // let samplewithleft = oprf_padding.sample(&mut left);
    let (mut left, mut right) = ctx.prss_rng();
    let rng = if ctx.role() == Role::H1 {
        &mut right
    } else if ctx.role() == Role::H2 {
        &mut left
    } else {
        return Ok(0);
    };

    let sample = oprf_padding.sample(rng);

    Ok(sample)
}

#[cfg(all(test, unit_test))]
mod tests {
    use crate::{
        ff::{boolean_array::BA16, U128Conversions},
        helpers::Role,
        protocol::{
            context::Context, dp::gen_binomial_noise,
            ipa_prf::oprf_padding::sample_shared_randomness,
        },
        secret_sharing::{replicated::semi_honest::AdditiveShare, TransposeFrom},
        test_fixture::{Reconstruct, Runner, TestWorld},
    };

    #[tokio::test]
    pub async fn test_sample_shared_randomness() {
        println!("in test_sample_shared_randomness");
        let world = TestWorld::default();
        let result = world
            .semi_honest((), |ctx, ()| async move {
                sample_shared_randomness::<_>(ctx).await
            })
            .await;
        println!("result = {:?}", result);
    }
}

//  OLD APPROACH to generate fake rows from the dummy matchkeys and dummy breakdown keys.
//
// /// We need to make sure that the fake breakdown keys will actually make it to being revealed in
// /// aggregation.  Reasons a fake breakdown would not be revealed are that
// /// 1. it was added to a matchkey that never matched (cardinality = 1) and so was dropped
// ///    after the matching stage [do we currently implement this optimization?]
// /// 2. it was added to a matchkey that had too many events.  We enforce a sensitivity bound on
// ///    how many breakdowns can come from one user. To enforce this we need to drop events exceeding
// ///    this bound. So if make fake breakdowns were all added with the same matchkey, many of them
// ///    could get dropped before ever being revealed.
// /// Based on these two factors we need to ensure the following behavior for adding in fake
// /// breakdown keys.  First, they are not added to matchkeys which we know will have cardinality =1.
// /// Second, if the number of dummies to be added for breakdown keys is larger than from OPRF padding
// /// (which could happen with some parameter sets such as a large number of breakdown keys, e.g 100k)
// /// we need to make sure that the matchkeys which are generated to go with the remaining fake
// /// breakdowns are not all the same and not all unique.  The approach we take is to add them to
// /// fake matchkeys with cardinality 2 (except in the odd case where we loose one).
// ///
// ///
// ///
// /// // Now we have two vectors of dummies for matchkeys and breakdown keys; we need to combine these to
//     // create dummy rows to be added.
//     // H_i creates a flattened list of the dummies to be added.  Then reshares them with
//     // the help of H_{i+1}.
//     let mut dummy_mks = vec![];
//     let mut dummy_breakdowns = vec![];
//     // let num_dummies_to_add = max(total_fake_breakdownkeys, total_fake_matchkeys);
// // let mut bk_counter = BreakdownCounter {
// // bk: 0,
// // bkcount: 0,
// // breakdown_cardinalities: breakdown_cardinalities,
// // num_breakdowns: num_breakdowns,
// // };
// //
// // let mut mk_counter = MatchkeyCounter::new(matchkey_cardinalities, matchkey_cardinality_cap, rng);
// // while bk_counter.remaining() && mk_counter.remaining() {
// // // maybe shouldn't add breakdowns when mkcard = 0, or these may never
// // // make it to be revealed for aggregation
// //
// // if mk_counter.mkcard == 1 {
// // dummy_mks.push(mk_counter.current_mk());
// // mk_counter.next();
// // } else {
// // dummy_mks.push(mk_counter.current_mk());
// // dummy_breakdowns.push(bk_counter.current_bk());
// //
// // mk_counter.next();
// // bk_counter.next();
// // }
// // }
// pub struct MatchkeyCounter {
//     pub mkcard: u32,
//     pub mkcount: u32,
//     pub matchkey_cardinalities: Vec<u32>,
//     pub matchkey_cardinality_cap: u32,
//     pub current_mk: u64,
//     pub oprf_padding_finished: bool,
//     pub counter_for_bk_only: u32,
//     pub rng: ThreadRng,
// }
// impl MatchkeyCounter {
//     pub fn new(
//         matchkey_cardinalities: Vec<u32>,
//         matchkey_cardinality_cap: u32,
//         mut rng: &mut InstrumentedSequentialSharedRandomness,
//     ) -> Self {
//         Self {
//             mkcard: 1,
//             mkcount: 0,
//             matchkey_cardinalities,
//             matchkey_cardinality_cap,
//             current_mk: rng.gen(),
//             oprf_padding_finished: false,
//             counter_for_bk_only: 0,
//             rng,
//         }
//     }
//     fn remaining(&self) -> bool {
//         self.mkcard < self.matchkey_cardinality_cap
//             && self.mkcount < self.matchkey_cardinalities[self.matchkey_cardinality_cap - 1]
//     }
//     fn next(&mut self) {
//         if self.remaining() {
//             if self.mkcount < self.matchkey_cardinalities[self.mkcard - 1] {
//                 self.mkcount += 1;
//             } else {
//                 self.mkcard += 1;
//                 self.mkcount = 0;
//                 self.current_mk = self.rng.gen();
//             }
//         }
//     }
//     fn current_mk(&self) -> u64 {
//         if self.remaining() {
//             self.current_mk
//         } else {
//             if self.counter_for_bk_only == 1 {
//                 // generate fresh matchkey
//                 self.current_mk = self.rng.gen();
//                 self.counter_for_bk_only = 1;
//                 return self.current_mk;
//             } else {
//                 return self.current_mk;
//             }
//         }
//     }
// }
//
// pub struct BreakdownCounter {
//     pub bk: u32,
//     pub bkcount: u32,
//     pub breakdown_cardinalities: Vec<u32>,
//     pub num_breakdowns: u32,
// }
// impl BreakdownCounter {
//     fn remaining(&self) -> bool {
//         self.bk < self.num_breakdowns
//             && self.bkcount < self.breakdown_cardinalities[self.num_breakdowns - 1]
//     }
//     fn next(&mut self) {
//         if self.remaining() {
//             if self.bkcount < self.breakdown_cardinalities[self.bk - 1] {
//                 self.bkcount += 1;
//             } else {
//                 self.bk += 1;
//                 self.bkcount = 0;
//             }
//         }
//     }
//     fn current_bk(&self) -> u32 {
//         if self.remaining() {
//             self.bk
//         } else {
//             0_u32
//         }
//     }
// }
