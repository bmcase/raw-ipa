"""
Single bit radix sort
"""
from Compiler import types, library, instructions, sorting

def dest_comp(col):
    """
    Compute the 'destination' permutation
    
    Calculate the permutation to stable sort a bit vector.

    In the original, we multiply have of the cumulative sums
    by the bits, and half by the complements of the bits.
    This can be improved by just refactoring:

    dest[i] = (1 - keys[i]) * cumval[i] + keys[i] * cumval[i + n]

    = cumval[i] + keys[i] * (cumval[i + n] - cumval[i])

    Note: this gives the destination for 1-origin indexing
    for 0-origin (as in Python) we must subtract 1.
    """
    num = len(col)
    cum = types.Array(2 * num, type(col))
    cum.assign_vector(1 - col, base = 0)
    cum.assign_vector(col, base = num)
    @library.for_range(len(cum) - 1)
    def _(i):
        cum[i + 1] = cum[i + 1] + cum[i]
    c_0 = cum.get_vector(base = 0, size = num)
    c_1 = cum.get_vector(base = num, size = num)
    # (1 - x0) * c[0] + x0 * c[1] = c[0] + x0 * (c[1] - c[0])
    # dest = (c_0 + col * (c_1 - c_0))
    # Original
    dest = (1 - col) * c_0 + col * c_1

    return dest - 1

            
def bit_radix_sort(bs, D):

    n_bits, num = bs.sizes
    assert num == len(D)
    assert n_bits == len(bs)
    # Start with the identity permutation
    h = types.Array.create_from(types.sint(types.regint.inc(num)))
    @library.for_range(n_bits)
    def _(i):
        perm = types.Array.create_from(dest_comp(bs[i].get_vector()))

        sorting.reveal_sort(perm, h, reverse=False)
        @library.if_e(i < n_bits - 1)
        def _():
            sorting.reveal_sort(h, bs[i + 1], reverse=True)
        @library.else_
        def _():
            sorting.reveal_sort(h, D, reverse=True)
