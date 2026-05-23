.data
newline: .asciiz "\n"
var_x: .word 0
var_y: .word 0

.text
.globl main
main:
  addiu $sp, $sp, -4     # space for $ra
  sw $ra, 0($sp)         # save return address
  move $fp, $sp          # frame pointer
  addiu $sp, $sp, -4     # local variables
  li $v0, 10
  la $t8, var_x
  sw $v0, 0($t8)         # store to global x
  li $v0, 5
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_x
  lw $v0, 0($t8)         # load global x
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  la $t8, var_y
  sw $v0, 0($t8)         # store to global y
  la $t8, var_x
  lw $v0, 0($t8)         # load global x
  move $a0, $v0          # value to print
  li $v0, 1              # print int syscall
  syscall
  la $a0, newline
  li $v0, 4              # print string syscall
  syscall
  la $t8, var_y
  lw $v0, 0($t8)         # load global y
  move $a0, $v0          # value to print
  li $v0, 1              # print int syscall
  syscall
  la $a0, newline
  li $v0, 4              # print string syscall
  syscall
  li $v0, 10             # exit syscall
  syscall

