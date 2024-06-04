# binomial with variable p
# implement calculations to instantiation Thm 1 of https://arxiv.org/pdf/1805.10559
import math

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
def check_max(N,p,d,s,delta,Delta_infty):
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
        if check_max(N,p,d,s,delta,Delta_infty):
            if desired_epsilon >= epsilon(N,p,delta, s, d, Delta_1, Delta_2, Delta_infty):
                return N
    print("smallest N not found")
    return -1

def find_smallest_N_binary_search(desired_epsilon,p,delta,d,s,Delta_1,Delta_2,Delta_infty):
    lower = 1
    higher = 10000000
    index = 0

    while(lower <= higher):
        mid = math.floor((higher - lower)/2) + lower
        if(check_max(mid,p,d,s,delta,Delta_infty) and (desired_epsilon >= epsilon(mid,p,delta, s, d, Delta_1, Delta_2, Delta_infty))):
            index = mid
            higher = mid - 1
        else:
            lower = mid + 1
    return index


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





aggregation_p_one_half()
aggregation_p_one_forth()
aggregation_p_three_forths()

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


# so it seems that there may not be a gain in utility (decreasing error) by decreasing p (at least in the 1 dimension case)





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
    assert(check_max(N,p,d,s,delta,Delta_infty))
    eps = epsilon(N,p,delta, s, d, Delta_1, Delta_2, Delta_infty)
    # print("epsilon ", eps)
    # print("error", error(N,p,d,s))
    return eps
assert(simple_aggregation_case() > 0.6375 and simple_aggregation_case() < 0.6376)
