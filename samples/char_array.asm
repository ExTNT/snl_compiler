.data
newline: .asciiz "\n"
  .align 2
var_s: .space 5
var_i: .word 0

.text
.globl main
main:
  addiu $sp, $sp, -4     # space for $ra
  sw $ra, 0($sp)         # save return address
  move $fp, $sp          # frame pointer
  li $v0, 72
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_s
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 0
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sb $v0, 0($t0)
  li $v0, 101
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_s
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 1
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sb $v0, 0($t0)
  li $v0, 108
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_s
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sb $v0, 0($t0)
  li $v0, 108
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_s
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 3
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sb $v0, 0($t0)
  li $v0, 111
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_s
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 4
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sb $v0, 0($t0)
  li $v0, 0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_i
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
loop_0:
  li $v0, 5
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_i
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, endloop_1
  la $t0, var_s
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t0, var_i
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lb $v0, 0($t0)
  move $a0, $v0          # value to print
  li $v0, 11             # print char syscall
  syscall
  la $a0, newline
  li $v0, 4              # print string syscall
  syscall
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_i
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_i
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  j loop_0
endloop_1:
main_exit:
  li $v0, 10             # exit syscall
  syscall

