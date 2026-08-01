.data
newline: .asciiz "\n"
  .align 2
var_coins: .space 16
  .align 2
var_count: .space 16
var_amount: .word 0
var_m: .word 0
var_rem: .word 0
var_i: .word 0

.text
.globl main
main:
  addiu $sp, $sp, -4     # space for $ra
  sw $ra, 0($sp)         # save return address
  move $fp, $sp          # frame pointer
  li $v0, 10
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_coins
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 0
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 5
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_coins
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 1
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 2
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_coins
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 2
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_coins
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 3
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 37
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_amount
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 4
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_m
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  la $t0, var_amount
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_rem
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_i
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
loop_0:
  la $t0, var_m
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_i
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, endloop_1
  li $v0, 0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_count
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t0, var_i
  lw $v0, 0($t0)
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
loop_2:
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_rem
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_coins
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t0, var_i
  lw $v0, 0($t0)
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, endloop_3
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_count
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t0, var_i
  lw $v0, 0($t0)
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_count
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t0, var_i
  lw $v0, 0($t0)
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  la $t0, var_coins
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t0, var_i
  lw $v0, 0($t0)
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_rem
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  subu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_rem
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  j loop_2
endloop_3:
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
  li $v0, 0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_i
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
loop_4:
  la $t0, var_m
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_i
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, endloop_5
  la $t0, var_coins
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t0, var_i
  lw $v0, 0($t0)
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
  la $t0, var_count
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t0, var_i
  lw $v0, 0($t0)
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
  j loop_4
endloop_5:
  la $t0, var_rem
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

