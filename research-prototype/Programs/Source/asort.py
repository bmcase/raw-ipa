"""
New radix sort, one bit values and two bit valus
"""
from Compiler import types, library, instructions, sorting
from single import bit_radix_sort
from double import double_bit_radix_sort
from triple import triple_bit_radix_sort


def radix_sort(arr, D, n_bits = None, signed = False, chunk = 1):
    assert len(arr) == len(D)
    bs = types.Matrix.create_from(arr.get_vector().bit_decompose(n_bits))
    if signed and len(bs) > 1:
        bs[-1][:] = bs[-1][:].bit_not()
    if chunk == 1:
        bit_radix_sort(bs, D)
    elif chunk == 2:
        double_bit_radix_sort(bs, D)
    elif chunk == 3:
        triple_bit_radix_sort(bs, D)
    else:
        raise ValueError("Illegal chunk value")
