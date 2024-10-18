        .data
argument:
        .word   3
str1:
        .string "Fibonacci("
str2:
        .string ") = "

        .globl  _start

        .text
_start: # initial value
        lw      a0, argument    # n = 10
        li      s0, 1           # for comparison with n (n <= 1)
        jal     ra, fib         # call fib(10)

        mv      a1, a0          # a1 : final falue
        lw      a0, argument    # a0 : argument
        jal     ra, printResult # print result

        j       exit            # go to exit

fib:
        ble     a0, s0, L1      # if(n <= 1)
        addi    sp, sp, -12     # push the stack
        sw      ra, 8(sp)       # store return address
        sw      a0, 4(sp)       # store argument n
        addi    a0, a0, -1      # argument = n - 1
        jal     ra, fib         # call fib(n - 1)
        sw      a0, 0(sp)       # store return value of fib(n - 1)
        lw      a0, 4(sp)       # load argument n
        addi    a0, a0, -2      # argument = n - 2
        jal     ra, fib         # call fib(n - 2)
        lw      t0, 0(sp)       # load return value of fib(n - 1)
        add     a0, a0, t0      # fib(n - 1) + fib(n - 2)
        lw      ra, 8(sp)       # load return address
        addi    sp, sp, 12      # pop the stack
        ret                     # return

L1:
        ret                     # return

printResult: # Fibonacci(10) = 55
        mv      t0, a0
        mv      t1, a1
        la      a0, str1
        li      a7, 4
        ecall                   # print string str1
        mv      a0, t0
        li      a7, 1
        ecall                   # print int argument n
        la      a0, str2
        li      a7, 4
        ecall                   # print string str2
        mv      a0, t1
        li      a7, 1
        ecall                   # print int result
        ret

exit:
        li      a7, 10
        ecall                   # exit
