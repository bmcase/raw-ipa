import itertools
from Compiler import types, library, instructions, sorting

def dest_comp(B):
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
    Bt = B.transpose()
    Bt_flat = Bt.get_vector()
    St_flat = Bt.value_type.Array(len(Bt_flat))
    St_flat.assign(Bt_flat)
    num = len(St_flat) // 2
    @library.for_range(len(St_flat) - 1)
    def _(i):
        St_flat[i + 1] = St_flat[i + 1] + St_flat[i]
    c_0 = St_flat.get_vector(size = num)
    c_1 = St_flat.get_vector(base = num, size = num)
    col = Bt_flat.get_vector(base = num, size = num)

    dest = (c_0 + col * (c_1 - c_0))

    Tt = types.Array(num, B.value_type)
    Tt.assign_vector(dest - 1) # Make 0-origin
    return Tt

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
    cum.assign_vector(prod - col0 - col1 + 1) # 00
    # x0 * (1 - x1)
    cum.assign_vector(col0 - prod, base = num) # 01
    # x1 * (1 - x0)
    cum.assign_vector(col1 - prod, base = 2 * num) # 10
    # x0 * x1
    cum.assign_vector(prod, base = 3 * num) # 11
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
    c00 = cum.get_vector(size = num)
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
    Tt = types.Array(num, type(col0))
    Tt.assign_vector(dest - 1) # Make 0-origin
    return Tt
    
def double_bit_radix_sort(bs, D):
    """
    Use two bits at a time.

    There's an annoying problem if n_bits is odd.
    """
    n_bits, num = bs.sizes
    h = types.Array.create_from(types.sint(types.regint.inc(num)))
    # Test if n_bits is odd
    @library.for_range(n_bits // 2)
    def _(i):
        perm = double_dest(bs[2 * i].get_vector(),
                           bs[2 * i + 1].get_vector())
        sorting.reveal_sort(perm, h, reverse = False)
        @library.if_e(2 * i + 3 < n_bits)
        def _(): # sort the next 2 bits
            # It would be nice if slice behaved
            bot = (2 * i + 2) * num
            tmp = types.Matrix(num, 2, bs.value_type)
            tmp.assign_vector(bs.get_vector(base = bot, size = 2 * num))
            
            sorting.reveal_sort(h, tmp, reverse = True)
            bs.assign_vector(tmp.get_vector(), base = bot)
        @library.else_
        def _():
            @library.if_(n_bits % 2 == 1)
            def odd_case():
                sorting.reveal_sort(h, bs[n_bits - 1], reverse = True)
                c = types.Array.create_from(dest_comp(bs[-1]))
                sorting.reveal_sort(c, h, reverse=False)
    # Now take care of the odd case
    sorting.reveal_sort(h, D, reverse = True)
            
def bit_radix_sort(bs, D):

    n_bits, num = bs.sizes
    assert num == len(D)
    assert n_bits == len(bs)
    B = types.sint.Matrix(num, 2)
    h = types.Array.create_from(types.sint(types.regint.inc(num)))
    @library.for_range(n_bits)
    def _(i):
        b = bs[i]
        B.set_column(0, 1 - b.get_vector())
        B.set_column(1, b.get_vector())
        c = types.Array.create_from(dest_comp(B))
        sorting.reveal_sort(c, h, reverse=False)
        @library.if_e(i < n_bits - 1)
        def _():
            sorting.reveal_sort(h, bs[i + 1], reverse=True)
        @library.else_
        def _():
            sorting.reveal_sort(h, D, reverse=True)

def radix_sort(k, D, n_bits=None, signed=True, two_bit = False):
    assert len(k) == len(D)
    bs = types.Matrix.create_from(k.get_vector().bit_decompose(n_bits))
    if signed and len(bs) > 1:
        bs[-1][:] = bs[-1][:].bit_not()
    if two_bit:
        double_bit_radix_sort(bs, D)
    else:
        bit_radix_sort(bs, D)
