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
  addiu $sp, $sp, -4     # local variables
  li $v0, 65
  la $t8, var_c
  sb $v0, 0($t8)         # store to global c
  li $v0, 66
  la $t8, var_d
  sb $v0, 0($t8)         # store to global d
  la $t8, var_c
  lb $v0, 0($t8)         # load global c
  move $a0, $v0          # value to print
  li $v0, 11             # print char syscall
  syscall
  la $a0, newline
  li $v0, 4              # print string syscall
  syscall
  la $t8, var_d
  lb $v0, 0($t8)         # load global d
  move $a0, $v0          # value to print
  li $v0, 11             # print char syscall
  syscall
  la $a0, newline
  li $v0, 4              # print string syscall
  syscall
  li $v0, 10             # exit syscall
  syscall

