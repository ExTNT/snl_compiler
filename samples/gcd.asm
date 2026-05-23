.data
newline: .asciiz "\n"
var_a: .word 0
var_b: .word 0
var_done: .word 0

.text
.globl main
main:
  addiu $sp, $sp, -4     # space for $ra
  sw $ra, 0($sp)         # save return address
  move $fp, $sp          # frame pointer
  addiu $sp, $sp, -4     # local variables
  li $v0, 48
  la $t8, var_a
  sw $v0, 0($t8)         # store to global a
  li $v0, 18
  la $t8, var_b
  sw $v0, 0($t8)         # store to global b
  li $v0, 0
  la $t8, var_done
  sw $v0, 0($t8)         # store to global done
loop_0:
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_done
  lw $v0, 0($t8)         # load global done
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, endloop_1
  la $t8, var_b
  lw $v0, 0($t8)         # load global b
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_a
  lw $v0, 0($t8)         # load global a
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, else_2
  la $t8, var_a
  lw $v0, 0($t8)         # load global a
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_b
  lw $v0, 0($t8)         # load global b
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  subu $v0, $v0, $t0
  la $t8, var_b
  sw $v0, 0($t8)         # store to global b
  j endif_3
else_2:
  la $t8, var_a
  lw $v0, 0($t8)         # load global a
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_b
  lw $v0, 0($t8)         # load global b
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, else_4
  la $t8, var_b
  lw $v0, 0($t8)         # load global b
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_a
  lw $v0, 0($t8)         # load global a
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  subu $v0, $v0, $t0
  la $t8, var_a
  sw $v0, 0($t8)         # store to global a
  j endif_5
else_4:
  li $v0, 1
  la $t8, var_done
  sw $v0, 0($t8)         # store to global done
endif_5:
endif_3:
  j loop_0
endloop_1:
  la $t8, var_a
  lw $v0, 0($t8)         # load global a
  move $a0, $v0          # value to print
  li $v0, 1              # print int syscall
  syscall
  la $a0, newline
  li $v0, 4              # print string syscall
  syscall
  li $v0, 10             # exit syscall
  syscall

