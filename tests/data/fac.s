    .text                  # recursive implementation of factorial
    .globl _start
fact: # arg: n in a0, returns n! in a1
    addi   sp, sp, -8      # reserve our stack area
    sw     ra, 0(sp)       # save the return address
    li     t0, 2
    blt    a0, t0, ret_one # 0! and 1! == 1
    sw     a0, 4(sp)       # save our n
    addi   a0, a0, -1
    jal    fact            # call fact (n-1)
# a1 <- fact(n-1)
    lw     t0, 4(sp)       # t0 <- n
    mul    a1, t0, a1      # a1 <- n * fact(n-1)
    j      done
ret_one:
    li     a1, 1
done:
    lw     ra, 0(sp)       # restore return address from stack
    addi   sp, sp, 8       # free our stack frame
    jr     ra              # and return
_start:
    li     a0, 5           # compute 5!
    jal    fact
    addi   a0, a1, 0       # put return value into a0 for print
    li     a7, 1           # print it
    ecall
    li     a7, 10
    ecall                  # and exit
