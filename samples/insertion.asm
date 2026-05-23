.data
newline: .asciiz "\n"
  .align 2
var_a: .space 20
var_n: .word 0
var_i: .word 0
var_key: .word 0
var_j: .word 0
var_ipos: .word 0
var_done: .word 0

.text
.globl main
main:
  addiu $sp, $sp, -4     # space for $ra
  sw $ra, 0($sp)         # save return address
  move $fp, $sp          # frame pointer
  addiu $sp, $sp, -4     # local variables
  li $v0, 64
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
  li $v0, 34
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
  li $v0, 25
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
  li $v0, 22
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
  li $v0, 5
  la $t8, var_n
  sw $v0, 0($t8)         # store to global n
  li $v0, 1
  la $t8, var_i
  sw $v0, 0($t8)         # store to global i
loop_0:
  la $t8, var_n
  lw $v0, 0($t8)         # load global n
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_i
  lw $v0, 0($t8)         # load global i
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, endloop_1
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t8, var_i
  lw $v0, 0($t8)         # load global i
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($t0)
  la $t8, var_key
  sw $v0, 0($t8)         # store to global key
  la $t8, var_i
  lw $v0, 0($t8)         # load global i
  la $t8, var_j
  sw $v0, 0($t8)         # store to global j
  la $t8, var_i
  lw $v0, 0($t8)         # load global i
  la $t8, var_ipos
  sw $v0, 0($t8)         # store to global ipos
  li $v0, 0
  la $t8, var_done
  sw $v0, 0($t8)         # store to global done
loop_2:
  li $v0, 0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_done
  lw $v0, 0($t8)         # load global done
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  beq $v0, $t0, eq_true_4
  li $v0, 0
  j eq_end_5
eq_true_4:
  li $v0, 1
eq_end_5:
  beqz $v0, endloop_3
  li $v0, 0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_j
  lw $v0, 0($t8)         # load global j
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  beq $v0, $t0, eq_true_8
  li $v0, 0
  j eq_end_9
eq_true_8:
  li $v0, 1
eq_end_9:
  beqz $v0, else_6
  li $v0, 0
  la $t8, var_ipos
  sw $v0, 0($t8)         # store to global ipos
  li $v0, 1
  la $t8, var_done
  sw $v0, 0($t8)         # store to global done
  j endif_7
else_6:
  la $t8, var_key
  lw $v0, 0($t8)         # load global key
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_j
  lw $v0, 0($t8)         # load global j
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
  slt $v0, $v0, $t0
  beqz $v0, else_10
  la $t8, var_j
  lw $v0, 0($t8)         # load global j
  la $t8, var_ipos
  sw $v0, 0($t8)         # store to global ipos
  li $v0, 1
  la $t8, var_done
  sw $v0, 0($t8)         # store to global done
  j endif_11
else_10:
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_j
  lw $v0, 0($t8)         # load global j
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  subu $v0, $v0, $t0
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t8, var_j
  lw $v0, 0($t8)         # load global j
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
  la $t8, var_j
  lw $v0, 0($t8)         # load global j
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  subu $v0, $v0, $t0
  la $t8, var_j
  sw $v0, 0($t8)         # store to global j
endif_11:
endif_7:
  j loop_2
endloop_3:
  la $t8, var_key
  lw $v0, 0($t8)         # load global key
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t8, var_ipos
  lw $v0, 0($t8)         # load global ipos
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
  la $t8, var_i
  lw $v0, 0($t8)         # load global i
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  la $t8, var_i
  sw $v0, 0($t8)         # store to global i
  j loop_0
endloop_1:
  li $v0, 0
  la $t8, var_i
  sw $v0, 0($t8)         # store to global i
loop_12:
  la $t8, var_n
  lw $v0, 0($t8)         # load global n
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_i
  lw $v0, 0($t8)         # load global i
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, endloop_13
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t8, var_i
  lw $v0, 0($t8)         # load global i
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
  j loop_12
endloop_13:
  li $v0, 10             # exit syscall
  syscall

