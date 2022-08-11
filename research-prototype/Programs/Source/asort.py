"""
New radix sort, one bit values and two bit valus
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

def double_dest(col0, col1):
    """
    bs is an n by 2 bit array.
    """
    num = len(col0)
    assert num == len(col1)
    # num, _ = bs.sizes
    cum = types.sint.Array(num * 4)
    # col0 = bs.get_column(0)
    # col1 = bs.get_column(1)
    prod = col0 * col1
    # (1 - x0) * (1 - x1)
    cum.assign_vector(prod - col0 - col1 + 1, base = 0) # 00
    # x0 * (1 - x1)
    cum.assign_vector(col0 - prod, base = num) # 01
    # x1 * (1 - x0)
    cum.assign_vector(col1 - prod, base = 2 * num) # 10
    # x0 * x1
    cum.assign_vector(prod, base = 3 * num) # 11
    # Prefix sum
    @library.for_range(len(cum) - 1)
    def _(i):
        cum[i + 1] = cum[i + 1] + cum[i]
    # (1 - x0) * (1 - x1) * c[00]
    # + x0 * (1 - x1) * c[01]
    # + (1 - x0) * x1 * c[10]
    # + x0 * x1 * c[11]
    # = c[00] + x0 * (-c[00] + c[01])
    # + x1 * (- c[00] + c[10])
    # + x0 * x1 * (c[00] - c[01] - c[10] + c[11])
    # coefficients of 1
    c00 = cum.get_vector(base = 0, size = num)
    c01 = cum.get_vector(base = num, size = num)
    c10 = cum.get_vector(base = 2 * num, size = num)
    c11 = cum.get_vector(base = 3 * num, size = num)
    one_contrib = c00
    # coefficient of col0
    col0_contrib = c01 - c00
    col1_contrib = c10 - c00
    prod_contrib = c00 - c01 - c10 + c11

    dest = (one_contrib
            + col0 * col0_contrib
            + col1 * col1_contrib
            + prod * prod_contrib)

    return  dest - 1
    
def double_bit_radix_sort(bs, D):
    """
    Use two bits at a time.

    There's an annoying problem if n_bits is odd.
    """
    n_bits, num = bs.sizes
    # Start with the identity permutation
    h = types.Array.create_from(types.sint(types.regint.inc(num)))
    # Test if n_bits is odd
    @library.for_range(n_bits // 2)
    def _(i):
        perm = double_dest(bs[2 * i].get_vector(),
                           bs[2 * i + 1].get_vector())

        sorting.reveal_sort(perm, h, reverse = False)
        @library.if_e(2 * i + 3 < n_bits)
        def _(): # permute the next two columns

            bot = (2 * i + 2) * num
            tmp = types.Matrix(num, 2, bs.value_type)
            tmp.set_column(0, bs[2 * i + 2].get_vector())
            tmp.set_column(1, bs[2 * i + 3].get_vector())
            sorting.reveal_sort(h, tmp, reverse = True)
            bs[2 * i + 2].assign_vector(tmp.get_column(0))
            bs[2 * i + 3].assign_vector(tmp.get_column(1))

        @library.else_
        def _():
            @library.if_e(n_bits % 2 == 0)
            def even_case():
                sorting.reveal_sort(h, D, reverse = True)
            @library.else_
            def odd_case():
                sorting.reveal_sort(h, bs[n_bits - 1], reverse = True)
                perm = types.Array.create_from(dest_comp(bs[-1].get_vector()))
                sorting.reveal_sort(perm, h, reverse = False)
                sorting.reveal_sort(h, D, reverse = True)

    # Now take care of the odd case
            
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

def radix_sort(arr, D, n_bits = None, signed = False, two_bit = False):
    assert len(arr) == len(D)
    bs = types.Matrix.create_from(arr.get_vector().bit_decompose(n_bits))
    if signed and len(bs) > 1:
        bs[-1][:] = bs[-1][:].bit_not()
    if two_bit:
        double_bit_radix_sort(bs, D)
    else:
        bit_radix_sort(bs, D)
