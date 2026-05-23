.data
newline: .asciiz "\n"
var_n: .word 0
var_d: .word 0
var_primeFlag: .word 0
var_remainder: .word 0

.text
.globl main
main:
  addiu $sp, $sp, -4     # space for $ra
  sw $ra, 0($sp)         # save return address
  move $fp, $sp          # frame pointer
  addiu $sp, $sp, -4     # local variables
  li $v0, 17
  la $t8, var_n
  sw $v0, 0($t8)         # store to global n
  li $v0, 2
  la $t8, var_d
  sw $v0, 0($t8)         # store to global d
  li $v0, 1
  la $t8, var_primeFlag
  sw $v0, 0($t8)         # store to global primeFlag
loop_0:
  la $t8, var_n
  lw $v0, 0($t8)         # load global n
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_d
  lw $v0, 0($t8)         # load global d
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, endloop_1
  la $t8, var_n
  lw $v0, 0($t8)         # load global n
  la $t8, var_remainder
  sw $v0, 0($t8)         # store to global remainder
loop_2:
  la $t8, var_remainder
  lw $v0, 0($t8)         # load global remainder
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_d
  lw $v0, 0($t8)         # load global d
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, endloop_3
  la $t8, var_d
  lw $v0, 0($t8)         # load global d
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_remainder
  lw $v0, 0($t8)         # load global remainder
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  subu $v0, $v0, $t0
  la $t8, var_remainder
  sw $v0, 0($t8)         # store to global remainder
  j loop_2
endloop_3:
  la $t8, var_d
  lw $v0, 0($t8)         # load global d
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_remainder
  lw $v0, 0($t8)         # load global remainder
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, else_4
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_d
  lw $v0, 0($t8)         # load global d
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  la $t8, var_d
  sw $v0, 0($t8)         # store to global d
  j endif_5
else_4:
  li $v0, 0
  la $t8, var_primeFlag
  sw $v0, 0($t8)         # store to global primeFlag
  la $t8, var_n
  lw $v0, 0($t8)         # load global n
  la $t8, var_d
  sw $v0, 0($t8)         # store to global d
endif_5:
  j loop_0
endloop_1:
  la $t8, var_primeFlag
  lw $v0, 0($t8)         # load global primeFlag
  move $a0, $v0          # value to print
  li $v0, 1              # print int syscall
  syscall
  la $a0, newline
  li $v0, 4              # print string syscall
  syscall
  li $v0, 10             # exit syscall
  syscall

