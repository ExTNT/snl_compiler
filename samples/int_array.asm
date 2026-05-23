.data
newline: .asciiz "\n"
  .align 2
var_a: .space 20
var_i: .word 0

.text
.globl main
main:
  addiu $sp, $sp, -4     # space for $ra
  sw $ra, 0($sp)         # save return address
  move $fp, $sp          # frame pointer
  addiu $sp, $sp, -4     # local variables
  li $v0, 1
  la $t8, var_i
  sw $v0, 0($t8)         # store to global i
loop_0:
  li $v0, 6
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_i
  lw $v0, 0($t8)         # load global i
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, endloop_1
  li $v0, 2
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_i
  lw $v0, 0($t8)         # load global i
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  mul $t7, $v0, $t0
  move $v0, $t7
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t8, var_i
  lw $v0, 0($t8)         # load global i
  addiu $v0, $v0, -1
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
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 1
  addiu $v0, $v0, -1
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
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 3
  addiu $v0, $v0, -1
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
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 5
  addiu $v0, $v0, -1
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

