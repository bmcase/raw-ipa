extern crate ipa_macros;

use futures::stream::{iter as stream_iter, StreamExt, TryStreamExt};
use ipa_macros::Step;

use crate::{
    error::Error,
    ff::{Gf2, PrimeField, Serializable},
    protocol::{
        context::{UpgradableContext, UpgradedContext, Validator},
        ipa_prf::prf_sharding::bucket::move_single_value_to_bucket,
        modulus_conversion::convert_bits,
        sort::generate_permutation::ShuffledPermutationWrapper,
        BasicProtocols, RecordId,
    },
    secret_sharing::{
        replicated::{
            malicious::{DowngradeMalicious, ExtendableField},
            semi_honest::AdditiveShare as Replicated,
        },
        BitDecomposed, Linear as LinearSecretSharing,
    },
    seq_join::seq_join,
};

/// Aggregation step for Oblivious Attribution protocol.
/// # Panics
/// It probably won't
///
/// # Errors
/// propagates errors from multiplications
#[tracing::instrument(name = "aggregate_credit", skip_all)]
// instrumenting this function makes the return type look bad to Clippy
#[allow(clippy::type_complexity)]
pub async fn aggregate_credit<V, C, F, IC, IB, S>(
    validator: V,
    breakdown_keys: IB,
    capped_credits: IC,
    max_breakdown_key: u32,
) -> Result<(V, Vec<S>), Error>
where
    V: Validator<C, F>,
    C: UpgradableContext<Validator<F> = V>,
    C::UpgradedContext<F>: UpgradedContext<F, Share = S>,
    F: PrimeField + ExtendableField,
    IB: IntoIterator<Item = BitDecomposed<Replicated<Gf2>>> + ExactSizeIterator + Send,
    IB::IntoIter: Send,
    IC: IntoIterator<Item = S> + ExactSizeIterator + Send,
    IC::IntoIter: Send,
    S: LinearSecretSharing<F> + BasicProtocols<C::UpgradedContext<F>, F> + Serializable + 'static,
    ShuffledPermutationWrapper<S, C::UpgradedContext<F>>: DowngradeMalicious<Target = Vec<u32>>,
{
    let m_ctx = validator.context();
    let res =
        simple_aggregate_credit(m_ctx, breakdown_keys, capped_credits, max_breakdown_key).await?;
    Ok((validator, res))
}

async fn simple_aggregate_credit<F, C, IC, IB, S>(
    ctx: C,
    breakdown_keys: IB,
    capped_credits: IC,
    max_breakdown_key: u32,
) -> Result<Vec<S>, Error>
where
    F: PrimeField,
    IB: IntoIterator<Item = BitDecomposed<Replicated<Gf2>>> + ExactSizeIterator + Send,
    IB::IntoIter: Send,
    IC: IntoIterator<Item = S> + ExactSizeIterator + Send,
    IC::IntoIter: Send,
    C: UpgradedContext<F, Share = S>,
    S: LinearSecretSharing<F> + BasicProtocols<C, F> + Serializable + 'static,
{
    let record_count = breakdown_keys.len();
    let valid_bits_count = u32::BITS - (max_breakdown_key - 1).leading_zeros();

    let move_value_to_bucket_context = ctx
        .narrow(&Step::MoveValueToBucket)
        .set_total_records(record_count);

    let converted_bk = convert_bits(
        ctx.narrow(&Step::ModConvBreakdownKeyBits)
            .set_total_records(record_count),
        stream_iter(breakdown_keys),
        0..valid_bits_count,
    );

    let increments = seq_join(
        ctx.active_work(),
        converted_bk
            .zip(stream_iter(capped_credits))
            .enumerate()
            .map(|(i, (bk, cred))| {
                let ctx = move_value_to_bucket_context.clone();
                async move {
                    move_single_value_to_bucket(
                        ctx,
                        RecordId::from(i),
                        bk.unwrap(),
                        cred,
                        usize::try_from(max_breakdown_key).unwrap(),
                        true,
                    )
                    .await
                }
            }),
    );

    let aggregate = increments
        .try_fold(
            vec![S::ZERO; max_breakdown_key as usize],
            |mut acc, row| async move {
                for (i, incr) in row.into_iter().enumerate() {
                    acc[i] += &incr;
                }
                Ok(acc)
            },
        )
        .await?;
    Ok(aggregate)
}

#[derive(Step)]
pub(crate) enum Step {
    MoveValueToBucket,
    ModConvBreakdownKeyBits,
}

#[cfg(all(test, unit_test))]
mod tests {
    use super::aggregate_credit;
    use crate::{
        ff::{Field, Fp32BitPrime, Gf2},
        protocol::context::UpgradableContext,
        secret_sharing::BitDecomposed,
        test_fixture::{Reconstruct, Runner, TestWorld},
    };

    #[tokio::test]
    pub async fn aggregate() {
        const MAX_BREAKDOWN_KEY: u32 = 8;

        const EXPECTED: &[u128] = &[0, 0, 12, 0, 18, 6, 0, 0];

        // (breakdown_key, credit)
        const INPUT: &[(u32, u32)] = &[
            (3, 0),
            (4, 0),
            (4, 18),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (1, 0),
            (0, 0),
            (2, 2),
            (0, 0),
            (0, 0),
            (2, 0),
            (2, 10),
            (0, 0),
            (0, 0),
            (5, 6),
            (0, 0),
        ];

        let world = TestWorld::default();
        let result = world
            .semi_honest(
                INPUT.iter().map(|&(bk, credit)| {
                    (
                        // decomposed breakdown key
                        BitDecomposed::decompose(
                            u32::BITS - (MAX_BREAKDOWN_KEY - 1).leading_zeros(),
                            |i| Gf2::try_from((u128::from(bk) >> i) & 1).unwrap(),
                        ),
                        // credit
                        Fp32BitPrime::truncate_from(credit),
                    )
                }),
                |ctx, shares| async move {
                    let (bk_shares, credit_shares): (Vec<_>, Vec<_>) = shares.into_iter().unzip();
                    let validator = ctx.validator::<Fp32BitPrime>();
                    let (_validator, output) = aggregate_credit(
                        validator, // note: not upgrading any inputs, so semi-honest only.
                        bk_shares.into_iter(),
                        credit_shares.into_iter(),
                        MAX_BREAKDOWN_KEY,
                    )
                    .await
                    .unwrap();
                    output
                },
            )
            .await
            .reconstruct();
        assert_eq!(result, EXPECTED);
    }
}