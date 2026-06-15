.data
newline: .asciiz "\n"
  .align 2
var_wt: .space 20
  .align 2
var_val: .space 20
  .align 2
var_dp: .space 44
var_n: .word 0
var_cap: .word 0
var_i: .word 0
var_w: .word 0
var_take: .word 0
var_skip: .word 0

.text
.globl main
main:
  addiu $sp, $sp, -4     # space for $ra
  sw $ra, 0($sp)         # save return address
  move $fp, $sp          # frame pointer
  addiu $sp, $sp, -4     # local variables
  li $v0, 2
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_wt
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 0
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 3
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_wt
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 1
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 4
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_wt
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 2
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 5
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_wt
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 3
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_wt
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 4
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 3
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_val
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 0
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 4
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_val
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 1
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 5
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_val
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 2
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 8
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_val
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 3
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_val
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 4
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 5
  la $t8, var_n
  sw $v0, 0($t8)         # store to global n
  li $v0, 10
  la $t8, var_cap
  sw $v0, 0($t8)         # store to global cap
  li $v0, 0
  la $t8, var_w
  sw $v0, 0($t8)         # store to global w
loop_0:
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_cap
  lw $v0, 0($t8)         # load global cap
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_w
  lw $v0, 0($t8)         # load global w
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, endloop_1
  li $v0, 0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_dp
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t8, var_w
  lw $v0, 0($t8)         # load global w
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_w
  lw $v0, 0($t8)         # load global w
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  la $t8, var_w
  sw $v0, 0($t8)         # store to global w
  j loop_0
endloop_1:
  li $v0, 0
  la $t8, var_i
  sw $v0, 0($t8)         # store to global i
loop_2:
  la $t8, var_n
  lw $v0, 0($t8)         # load global n
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_i
  lw $v0, 0($t8)         # load global i
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, endloop_3
  la $t8, var_cap
  lw $v0, 0($t8)         # load global cap
  la $t8, var_w
  sw $v0, 0($t8)         # store to global w
loop_4:
  la $t8, var_w
  lw $v0, 0($t8)         # load global w
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  li $v0, 0
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, endloop_5
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_w
  lw $v0, 0($t8)         # load global w
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_wt
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t8, var_i
  lw $v0, 0($t8)         # load global i
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, else_6
  la $t0, var_val
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t8, var_i
  lw $v0, 0($t8)         # load global i
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_dp
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t0, var_wt
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t8, var_i
  lw $v0, 0($t8)         # load global i
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_w
  lw $v0, 0($t8)         # load global w
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  subu $v0, $v0, $t0
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  la $t8, var_take
  sw $v0, 0($t8)         # store to global take
  la $t0, var_dp
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t8, var_w
  lw $v0, 0($t8)         # load global w
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($t0)
  la $t8, var_skip
  sw $v0, 0($t8)         # store to global skip
  la $t8, var_take
  lw $v0, 0($t8)         # load global take
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_skip
  lw $v0, 0($t8)         # load global skip
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, else_8
  la $t8, var_take
  lw $v0, 0($t8)         # load global take
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_dp
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t8, var_w
  lw $v0, 0($t8)         # load global w
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  j endif_9
else_8:
  la $t8, var_take
  lw $v0, 0($t8)         # load global take
  la $t8, var_take
  sw $v0, 0($t8)         # store to global take
endif_9:
  j endif_7
else_6:
  la $t8, var_take
  lw $v0, 0($t8)         # load global take
  la $t8, var_take
  sw $v0, 0($t8)         # store to global take
endif_7:
  la $t8, var_w
  lw $v0, 0($t8)         # load global w
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_wt
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t8, var_i
  lw $v0, 0($t8)         # load global i
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  beq $v0, $t0, eq_true_12
  li $v0, 0
  j eq_end_13
eq_true_12:
  li $v0, 1
eq_end_13:
  beqz $v0, else_10
  la $t0, var_val
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t8, var_i
  lw $v0, 0($t8)         # load global i
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($t0)
  la $t8, var_take
  sw $v0, 0($t8)         # store to global take
  la $t0, var_dp
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t8, var_w
  lw $v0, 0($t8)         # load global w
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($t0)
  la $t8, var_skip
  sw $v0, 0($t8)         # store to global skip
  la $t8, var_take
  lw $v0, 0($t8)         # load global take
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_skip
  lw $v0, 0($t8)         # load global skip
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, else_14
  la $t8, var_take
  lw $v0, 0($t8)         # load global take
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_dp
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t8, var_w
  lw $v0, 0($t8)         # load global w
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  j endif_15
else_14:
  la $t8, var_take
  lw $v0, 0($t8)         # load global take
  la $t8, var_take
  sw $v0, 0($t8)         # store to global take
endif_15:
  j endif_11
else_10:
  la $t8, var_take
  lw $v0, 0($t8)         # load global take
  la $t8, var_take
  sw $v0, 0($t8)         # store to global take
endif_11:
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_w
  lw $v0, 0($t8)         # load global w
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  subu $v0, $v0, $t0
  la $t8, var_w
  sw $v0, 0($t8)         # store to global w
  j loop_4
endloop_5:
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_i
  lw $v0, 0($t8)         # load global i
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  la $t8, var_i
  sw $v0, 0($t8)         # store to global i
  j loop_2
endloop_3:
  la $t8, var_cap
  lw $v0, 0($t8)         # load global cap
  move $a0, $v0          # value to print
  li $v0, 1              # print int syscall
  syscall
  la $a0, newline
  li $v0, 4              # print string syscall
  syscall
  la $t0, var_dp
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t8, var_cap
  lw $v0, 0($t8)         # load global cap
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($t0)
  move $a0, $v0          # value to print
  li $v0, 1              # print int syscall
  syscall
  la $a0, newline
  li $v0, 4              # print string syscall
  syscall
  li $v0, 10             # exit syscall
  syscall

