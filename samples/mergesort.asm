.data
newline: .asciiz "\n"
  .align 2
var_a: .space 32
  .align 2
var_tmp: .space 32
var_n: .word 0

.text
.globl main
main:
  addiu $sp, $sp, -4     # space for $ra
  sw $ra, 0($sp)         # save return address
  move $fp, $sp          # frame pointer
  li $v0, 38
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_a
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
  li $v0, 27
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_a
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
  li $v0, 43
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_a
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
  li $v0, 3
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_a
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
  li $v0, 9
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 4
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 82
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 5
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 10
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 6
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
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  li $v0, 7
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 8
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_n
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_n
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  subu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)
  li $v0, 0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)
  move $t0, $zero          # top-level static link
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # static link
  jal proc_msort
  addiu $sp, $sp, 12
  li $v0, 0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_n
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
loop_0:
  li $v0, 8
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_n
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, endloop_1
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t0, var_n
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
  j loop_0
endloop_1:
main_exit:
  li $v0, 10             # exit syscall
  syscall


proc_merge:
  addiu $sp, $sp, -8     # space for $fp + $ra
  sw $fp, 0($sp)         # save old $fp
  sw $ra, 4($sp)         # save return address
  move $fp, $sp          # frame pointer
  addiu $sp, $sp, -12
  move $t0, $fp
  addiu $t0, $t0, 12
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  move $t0, $fp
  addiu $t0, $t0, -4
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  move $t0, $fp
  addiu $t0, $t0, 16
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  move $t0, $fp
  addiu $t0, $t0, -8
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  move $t0, $fp
  addiu $t0, $t0, 12
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  move $t0, $fp
  addiu $t0, $t0, -12
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
loop_3:
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  move $t0, $fp
  addiu $t0, $t0, 20
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  move $t0, $fp
  addiu $t0, $t0, -12
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, endloop_4
  move $t0, $fp
  addiu $t0, $t0, -4
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  move $t0, $fp
  addiu $t0, $t0, 16
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, else_5
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  move $t0, $fp
  addiu $t0, $t0, -8
  lw $v0, 0($t0)
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_tmp
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  move $t0, $fp
  addiu $t0, $t0, -12
  lw $v0, 0($t0)
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  move $t0, $fp
  addiu $t0, $t0, -8
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  move $t0, $fp
  addiu $t0, $t0, -8
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  j endif_6
else_5:
  move $t0, $fp
  addiu $t0, $t0, -8
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  move $t0, $fp
  addiu $t0, $t0, 20
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, else_7
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  move $t0, $fp
  addiu $t0, $t0, -4
  lw $v0, 0($t0)
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_tmp
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  move $t0, $fp
  addiu $t0, $t0, -12
  lw $v0, 0($t0)
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  move $t0, $fp
  addiu $t0, $t0, -4
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  move $t0, $fp
  addiu $t0, $t0, -4
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  j endif_8
else_7:
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  move $t0, $fp
  addiu $t0, $t0, -8
  lw $v0, 0($t0)
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  move $t0, $fp
  addiu $t0, $t0, -4
  lw $v0, 0($t0)
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, else_9
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  move $t0, $fp
  addiu $t0, $t0, -4
  lw $v0, 0($t0)
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_tmp
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  move $t0, $fp
  addiu $t0, $t0, -12
  lw $v0, 0($t0)
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  move $t0, $fp
  addiu $t0, $t0, -4
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  move $t0, $fp
  addiu $t0, $t0, -4
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  j endif_10
else_9:
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  move $t0, $fp
  addiu $t0, $t0, -8
  lw $v0, 0($t0)
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_tmp
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  move $t0, $fp
  addiu $t0, $t0, -12
  lw $v0, 0($t0)
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  move $t0, $fp
  addiu $t0, $t0, -8
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  move $t0, $fp
  addiu $t0, $t0, -8
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
endif_10:
endif_8:
endif_6:
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  move $t0, $fp
  addiu $t0, $t0, -12
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  move $t0, $fp
  addiu $t0, $t0, -12
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  j loop_3
endloop_4:
  move $t0, $fp
  addiu $t0, $t0, 12
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  move $t0, $fp
  addiu $t0, $t0, -12
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
loop_11:
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  move $t0, $fp
  addiu $t0, $t0, 20
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  move $t0, $fp
  addiu $t0, $t0, -12
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, endloop_12
  la $t0, var_tmp
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  move $t0, $fp
  addiu $t0, $t0, -12
  lw $v0, 0($t0)
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  move $t0, $fp
  addiu $t0, $t0, -12
  lw $v0, 0($t0)
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  move $t0, $fp
  addiu $t0, $t0, -12
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  move $t0, $fp
  addiu $t0, $t0, -12
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  j loop_11
endloop_12:
__snl_epilogue_2:
  move $sp, $fp          # discard locals
  lw $fp, 0($sp)         # restore old $fp
  lw $ra, 4($sp)         # restore $ra
  addiu $sp, $sp, 8      # deallocate $fp + $ra slots
  jr $ra                  # return

proc_msort:
  addiu $sp, $sp, -8     # space for $fp + $ra
  sw $fp, 0($sp)         # save old $fp
  sw $ra, 4($sp)         # save return address
  move $fp, $sp          # frame pointer
  addiu $sp, $sp, -4
  move $t0, $fp
  addiu $t0, $t0, 16
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  move $t0, $fp
  addiu $t0, $t0, 12
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, else_14
  li $v0, 2
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  move $t0, $fp
  addiu $t0, $t0, 16
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  move $t0, $fp
  addiu $t0, $t0, 12
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  div $v0, $v0, $t0
  mflo $v0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  move $t0, $fp
  addiu $t0, $t0, -4
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
  move $t0, $fp
  addiu $t0, $t0, -4
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)
  move $t0, $fp
  addiu $t0, $t0, 12
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)
  move $t0, $zero          # top-level static link
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # static link
  jal proc_msort
  addiu $sp, $sp, 12
  move $t0, $fp
  addiu $t0, $t0, 16
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  move $t0, $fp
  addiu $t0, $t0, -4
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)
  move $t0, $zero          # top-level static link
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # static link
  jal proc_msort
  addiu $sp, $sp, 12
  move $t0, $fp
  addiu $t0, $t0, 16
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)
  move $t0, $fp
  addiu $t0, $t0, -4
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)
  move $t0, $fp
  addiu $t0, $t0, 12
  lw $v0, 0($t0)
  addiu $sp, $sp, -4
  sw $v0, 0($sp)
  move $t0, $zero          # top-level static link
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # static link
  jal proc_merge
  addiu $sp, $sp, 16
  j endif_15
else_14:
  li $v0, 0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # save rhs value
  move $t0, $fp
  addiu $t0, $t0, -4
  lw $v0, 0($sp)          # restore rhs value
  addiu $sp, $sp, 4
  sw $v0, 0($t0)
endif_15:
__snl_epilogue_13:
  move $sp, $fp          # discard locals
  lw $fp, 0($sp)         # restore old $fp
  lw $ra, 4($sp)         # restore $ra
  addiu $sp, $sp, 8      # deallocate $fp + $ra slots
  jr $ra                  # return
