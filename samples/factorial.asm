.data
newline: .asciiz "\n"
var_result: .word 0
var_n: .word 0

.text
.globl main
main:
  addiu $sp, $sp, -4     # space for $ra
  sw $ra, 0($sp)         # save return address
  move $fp, $sp          # frame pointer
  addiu $sp, $sp, -4     # local variables
  li $v0, 5
  la $t8, var_n
  sw $v0, 0($t8)         # store to global n
  la $t8, var_n
  lw $v0, 0($t8)         # load global n
  addiu $sp, $sp, -4
  sw $v0, 0($sp)
  jal proc_fact
  addiu $sp, $sp, 4
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


proc_fact:
  addiu $sp, $sp, -8     # space for $fp + $ra
  sw $fp, 0($sp)         # save old $fp
  sw $ra, 4($sp)         # save return address
  move $fp, $sp          # frame pointer
  addiu $sp, $sp, -8     # locals
  li $v0, 2
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  lw $v0, 8($fp)       # load m
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, else_0
  li $v0, 1
  la $t8, var_result
  sw $v0, 0($t8)         # store to global result
  j endif_1
else_0:
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  lw $v0, 8($fp)       # load m
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  subu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)
  jal proc_fact
  addiu $sp, $sp, 4
  la $t8, var_result
  lw $v0, 0($t8)         # load global result
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  lw $v0, 8($fp)       # load m
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  mul $t7, $v0, $t0
  move $v0, $t7
  la $t8, var_result
  sw $v0, 0($t8)         # store to global result
endif_1:
  addiu $sp, $sp, 8      # deallocate locals
  lw $fp, 0($sp)         # restore old $fp
  lw $ra, 4($sp)         # restore $ra
  addiu $sp, $sp, 8      # deallocate $fp + $ra slots
  jr $ra                  # return
