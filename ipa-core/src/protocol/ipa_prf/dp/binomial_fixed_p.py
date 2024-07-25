# binomial with variable p
# implement calculations to instantiation Thm 1 of https://arxiv.org/pdf/1805.10559
import math
import locale
locale.setlocale(locale.LC_NUMERIC, 'en_US')

# equation (17)
def b_p(p):
    return (2/3) * (p**2 + (1-p)**2) + 1 - 2*p

# equation (12)
def c_p(p):
    return math.sqrt(2) * (3 * p**3 + 3*(1-p)**3 + 2 * p**2 + 2 * (1-p)**2)

# equation (16)
def d_p(p):
    return (4/3) * (p**2 + (1-p)**2)

# equation (7)
def epsilon(N,p,delta, s, d, Delta_1, Delta_2, Delta_infty):
    first_term_num = Delta_2 * math.sqrt(2 * math.log(1.25 / delta))
    first_term_den = s * math.sqrt(N * p * (1-p))
    second_term_num = Delta_2 * c_p(p) * math.sqrt(math.log(10/delta)) + Delta_1 * b_p(p)
    second_term_den = s * N * p * (1 - p)*(1 - delta/10)
    third_term_num = (2/3) * Delta_infty * math.log(1.25/delta) + Delta_infty * d_p(p) * math.log(20 * d/delta) * math.log(10/delta)
    third_term_den = s * N * p * (1-p)
    return first_term_num / first_term_den + second_term_num / second_term_den + third_term_num / third_term_den

# constraint in Thm 1
def delta_contraint(N,p,d,s,delta,Delta_infty):
    lhs = N * p * (1-p)
    rhs = max(23 * math.log(10 * d / delta), 2 * Delta_infty / s)
    # print("lhs",lhs)
    # print("rhs",rhs)
    return (lhs >= rhs)

# error of mechanism in Thm 1
def error(N,p,d,s):
    return d * s**2 * N * p * (1-p)


# for fixed p (and other params), find smallest N such that epsilon < desired_epsilon
def find_smallest_N(desired_epsilon,p,delta,d,s,Delta_1,Delta_2,Delta_infty):
    for N in range(1,10**9):
        if delta_contraint(N,p,d,s,delta,Delta_infty):
            if desired_epsilon >= epsilon(N,p,delta, s, d, Delta_1, Delta_2, Delta_infty):
                return N
    print("smallest N not found")
    return -1

def find_smallest_N_binary_search(desired_epsilon,p,delta,d,s,Delta_1,Delta_2,Delta_infty):
    lower = 1
    higher = 10**22
    index = 0

    while(lower <= higher):
        mid = math.floor((higher - lower)/2) + lower
        if(delta_contraint(mid,p,d,s,delta,Delta_infty) and (desired_epsilon >= epsilon(mid,p,delta, s, d, Delta_1, Delta_2, Delta_infty))):
            index = mid
            higher = mid - 1
        else:
            lower = mid + 1
    if index == 0:
        print("smallest_N not found less than `higher`")
        assert(index > 0)
    return index

def study_quantization_scale():
    print("study quantization scale")
    p = 1/2
    desired_epsilon  = 1
    delta = 10**(-6)
    d = 1
    Delta_1 = 1
    Delta_2 = 1
    Delta_infty = 1
    s = 1
    smallest_N = find_smallest_N_binary_search(desired_epsilon,p,delta,d,s,Delta_1,Delta_2,Delta_infty)
    err = error(smallest_N,p,d,s)
    print(f"s = {s}, smallest_N = {smallest_N:,}, error = {err}")
    for j in range(10,1010,10):
        s = 1 / j
        smallest_N = find_smallest_N_binary_search(desired_epsilon,p,delta,d,s,Delta_1,Delta_2,Delta_infty)
        err = error(smallest_N,p,d,s)
        print(f"s = {s}, smallest_N = {smallest_N:,}, error = {err}")

# study_quantization_scale()


# for fixed p (and other params), compare which contraint (epsilon or delta) is active for a particular N
def compare_constraints(desired_epsilon,p,delta,d,s,Delta_1,Delta_2,Delta_infty):
    for N in range(1,10**4,10):
        if N == 1:
            assert(not delta_contraint(N,p,d,s,delta,Delta_infty))
            assert(not (desired_epsilon >= epsilon(N,p,delta, s, d, Delta_1, Delta_2, Delta_infty)))
        print("N = ", N)
        print("constrainted by delta: ", not delta_contraint(N,p,d,s,delta,Delta_infty))
        print("constrained by epsilon: ", not (desired_epsilon >= epsilon(N,p,delta, s, d, Delta_1, Delta_2, Delta_infty)))



def aggregation_p_one_half():
    p = 1/2
    desired_epsilon  = 1
    delta = 10**(-6)
    d = 1
    s = 1
    Delta_1 = 1
    Delta_2 = 1
    Delta_infty = 1
    smallest_N = find_smallest_N(desired_epsilon,p,delta,d,s,Delta_1,Delta_2,Delta_infty)
    smallest_N_bs = find_smallest_N_binary_search(desired_epsilon,p,delta,d,s,Delta_1,Delta_2,Delta_infty)
    print("smallest_N =", smallest_N)
    print("smallest_N_bs =", smallest_N_bs)
    assert(smallest_N == smallest_N_bs)
    err = error(smallest_N,p,d,s)
    print("with p = ", p)
    print("smallest_N =", smallest_N)
    print("error =", err)
    print()
#     compare_constraints(desired_epsilon,p,delta,d,s,Delta_1,Delta_2,Delta_infty)



def aggregation_p_one_forth():
    p = 1/4
    desired_epsilon  = 1
    delta = 10**(-6)
    d = 1
    s = 1
    Delta_1 = 1
    Delta_2 = 1
    Delta_infty = 1
    smallest_N = find_smallest_N(desired_epsilon,p,delta,d,s,Delta_1,Delta_2,Delta_infty)
    smallest_N_bs = find_smallest_N_binary_search(desired_epsilon,p,delta,d,s,Delta_1,Delta_2,Delta_infty)
    assert(smallest_N == smallest_N_bs)
    err = error(smallest_N,p,d,s)
    print("with p = ", p)
    print("smallest_N =", smallest_N)
    print("error =", err)
    print()

def aggregation_p_three_forths():
    p = 3/4
    desired_epsilon  = 1
    delta = 10**(-6)
    d = 1
    s = 1
    Delta_1 = 1
    Delta_2 = 1
    Delta_infty = 1
    smallest_N = find_smallest_N(desired_epsilon,p,delta,d,s,Delta_1,Delta_2,Delta_infty)
    smallest_N_bs = find_smallest_N_binary_search(desired_epsilon,p,delta,d,s,Delta_1,Delta_2,Delta_infty)
    assert(smallest_N == smallest_N_bs)
    err = error(smallest_N,p,d,s)
    print("with p = ", p)
    print("smallest_N =", smallest_N)
    print("error =", err)
    print()

def aggregation_s_100th():
    p = 1/2
    desired_epsilon  = 1
    delta = 10**(-6)
    d = 1
    s = 1/100
    Delta_1 = 1
    Delta_2 = 1
    Delta_infty = 1
    smallest_N = find_smallest_N(desired_epsilon,p,delta,d,s,Delta_1,Delta_2,Delta_infty)
    smallest_N_bs = find_smallest_N_binary_search(desired_epsilon,p,delta,d,s,Delta_1,Delta_2,Delta_infty)
    print("smallest_N =", smallest_N)
    print("smallest_N_bs =", smallest_N_bs)
    assert(smallest_N == smallest_N_bs)
    err = error(smallest_N,p,d,s)
    print("with p = ", p)
    print("smallest_N =", smallest_N)
    print("error =", err)
    print()

def aggregation_s_10th():
    p = 1/2
    desired_epsilon  = 1
    delta = 10**(-6)
    d = 1
    s = 1/10
    Delta_1 = 1
    Delta_2 = 1
    Delta_infty = 1
    smallest_N = find_smallest_N(desired_epsilon,p,delta,d,s,Delta_1,Delta_2,Delta_infty)
    smallest_N_bs = find_smallest_N_binary_search(desired_epsilon,p,delta,d,s,Delta_1,Delta_2,Delta_infty)
    print("smallest_N =", smallest_N)
    print("smallest_N_bs =", smallest_N_bs)
    assert(smallest_N == smallest_N_bs)
    err = error(smallest_N,p,d,s)
    print("with p = ", p)
    print("smallest_N =", smallest_N)
    print("error =", err)
    print()

def walr(j,p):
    s = 1/j
    print("WALR, s=",s)
#     p = 1/2
    desired_epsilon  = 1
    delta = 10**(-6)
    d = 32
    Delta_1 = 32 * 256
    Delta_2 = math.sqrt(32) * 256
    Delta_infty = 256
#     smallest_N = find_smallest_N(desired_epsilon,p,delta,d,s,Delta_1,Delta_2,Delta_infty)
    smallest_N_bs = find_smallest_N_binary_search(desired_epsilon,p,delta,d,s,Delta_1,Delta_2,Delta_infty)
#     print("smallest_N =", smallest_N)
    print("smallest_N =", locale.format_string("%d", smallest_N_bs, grouping=True))
#     assert(smallest_N == smallest_N_bs)
    err = error(smallest_N_bs,p,d,s)
#     print("with p = ", p)
    print("error =", locale.format_string("%d", err, grouping=True) )
    print()

def parameter_table():
    print("printing parameter_table")
    epsilon_delta_values = [(0.01, 10**(-9)), (0.1, 10**(-7)), (1, 10**(-7)), (5, 10**(-7))]
    #     epsilon_values = [0.01,0.1,1,5,10]
#     epsilon_values = [5]
#     delta_values = [10**(-9),10**(-8),10**(-7),10**(-6)]
    dimension_values = [32]
    per_user_credit_cap_values = [1,16,32,64]
    quantization_scale_values = [1, 0.5, 0.2, 0.15, 0.1, 0.05, 0.01, 0.008, 0.005, 0.001]
    quantization_scale_values = sorted(quantization_scale_values, reverse=True)


    p = 0.5
    # Create a string with placeholders for the values
    table_format_string = "{epsilon}, {delta}, {dimension}, {ell_1_sensitivity}, {ell_2_sensitivity}, {ell_infity_sensitivity}, {quantization_scale}, {num_bernoulli}, {err}"
#     for epsilon in epsilon_values:
#         for delta in delta_values:

    for (epsilon, delta) in epsilon_delta_values:
        for dimension in dimension_values:
            for per_user_credit_cap in per_user_credit_cap_values:
                ell_1_sensitivity = per_user_credit_cap
                ell_2_sensitivity = per_user_credit_cap
                ell_infity_sensitivity = per_user_credit_cap
                for quantization_scale in quantization_scale_values:
                    num_bernoulli = find_smallest_N_binary_search(epsilon,p,delta,dimension,quantization_scale,ell_1_sensitivity,ell_2_sensitivity,ell_infity_sensitivity)
                    err = error(num_bernoulli,p,dimension,quantization_scale)
                    # Replace the placeholders with the actual values
                    table_row = table_format_string.format(
                        epsilon=epsilon,
                        delta=delta,
                        dimension=dimension,
                        ell_1_sensitivity=ell_1_sensitivity,
                        ell_2_sensitivity=ell_2_sensitivity,
                        ell_infity_sensitivity=ell_infity_sensitivity,
                        quantization_scale=quantization_scale,
                        num_bernoulli=num_bernoulli,
                        err=err
                    )
                    # Print the table row
                    print(table_row)

parameter_table()




def binomial_pdf(p,n,k):
    return math.comb(n, k) * p**k * (1-p)**(n-k)

def binomial_tails(p,n):
    # probability that a binomial is more than 5 standard_deviations from its mean
    standard_deviation = math.sqrt(n * p * (1-p))
    mean = n * p

    lower_bound = mean - 5 * standard_deviation
    upper_bound = mean + 5 * standard_deviation

    outer_prob = 0
    for i in range(0,math.ceil(lower_bound)):
        outer_prob += binomial_pdf(p,n,i)

    return 2 * outer_prob

# Binomial binomial tails
def binomial_tail_bound():
    p = 1/2
    desired_epsilon  = 3.1
    delta = 10**(-6)
    d = 1
    s = 1
    Delta_1 = 32
    Delta_2 = 32
    Delta_infty = 32
    smallest_N = find_smallest_N_binary_search(desired_epsilon,p,delta,d,s,Delta_1,Delta_2,Delta_infty)
    print("smallest N",smallest_N)
    print("binomial tail bound", binomial_tails(p,smallest_N))



# binomial_tail_bound()

# aggregation_p_one_half()
# aggregation_s_10th()
# aggregation_s_100th()

# aggregation_p_one_forth()
# aggregation_p_three_forths()


#walr
# walr(1,0.5)
# walr(10,0.5)
# walr(100,0.5)
# walr(.1,0.5)



# Output is
# with p =  0.5
# smallest_N = 1483
# error = 370.75

# with p =  0.25
# smallest_N = 1978
# error = 370.875

# with p =  0.75
# smallest_N = 1978
# error = 370.875


# so it seems that there may not be a gain in utility (decreasing error) by decreasing p (at least in the 1 dimensional case)





# tests
def simple_aggregation_case():
    delta = 10**(-6)
    d = 1
    s = 1
    p = 1/2
    Delta_1 = 1
    Delta_2 = 1
    Delta_infty = 1
    N = 2000
    assert(delta_contraint(N,p,d,s,delta,Delta_infty))
    eps = epsilon(N,p,delta, s, d, Delta_1, Delta_2, Delta_infty)
    # print("epsilon ", eps)
    # print("error", error(N,p,d,s))
    return eps
assert(simple_aggregation_case() > 0.6375 and simple_aggregation_case() < 0.6376)

#
# def test_functions_of_p():
#     p = 0.5
#     print("b_p: " , b_p(p))
#     print("c_p: " , c_p(p))
#     print("d_p: " , d_p(p))
#
#     assert(b_p(p) == 1/3)
#     assert(c_p(p) == 5/2)
#     assert(d_p(p) == 2/3)
#
# test_functions_of_p()


