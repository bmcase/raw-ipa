"""
Sort by 3 bits at a time
"""

from Compiler import types, library, instructions, sorting
from itertools import product
from single import dest_comp
from double import double_dest_comp

def triple_dest_comp(col0, col1, col2):

    num = len(col0)
    assert (num == len(col1)) and (num == len(col2))

    x01 = col0 * col1
    x02 = col0 * col2
    x12 = col1 * col2
    x012 = col0 * x12

    cum = types.sint.Array(num * 8)

    cum.assign_vector(1 - col0 - col1 - col2 + x01 + x02 + x12 - x012,
                      base = 0)
    cum.assign_vector(col0 - x01 - x02 + x012, base = num)
    cum.assign_vector(col1 - x01 - x12 + x012, base = 2 * num)
    cum.assign_vector(x01 - x012, base = 3 * num)
    cum.assign_vector(col2 - x02 - x12 + x012, base = 4 * num)
    cum.assign_vector(x02 - x012, base = 5 * num)
    cum.assign_vector(x12 - x012, base = 6 * num)
    cum.assign_vector(x012, base = 7 * num)

    @library.for_range(len(cum) - 1)
    def _(i):
        cum[i + 1] = cum[i + 1] + cum[i]

    cparts = [cum.get_vector(base = _ * num, size = num)
              for _ in range(8)]

    dest = (cparts[0]
            + col0 * (cparts[1] - cparts[0])
            + col1 * (cparts[2] - cparts[0])
            + col2 * (cparts[4] - cparts[0])
            + x01 * (cparts[0] - cparts[1] - cparts[2] + cparts[3])
            + x02 * (cparts[0] - cparts[1] - cparts[4] + cparts[5])
            + x12 * (cparts[0] - cparts[2] - cparts[4] + cparts[6])
            + x012 * (- cparts[0] + cparts[1] + cparts[2] - cparts[3]
                      + cparts[4] - cparts[5] - cparts[6] + cparts[7])
            )
    return dest - 1

def triple_bit_radix_sort(bs, D):
    """
    Three bits at a time
    """
    n_bits, num = bs.sizes
    h = types.Array.create_from(types.sint(types.regint.inc(num)))
    # Test if n_bits is odd
    @library.for_range(n_bits // 3)
    def _(i):
        perm = triple_dest_comp(bs[3 * i].get_vector(),
                                bs[3 * i + 1].get_vector(),
                                bs[3 * i + 2].get_vector())

        sorting.reveal_sort(perm, h, reverse = False)
        @library.if_e(3 * i + 6 <= n_bits)
        def _(): # permute the next three columns

            tmp = types.Matrix(num, 3, bs.value_type)
            tmp.set_column(0, bs[3 * i + 3].get_vector())
            tmp.set_column(1, bs[3 * i + 4].get_vector())
            tmp.set_column(2, bs[3 * i + 5].get_vector())
            sorting.reveal_sort(h, tmp, reverse = True)
            bs[3 * i + 3].assign_vector(tmp.get_column(0))
            bs[3 * i + 4].assign_vector(tmp.get_column(1))
            bs[3 * i + 5].assign_vector(tmp.get_column(2))
                           
        # Handle ragged end
        @library.else_
        def _():
            @library.if_e(n_bits % 3 == 1)
            def mod1():
                sorting.reveal_sort(h, bs[n_bits - 1], reverse = True)
                perm = types.Array.create_from(dest_comp(bs[-1].get_vector()))
                sorting.reveal_sort(perm, h, reverse = False)
            @library.else_
            def _():
                @library.if_(n_bits % 3 == 2)
                def mod2():
                    tmp = types.Matrix(num, 2, bs.value_type)
                    tmp.set_column(0, bs[3 * i + 3].get_vector())
                    tmp.set_column(1, bs[3 * i + 4].get_vector())
                    sorting.reveal_sort(h, tmp, reverse = True)
                    perm = types.Array.create_from(
                        double_dest_comp(tmp.get_column(0),
                                         tmp.get_column(1)))
                    sorting.reveal_sort(perm, h, reverse = False)
                
    sorting.reveal_sort(h, D, reverse = True)

    

    
    

