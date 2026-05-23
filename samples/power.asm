.data
newline: .asciiz "\n"
var_base: .word 0
var_exp: .word 0
var_result: .word 0
var_i: .word 0

.text
.globl main
main:
  addiu $sp, $sp, -4     # space for $ra
  sw $ra, 0($sp)         # save return address
  move $fp, $sp          # frame pointer
  addiu $sp, $sp, -4     # local variables
  li $v0, 2
  la $t8, var_base
  sw $v0, 0($t8)         # store to global base
  li $v0, 8
  la $t8, var_exp
  sw $v0, 0($t8)         # store to global exp
  li $v0, 1
  la $t8, var_result
  sw $v0, 0($t8)         # store to global result
  li $v0, 0
  la $t8, var_i
  sw $v0, 0($t8)         # store to global i
loop_0:
  la $t8, var_exp
  lw $v0, 0($t8)         # load global exp
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_i
  lw $v0, 0($t8)         # load global i
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, endloop_1
  la $t8, var_base
  lw $v0, 0($t8)         # load global base
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_result
  lw $v0, 0($t8)         # load global result
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  mul $t7, $v0, $t0
  move $v0, $t7
  la $t8, var_result
  sw $v0, 0($t8)         # store to global result
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
  la $t8, var_result
  lw $v0, 0($t8)         # load global result
  move $a0, $v0          # value to print
  li $v0, 1              # print int syscall
  syscall
  la $a0, newline
  li $v0, 4              # print string syscall
  syscall
  li $v0, 10             # exit syscall
  syscall

