mod distributions;
mod insecure;
pub mod step;

#[cfg(any(test, feature = "test-fixture", feature = "cli"))]
pub use insecure::DiscreteDp as InsecureDiscreteDp;
use rand::Rng;

use crate::{
    error::Error,
    ff::{
        boolean::Boolean,
        boolean_array::{BooleanArray, BA32, BA64},
        U128Conversions,
    },
    helpers::{Direction, Role, TotalRecords},
    protocol::{
        context::Context,
        ipa_prf::{
            oprf_padding::{insecure::OPRFPaddingDp, step::PaddingDpStep},
            OPRFIPAInputRow,
        },
        RecordId,
    },
    secret_sharing::{
        replicated::{semi_honest::AdditiveShare as Replicated, ReplicatedSecretSharing},
        SharedValue,
    },
};

/// # Errors
/// Will propogate errors from `OPRFPaddingDp`
/// # Panics
/// Panics may happen in `apply_dp_padding_pass`
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
    input =
        apply_dp_padding_pass::<C, BK, TV, TS, B>(ctx, input, Role::H1, Role::H2, Role::H3).await?;
    // input =
    //     apply_dp_padding_pass::<C, BK, TV, TS, B>(ctx, input, Role::H3, Role::H1, Role::H2).await?;
    // input =
    //     apply_dp_padding_pass::<C, BK, TV, TS, B>(ctx, input, Role::H2, Role::H3, Role::H1).await?;

    Ok(input)
}

/// Apply dp padding with one pair of helpers generating the noise
/// Steps
///     1.  Helpers `h_i` and `h_i_plus_one` will get the same rng from PRSS
///         and use it to sample the same random noise for padding from `OPRFPaddingDp`.
///         They will generate secret shares of these fake rows.
///     2.  `h_i` and `h_i_plus_one` will send the send `total_number_of_fake_rows` to `h_out`
///     3.  `h_out` will generate secret shares of zero for as many rows as the `total_number_of_fake_rows`
///
/// # Errors
/// Will propogate errors from `OPRFPaddingDp`
/// # Panics
/// Will panic if called with Roles which are not all unique
pub async fn apply_dp_padding_pass<C, BK, TV, TS, const B: usize>(
    ctx: C,
    mut input: Vec<OPRFIPAInputRow<BK, TV, TS>>,
    h_i: Role,
    h_i_plus_one: Role,
    h_out: Role,
) -> Result<Vec<OPRFIPAInputRow<BK, TV, TS>>, Error>
where
    C: Context,
    BK: BooleanArray,
    TV: BooleanArray,
    TS: BooleanArray,
{
    // assert roles are all unique
    assert!(h_i != h_i_plus_one);
    assert!(h_i != h_out);
    assert!(h_out != h_i_plus_one);

    let matchkey_cardinality_cap = 10; // set by assumptions on capping that either happens on the device or is heuristic in IPA.
    let oprf_padding_sensitivity = 2; // document how set
    let mut total_number_of_fake_rows = 0;
    let mut padding_input_rows: Vec<OPRFIPAInputRow<BK, TV, TS>> = Vec::new();

    // Step 1: Helpers `h_i` and `h_i_plus_one` will get the same rng from PRSS
    // and use it to sample the same random noise for padding from OPRFPaddingDp.
    // They will generate secret shares of these fake rows.
    if ctx.role() != h_out {
        let (mut left, mut right) = ctx.prss_rng();
        let mut rng = &mut right;
        if ctx.role() == h_i {
            rng = &mut right;
        }
        if ctx.role() == h_i_plus_one {
            rng = &mut left;
        }

        // H_i samples how many dummies to create
        // padding for aggregation
        // let aggregation_padding_sensitivity = 10; // document how set
        // let aggregation_padding = OPRFPaddingDp::new(1.0, 1e-6, aggregation_padding_sensitivity)?;

        // let num_breakdowns = B;
        // let mut breakdown_cardinalities: Vec<_> = vec![];
        // // for every breakdown, sample how many dummies will be added
        // for _ in 0..num_breakdowns {
        //     let sample = aggregation_padding.sample(rng);
        //     breakdown_cardinalities.push(sample);
        //     total_fake_breakdownkeys += sample;
        // }

        // padding for oprf
        let oprf_padding = OPRFPaddingDp::new(1.0, 1e-6, oprf_padding_sensitivity)?;
        for cardinality in 1..=matchkey_cardinality_cap {
            let sample = oprf_padding.sample(rng);
            total_number_of_fake_rows += sample * cardinality;

            // this means there will be `sample` many unique
            // matchkeys to add each with cardinality = `cardinality`
            for _ in 0..sample {
                let dummy_mk: BA64 = rng.gen();
                for _ in 0..cardinality {
                    let mut match_key_shares: Replicated<BA64> = Replicated::default();
                    if ctx.role() == h_i {
                        match_key_shares = Replicated::new(BA64::ZERO, dummy_mk);
                    }
                    if ctx.role() == h_i_plus_one {
                        match_key_shares = Replicated::new(dummy_mk, BA64::ZERO);
                    }

                    let row = OPRFIPAInputRow {
                        match_key: match_key_shares,
                        is_trigger: Replicated::new(Boolean::FALSE, Boolean::FALSE),
                        breakdown_key: Replicated::new(BK::ZERO, BK::ZERO),
                        trigger_value: Replicated::new(TV::ZERO, TV::ZERO),
                        timestamp: Replicated::new(TS::ZERO, TS::ZERO),
                    };
                    padding_input_rows.push(row);
                }
            }
        }
    }

    // Step 2: h_i and h_i_plus_one will send the send total_number_of_fake_rows to h_out
    let send_ctx = ctx
        .narrow(&PaddingDpStep::SendFakeNumRecords)
        .set_total_records(TotalRecords::ONE);
    if ctx.role() == h_i {
        let send_channel = send_ctx.send_channel::<BA32>(send_ctx.role().peer(Direction::Left));
        let _ = send_channel
            .send(
                RecordId::FIRST,
                BA32::truncate_from(u128::from(total_number_of_fake_rows)),
            )
            .await;
    }
    if ctx.role() == h_i_plus_one {
        let send_channel = send_ctx.send_channel::<BA32>(send_ctx.role().peer(Direction::Right));
        let _ = send_channel
            .send(
                RecordId::FIRST,
                BA32::truncate_from(u128::from(total_number_of_fake_rows)),
            )
            .await;
    }
    if ctx.role() == h_out {
        // receive `total_number_of_fake_rows` from both other helpers and make sure they are the same
        let recv_channel_right =
            send_ctx.recv_channel::<BA32>(send_ctx.role().peer(Direction::Right));
        let from_right = match recv_channel_right.receive(RecordId::FIRST).await {
            Ok(v) => u32::try_from(v.as_u128()).unwrap(),
            Err(e) => return Err(e.into()),
        };

        let recv_channel_left =
            send_ctx.recv_channel::<BA32>(send_ctx.role().peer(Direction::Left));
        let from_left = match recv_channel_left.receive(RecordId::FIRST).await {
            Ok(v) => u32::try_from(v.as_u128()).unwrap(),
            Err(e) => return Err(e.into()),
        };
        assert_eq!(from_right, from_left);
        total_number_of_fake_rows = from_right;
    }

    // Step 3: `h_out` will generate secret shares of zero for as many rows as the `total_number_of_fake_rows`
    if ctx.role() == h_out {
        for _ in 0..total_number_of_fake_rows as usize {
            let row = OPRFIPAInputRow {
                match_key: Replicated::new(BA64::ZERO, BA64::ZERO),
                is_trigger: Replicated::new(Boolean::FALSE, Boolean::FALSE),
                breakdown_key: Replicated::new(BK::ZERO, BK::ZERO),
                trigger_value: Replicated::new(TV::ZERO, TV::ZERO),
                timestamp: Replicated::new(TS::ZERO, TS::ZERO),
            };
            padding_input_rows.push(row);
        }
    }

    input.extend(padding_input_rows);
    Ok(input)
}

#[cfg(all(test, unit_test))]
mod tests {
    use std::collections::{BTreeMap, HashMap};

    use crate::{
        error::Error,
        ff::{
            boolean_array::{BooleanArray, BA32, BA8},
            U128Conversions,
        },
        helpers::{Direction, Role, TotalRecords},
        protocol::{
            context::Context,
            ipa_prf::{
                oprf_padding::{apply_dp_padding_pass, insecure, insecure::OPRFPaddingDp},
                OPRFIPAInputRow,
            },
            RecordId,
        },
        test_fixture::{Reconstruct, Runner, TestWorld},
    };

    pub async fn set_up_apply_dp_padding_pass<C, BK, TV, TS, const B: usize>(
        ctx: C,
    ) -> Result<Vec<OPRFIPAInputRow<BK, TV, TS>>, Error>
    where
        C: Context,
        BK: BooleanArray,
        TV: BooleanArray,
        TS: BooleanArray,
    {
        let mut input: Vec<OPRFIPAInputRow<BK, TV, TS>> = Vec::new();
        input = apply_dp_padding_pass::<C, BK, TV, TS, B>(ctx, input, Role::H1, Role::H2, Role::H3)
            .await?;
        Ok(input)
    }

    #[tokio::test]
    pub async fn test_apply_dp_padding_pass() {
        type BK = BA8;
        type TV = BA8;
        type TS = BA8;
        const B: usize = 256;
        let world = TestWorld::default();

        let result = world
            .semi_honest((), |ctx, ()| async move {
                set_up_apply_dp_padding_pass::<_, BK, TV, TS, B>(ctx).await
            })
            .await
            .map(Result::unwrap);
        // for Role::H1, Role::H2, Role::H3
        println!("result[0][0] = {:?}", result[0][0]);
        println!("result[1][0] = {:?}", result[1][0]);
        println!("result[2][0] = {:?}", result[2][0]);
        // check that all three helpers added the same number of dummy shares
        assert!(result[0].len() == result[1].len() && result[0].len() == result[2].len());

        let result_reconstructed = result.reconstruct();
        // check that all fields besides the matchkey are zero and matchkey is not zero
        let mut user_id_counts: HashMap<u64, u32> = HashMap::new();
        for row in result_reconstructed {
            // println!("{row:?}");
            assert!(row.timestamp == 0);
            assert!(row.trigger_value == 0);
            assert!(!row.is_trigger_report);
            assert!(row.breakdown_key == 0);
            assert!(row.user_id != 0);

            let count = user_id_counts.entry(row.user_id).or_insert(0);
            *count += 1;
        }
        // Now look at now many times a user_id occured
        let mut sample_per_cardinality: BTreeMap<u32, u32> = BTreeMap::new();
        for cardinality in user_id_counts.values() {
            let count = sample_per_cardinality.entry(*cardinality).or_insert(0);
            *count += 1;
        }
        let mut distribution_of_samples: BTreeMap<u32, u32> = BTreeMap::new();

        for (cardinality, sample) in sample_per_cardinality {
            println!("{sample} user IDs occurred {cardinality} time(s)");
            let count = distribution_of_samples.entry(sample).or_insert(0);
            *count += 1;
        }

        for (sample, count) in &distribution_of_samples {
            println!("An OPRFPadding sample value equal to {sample} occurred {count} time(s)",);
        }
    }

    /// # Errors
    /// Will propogate errors from `OPRFPaddingDp`
    pub fn sample_shared_randomness<C>(ctx: &C) -> Result<u32, insecure::Error>
    where
        C: Context,
    {
        let oprf_padding = OPRFPaddingDp::new(1.0, 1e-6, 10_u32)?;
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

    #[tokio::test]
    pub async fn test_sample_shared_randomness() {
        println!("in test_sample_shared_randomness");
        let world = TestWorld::default();
        let result = world
            .semi_honest(
                (),
                |ctx, ()| async move { sample_shared_randomness::<_>(&ctx) },
            )
            .await;
        println!("result = {result:?}",);
    }

    pub async fn send_to_helper<C>(ctx: C) -> Result<BA32, Error>
    where
        C: Context,
    {
        let mut num_fake_rows: BA32 = BA32::truncate_from(u128::try_from(0).unwrap());

        if ctx.role() == Role::H1 {
            num_fake_rows = BA32::truncate_from(u128::try_from(2).unwrap());
        }
        if ctx.role() == Role::H2 {
            num_fake_rows = BA32::truncate_from(u128::try_from(3).unwrap());
        }
        let send_ctx = ctx.set_total_records(TotalRecords::ONE);
        if ctx.role() == Role::H1 {
            let send_channel = send_ctx.send_channel::<BA32>(send_ctx.role().peer(Direction::Left));
            let _ = send_channel.send(RecordId::FIRST, num_fake_rows).await;
            // send_channel.close(RecordId::FIRST).await;
        }

        if ctx.role() == Role::H3 {
            let recv_channel =
                send_ctx.recv_channel::<BA32>(send_ctx.role().peer(Direction::Right));
            match recv_channel.receive(RecordId::FIRST).await {
                Ok(v) => num_fake_rows = v,
                Err(e) => return Err(e.into()),
            }
        }
        Ok(num_fake_rows)
    }

    #[tokio::test]
    pub async fn test_send_to_helper() {
        let world = TestWorld::default();
        let result = world
            .semi_honest((), |ctx, ()| async move { send_to_helper::<_>(ctx).await })
            .await;
        // .map(Result::unwrap);
        println!("result = {result:?}",);
    }
}
