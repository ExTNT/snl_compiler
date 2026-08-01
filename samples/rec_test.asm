.data
newline: .asciiz "\n"
  .align 2
var_r: .space 8

.text
.globl main
main:
  addiu $sp, $sp, -4     # space for $ra
  sw $ra, 0($sp)         # save return address
  move $fp, $sp          # frame pointer
  li $v0, 100
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_r
  addiu $t0, $t0, 0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 88
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_r
  addiu $t0, $t0, 4
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sb $v0, 0($t0)
  la $t0, var_r
  addiu $t0, $t0, 0
  lw $v0, 0($t0)
  move $a0, $v0          # value to print
  li $v0, 1              # print int syscall
  syscall
  la $a0, newline
  li $v0, 4              # print string syscall
  syscall
  la $t0, var_r
  addiu $t0, $t0, 4
  lb $v0, 0($t0)
  move $a0, $v0          # value to print
  li $v0, 11             # print char syscall
  syscall
  la $a0, newline
  li $v0, 4              # print string syscall
  syscall
main_exit:
  li $v0, 10             # exit syscall
  syscall

