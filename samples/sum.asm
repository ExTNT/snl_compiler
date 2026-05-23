.data
newline: .asciiz "\n"
var_n: .word 0
var_total: .word 0
var_i: .word 0

.text
.globl main
main:
  addiu $sp, $sp, -4     # space for $ra
  sw $ra, 0($sp)         # save return address
  move $fp, $sp          # frame pointer
  addiu $sp, $sp, -4     # local variables
  li $v0, 10
  la $t8, var_n
  sw $v0, 0($t8)         # store to global n
  li $v0, 0
  la $t8, var_total
  sw $v0, 0($t8)         # store to global total
  li $v0, 1
  la $t8, var_i
  sw $v0, 0($t8)         # store to global i
loop_0:
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_n
  lw $v0, 0($t8)         # load global n
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_i
  lw $v0, 0($t8)         # load global i
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, endloop_1
  la $t8, var_i
  lw $v0, 0($t8)         # load global i
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_total
  lw $v0, 0($t8)         # load global total
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  la $t8, var_total
  sw $v0, 0($t8)         # store to global total
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
  la $t8, var_total
  lw $v0, 0($t8)         # load global total
  move $a0, $v0          # value to print
  li $v0, 1              # print int syscall
  syscall
  la $a0, newline
  li $v0, 4              # print string syscall
  syscall
  li $v0, 10             # exit syscall
  syscall

