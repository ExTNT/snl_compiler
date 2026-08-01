.data
newline: .asciiz "\n"
var_c: .word 0
var_d: .word 0

.text
.globl main
main:
  addiu $sp, $sp, -4     # space for $ra
  sw $ra, 0($sp)         # save return address
  move $fp, $sp          # frame pointer
  li $v0, 65
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_c
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sb $v0, 0($t0)
  li $v0, 66
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_d
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sb $v0, 0($t0)
  la $t0, var_c
  lb $v0, 0($t0)
  move $a0, $v0          # value to print
  li $v0, 11             # print char syscall
  syscall
  la $a0, newline
  li $v0, 4              # print string syscall
  syscall
  la $t0, var_d
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

