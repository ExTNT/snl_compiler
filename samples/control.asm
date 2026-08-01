.data
newline: .asciiz "\n"
var_n: .word 0

.text
.globl main
main:
  addiu $sp, $sp, -4     # space for $ra
  sw $ra, 0($sp)         # save return address
  move $fp, $sp          # frame pointer
  li $v0, 10
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_n
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 20
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_n
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, else_0
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_n
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_n
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  j endif_1
else_0:
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_n
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  subu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_n
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
endif_1:
loop_2:
  li $v0, 1
  beqz $v0, endloop_3
  la $t0, var_n
  lw $v0, 0($t0)
  move $a0, $v0          # value to print
  li $v0, 1              # print int syscall
  syscall
  la $a0, newline
  li $v0, 4              # print string syscall
  syscall
  j loop_2
endloop_3:
main_exit:
  li $v0, 10             # exit syscall
  syscall

