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
        boolean_array::{BooleanArray, BA64},
    },
    helpers::Role,
    protocol::{
        context::Context,
        ipa_prf::{oprf_padding::insecure::OPRFPaddingDp, OPRFIPAInputRow},
    },
    secret_sharing::{
        replicated::{semi_honest::AdditiveShare as Replicated, ReplicatedSecretSharing},
        SharedValue,
    },
};

/// # Errors
/// Will propogate errors from `OPRFPaddingDp`
/// # Panic
/// todo
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

    Ok(input)
}

/// Apple dp padding with one pair of helpers generating the noise
/// # Errors
/// Will propogate errors from `OPRFPaddingDp`
/// # Panic
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

    // H_i and H_{i+1} will need to use PRSS to establish a common shared secret
    // then they will use this to see the rng which needs to be passed in to the
    // OPRF padding struct.  They will then both generate the same random noise for padding.
    // For the values they will follow a convension to get the values into secret shares (maybe
    // also using PRSS values). H3 will set to zero.

    let matchkey_cardinality_cap = 10; // set by assumptions on capping that either happens on the device or is heuristic in IPA.
    let oprf_padding_sensitivity = 2; // document how set
    let mut total_number_of_fake_rows = 0;
    let mut duplicated_dummy_mks: Vec<BA64> = vec![];

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
        let oprf_padding = OPRFPaddingDp::new(10.0, 1e-6, oprf_padding_sensitivity)?;
        for cardinality in 1..=matchkey_cardinality_cap {
            let sample = oprf_padding.sample(rng);
            total_number_of_fake_rows += sample * cardinality;

            // this means there will be `sample` many unique
            // matchkeys to add each with cardinality = `cardinality`
            for _ in 0..sample {
                let dummy_mk: BA64 = rng.gen();
                for _ in 0..cardinality {
                    duplicated_dummy_mks.push(dummy_mk);
                }
            }
        }

        // H_i and H_{i+1} will generate the dummies
        // using reshare H_i will know the values and will reshare them with H_{i+1} (with H3 also generating PRSS shares as part
        // of reshare)
    }
    assert!(total_number_of_fake_rows as usize == duplicated_dummy_mks.len());

    let mut padding_input_rows: Vec<OPRFIPAInputRow<BK, TV, TS>> = Vec::new();
    for mk in duplicated_dummy_mks {
        let mut match_key_shares: Replicated<BA64> = Replicated::default();
        if ctx.role() == h_i {
            match_key_shares = Replicated::new(BA64::ZERO, mk);
        }
        if ctx.role() == h_i_plus_one {
            match_key_shares = Replicated::new(mk, BA64::ZERO);
        }
        if ctx.role() == h_out {
            match_key_shares = Replicated::new(BA64::ZERO, BA64::ZERO);
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

    input.extend(padding_input_rows);
    Ok(input)
}

#[cfg(all(test, unit_test))]
mod tests {
    use crate::{
        error::Error,
        ff::boolean_array::{BooleanArray, BA8},
        helpers::Role,
        protocol::{
            context::Context,
            ipa_prf::{
                oprf_padding::{apply_dp_padding_pass, insecure, insecure::OPRFPaddingDp},
                OPRFIPAInputRow,
            },
        },
        test_fixture::Reconstruct,
    };
    use crate::{
        // protocol::ipa_prf::oprf_padding::sample_shared_randomness,
        test_fixture::{Runner, TestWorld},
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

        // input =
        //     apply_dp_padding_pass::<C, BK, TV, TS, B>(ctx, input, Role::H1, Role::H2, Role::H3).await?;
        input = apply_dp_padding_pass::<C, BK, TV, TS, B>(ctx, input, Role::H3, Role::H1, Role::H2)
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
        // println!("result[0][0] = {:?}",result[0][0]);
        // println!("");
        // println!("result[1][0] = {:?}",result[1][0]);
        // println!("");
        // println!("result[2] = {:?}",result[2]);

        //  for Role::H3, Role::H1, Role::H2
        println!("result[0][0] = {:?}", result[0][0]);
        println!("***************");
        println!("result[1] = {:?}", result[1]);
        println!("***************");
        println!("result[2][0] = {:?}", result[2][0]);
        println!("***************");

        let result_reconstructed = result.reconstruct();

        // for row in result_reconstructed.iter().take(5) {
        //     println!("{row:?}",);
        // }
    }

    /// # Errors
    /// Will propogate errors from `OPRFPaddingDp`
    pub async fn sample_shared_randomness<C>(ctx: C) -> Result<u32, insecure::Error>
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
            .semi_honest((), |ctx, ()| async move {
                sample_shared_randomness::<_>(ctx).await
            })
            .await;
        println!("result = {result:?}",);
    }
}
