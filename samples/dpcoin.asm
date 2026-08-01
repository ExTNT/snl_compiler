.data
newline: .asciiz "\n"
  .align 2
var_coins: .space 16
  .align 2
var_dp: .space 64
var_amount: .word 0
var_m: .word 0
var_i: .word 0
var_j: .word 0
var_val: .word 0
var_best: .word 0

.text
.globl main
main:
  addiu $sp, $sp, -4     # space for $ra
  sw $ra, 0($sp)         # save return address
  move $fp, $sp          # frame pointer
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_coins
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
  la $t0, var_coins
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
  la $t0, var_coins
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
  li $v0, 7
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_coins
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
  li $v0, 15
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_amount
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 4
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_m
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_dp
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
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_i
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
loop_0:
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_amount
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_i
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, endloop_1
  li $v0, 999
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_dp
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t0, var_i
  lw $v0, 0($t0)
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
  la $t0, var_i
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_i
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  j loop_0
endloop_1:
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_i
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
loop_2:
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_amount
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_i
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, endloop_3
  li $v0, 999
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_best
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_j
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
loop_4:
  la $t0, var_m
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_j
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, endloop_5
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_i
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_coins
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t0, var_j
  lw $v0, 0($t0)
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, else_6
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_dp
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t0, var_coins
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t0, var_j
  lw $v0, 0($t0)
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_i
  lw $v0, 0($t0)
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
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_val
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  la $t0, var_best
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_val
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, else_8
  la $t0, var_val
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_best
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  j endif_9
else_8:
  la $t0, var_val
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_val
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
endif_9:
  j endif_7
else_6:
  la $t0, var_val
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_val
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
endif_7:
  la $t0, var_i
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_coins
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t0, var_j
  lw $v0, 0($t0)
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
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_best
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  j endif_11
else_10:
  la $t0, var_best
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_best
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
endif_11:
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_j
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_j
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  j loop_4
endloop_5:
  la $t0, var_best
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_dp
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t0, var_i
  lw $v0, 0($t0)
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
  la $t0, var_i
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_i
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  j loop_2
endloop_3:
  la $t0, var_amount
  lw $v0, 0($t0)
  move $a0, $v0          # value to print
  li $v0, 1              # print int syscall
  syscall
  la $a0, newline
  li $v0, 4              # print string syscall
  syscall
  la $t0, var_dp
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t0, var_amount
  lw $v0, 0($t0)
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
main_exit:
  li $v0, 10             # exit syscall
  syscall

