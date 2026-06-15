.data
newline: .asciiz "\n"
  .align 2
var_a: .space 40
var_target: .word 0
var_lo: .word 0
var_hi: .word 0
var_mid: .word 0
var_found: .word 0
var_result: .word 0

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
  la $t0, var_a
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
  li $v0, 5
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_a
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
  li $v0, 8
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_a
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
  li $v0, 12
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_a
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
  li $v0, 16
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_a
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
  li $v0, 23
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 5
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 38
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 6
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 45
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 7
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 56
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 8
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 67
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 9
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 38
  la $t8, var_target
  sw $v0, 0($t8)         # store to global target
  li $v0, 0
  la $t8, var_lo
  sw $v0, 0($t8)         # store to global lo
  li $v0, 9
  la $t8, var_hi
  sw $v0, 0($t8)         # store to global hi
  li $v0, 0
  la $t8, var_found
  sw $v0, 0($t8)         # store to global found
  li $v0, 0
  la $t8, var_result
  sw $v0, 0($t8)         # store to global result
loop_0:
  li $v0, 0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_found
  lw $v0, 0($t8)         # load global found
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  beq $v0, $t0, eq_true_2
  li $v0, 0
  j eq_end_3
eq_true_2:
  li $v0, 1
eq_end_3:
  beqz $v0, endloop_1
  la $t8, var_hi
  lw $v0, 0($t8)         # load global hi
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_lo
  lw $v0, 0($t8)         # load global lo
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, else_4
  li $v0, 2
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_hi
  lw $v0, 0($t8)         # load global hi
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_lo
  lw $v0, 0($t8)         # load global lo
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  div $v0, $v0, $t0
  mflo $v0
  la $t8, var_mid
  sw $v0, 0($t8)         # store to global mid
  la $t8, var_target
  lw $v0, 0($t8)         # load global target
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t8, var_mid
  lw $v0, 0($t8)         # load global mid
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
  la $t8, var_mid
  lw $v0, 0($t8)         # load global mid
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  la $t8, var_lo
  sw $v0, 0($t8)         # store to global lo
  j endif_7
else_6:
  la $t8, var_lo
  lw $v0, 0($t8)         # load global lo
  la $t8, var_lo
  sw $v0, 0($t8)         # store to global lo
endif_7:
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t8, var_mid
  lw $v0, 0($t8)         # load global mid
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_target
  lw $v0, 0($t8)         # load global target
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, else_8
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_mid
  lw $v0, 0($t8)         # load global mid
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  subu $v0, $v0, $t0
  la $t8, var_hi
  sw $v0, 0($t8)         # store to global hi
  j endif_9
else_8:
  la $t8, var_hi
  lw $v0, 0($t8)         # load global hi
  la $t8, var_hi
  sw $v0, 0($t8)         # store to global hi
endif_9:
  la $t8, var_target
  lw $v0, 0($t8)         # load global target
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t8, var_mid
  lw $v0, 0($t8)         # load global mid
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
  la $t8, var_found
  sw $v0, 0($t8)         # store to global found
  la $t8, var_mid
  lw $v0, 0($t8)         # load global mid
  la $t8, var_result
  sw $v0, 0($t8)         # store to global result
  j endif_11
else_10:
  la $t8, var_result
  lw $v0, 0($t8)         # load global result
  la $t8, var_result
  sw $v0, 0($t8)         # store to global result
endif_11:
  j endif_5
else_4:
  la $t8, var_target
  lw $v0, 0($t8)         # load global target
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t8, var_lo
  lw $v0, 0($t8)         # load global lo
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  beq $v0, $t0, eq_true_16
  li $v0, 0
  j eq_end_17
eq_true_16:
  li $v0, 1
eq_end_17:
  beqz $v0, else_14
  li $v0, 1
  la $t8, var_found
  sw $v0, 0($t8)         # store to global found
  la $t8, var_lo
  lw $v0, 0($t8)         # load global lo
  la $t8, var_result
  sw $v0, 0($t8)         # store to global result
  j endif_15
else_14:
  li $v0, 2
  la $t8, var_found
  sw $v0, 0($t8)         # store to global found
endif_15:
endif_5:
  la $t8, var_lo
  lw $v0, 0($t8)         # load global lo
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_hi
  lw $v0, 0($t8)         # load global hi
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, else_18
  li $v0, 2
  la $t8, var_found
  sw $v0, 0($t8)         # store to global found
  j endif_19
else_18:
  la $t8, var_found
  lw $v0, 0($t8)         # load global found
  la $t8, var_found
  sw $v0, 0($t8)         # store to global found
endif_19:
  j loop_0
endloop_1:
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_found
  lw $v0, 0($t8)         # load global found
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  beq $v0, $t0, eq_true_22
  li $v0, 0
  j eq_end_23
eq_true_22:
  li $v0, 1
eq_end_23:
  beqz $v0, else_20
  la $t8, var_target
  lw $v0, 0($t8)         # load global target
  move $a0, $v0          # value to print
  li $v0, 1              # print int syscall
  syscall
  la $a0, newline
  li $v0, 4              # print string syscall
  syscall
  la $t8, var_result
  lw $v0, 0($t8)         # load global result
  move $a0, $v0          # value to print
  li $v0, 1              # print int syscall
  syscall
  la $a0, newline
  li $v0, 4              # print string syscall
  syscall
  j endif_21
else_20:
  li $v0, 0
  la $t8, var_result
  sw $v0, 0($t8)         # store to global result
endif_21:
  li $v0, 10             # exit syscall
  syscall

