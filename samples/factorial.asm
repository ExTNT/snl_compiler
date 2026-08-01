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
  li $v0, 5
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_n
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  la $t0, var_n
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)
  move $t0, $zero          # top-level static link
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # static link
  jal proc_fact
  addiu $sp, $sp, 8
  la $t0, var_result
  lw $v0, 0($t0)
  move $a0, $v0          # value to print
  li $v0, 1              # print int syscall
  syscall
  la $a0, newline
  li $v0, 4              # print string syscall
  syscall
main_exit:
  li $v0, 10             # exit syscall
  syscall


proc_fact:
  addiu $sp, $sp, -8     # space for $fp + $ra
  sw $fp, 0($sp)         # save old $fp
  sw $ra, 4($sp)         # save return address
  move $fp, $sp          # frame pointer
  li $v0, 2
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  move $t0, $fp
  addiu $t0, $t0, 12
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, else_1
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_result
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  j endif_2
else_1:
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  move $t0, $fp
  addiu $t0, $t0, 12
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  subu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)
  move $t0, $zero          # top-level static link
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # static link
  jal proc_fact
  addiu $sp, $sp, 8
  la $t0, var_result
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  move $t0, $fp
  addiu $t0, $t0, 12
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  mul $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_result
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
endif_2:
__snl_epilogue_0:
  move $sp, $fp          # discard locals
  lw $fp, 0($sp)         # restore old $fp
  lw $ra, 4($sp)         # restore $ra
  addiu $sp, $sp, 8      # deallocate $fp + $ra slots
  jr $ra                  # return
