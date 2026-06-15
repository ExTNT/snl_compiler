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
  addiu $sp, $sp, -4     # local variables
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
  la $t8, var_n
  sw $v0, 0($t8)         # store to global n
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_n
  lw $v0, 0($t8)         # load global n
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  subu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)
  li $v0, 0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)
  jal proc_msort
  addiu $sp, $sp, 8
  li $v0, 0
  la $t8, var_n
  sw $v0, 0($t8)         # store to global n
loop_0:
  li $v0, 8
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  la $t8, var_n
  lw $v0, 0($t8)         # load global n
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, endloop_1
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  la $t8, var_n
  lw $v0, 0($t8)         # load global n
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
  la $t8, var_n
  lw $v0, 0($t8)         # load global n
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  la $t8, var_n
  sw $v0, 0($t8)         # store to global n
  j loop_0
endloop_1:
  li $v0, 10             # exit syscall
  syscall


proc_merge:
  addiu $sp, $sp, -8     # space for $fp + $ra
  sw $fp, 0($sp)         # save old $fp
  sw $ra, 4($sp)         # save return address
  move $fp, $sp          # frame pointer
  addiu $sp, $sp, -20     # locals
  lw $v0, 8($fp)       # load lo
  sw $v0, -8($fp)       # store to i
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  lw $v0, 12($fp)       # load mid
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  sw $v0, -12($fp)       # store to j
  lw $v0, 8($fp)       # load lo
  sw $v0, -16($fp)       # store to k
loop_2:
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  lw $v0, 16($fp)       # load hi
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  lw $v0, -16($fp)       # load k
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, endloop_3
  lw $v0, -8($fp)       # load i
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  lw $v0, 12($fp)       # load mid
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, else_4
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  lw $v0, -12($fp)       # load j
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
  lw $v0, -16($fp)       # load k
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
  lw $v0, -12($fp)       # load j
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  sw $v0, -12($fp)       # store to j
  j endif_5
else_4:
  lw $v0, -12($fp)       # load j
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  lw $v0, 16($fp)       # load hi
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, else_6
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  lw $v0, -8($fp)       # load i
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
  lw $v0, -16($fp)       # load k
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
  lw $v0, -8($fp)       # load i
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  sw $v0, -8($fp)       # store to i
  j endif_7
else_6:
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  lw $v0, -12($fp)       # load j
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
  lw $v0, -8($fp)       # load i
  sll $v0, $v0, 2
  lw $t0, 0($sp)          # restore base address
  addiu $sp, $sp, 4
  addu $t0, $t0, $v0
  lw $v0, 0($t0)
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, else_8
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  lw $v0, -8($fp)       # load i
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
  lw $v0, -16($fp)       # load k
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
  lw $v0, -8($fp)       # load i
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  sw $v0, -8($fp)       # store to i
  j endif_9
else_8:
  la $t0, var_a
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  lw $v0, -12($fp)       # load j
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
  lw $v0, -16($fp)       # load k
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
  lw $v0, -12($fp)       # load j
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  sw $v0, -12($fp)       # store to j
endif_9:
endif_7:
endif_5:
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  lw $v0, -16($fp)       # load k
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  sw $v0, -16($fp)       # store to k
  j loop_2
endloop_3:
  lw $v0, 8($fp)       # load lo
  sw $v0, -16($fp)       # store to k
loop_10:
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  lw $v0, 16($fp)       # load hi
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  lw $v0, -16($fp)       # load k
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, endloop_11
  la $t0, var_tmp
  addiu $sp, $sp, -4
  sw $t0, 0($sp)          # save base address
  lw $v0, -16($fp)       # load k
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
  lw $v0, -16($fp)       # load k
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
  lw $v0, -16($fp)       # load k
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  sw $v0, -16($fp)       # store to k
  j loop_10
endloop_11:
  addiu $sp, $sp, 20      # deallocate locals
  lw $fp, 0($sp)         # restore old $fp
  lw $ra, 4($sp)         # restore $ra
  addiu $sp, $sp, 8      # deallocate $fp + $ra slots
  jr $ra                  # return

proc_msort:
  addiu $sp, $sp, -8     # space for $fp + $ra
  sw $fp, 0($sp)         # save old $fp
  sw $ra, 4($sp)         # save return address
  move $fp, $sp          # frame pointer
  addiu $sp, $sp, -12     # locals
  lw $v0, 12($fp)       # load hi
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  lw $v0, 8($fp)       # load lo
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  slt $v0, $v0, $t0
  beqz $v0, else_12
  li $v0, 2
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  lw $v0, 12($fp)       # load hi
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  lw $v0, 8($fp)       # load lo
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  div $v0, $v0, $t0
  mflo $v0
  sw $v0, -8($fp)       # store to mid
  lw $v0, -8($fp)       # load mid
  addiu $sp, $sp, -4
  sw $v0, 0($sp)
  lw $v0, 8($fp)       # load lo
  addiu $sp, $sp, -4
  sw $v0, 0($sp)
  jal proc_msort
  addiu $sp, $sp, 8
  lw $v0, 12($fp)       # load hi
  addiu $sp, $sp, -4
  sw $v0, 0($sp)
  li $v0, 1
  addiu $sp, $sp, -4
  sw $v0, 0($sp)          # push right
  lw $v0, -8($fp)       # load mid
  lw $t0, 0($sp)          # pop right
  addiu $sp, $sp, 4
  addu $v0, $v0, $t0
  addiu $sp, $sp, -4
  sw $v0, 0($sp)
  jal proc_msort
  addiu $sp, $sp, 8
  lw $v0, 12($fp)       # load hi
  addiu $sp, $sp, -4
  sw $v0, 0($sp)
  lw $v0, -8($fp)       # load mid
  addiu $sp, $sp, -4
  sw $v0, 0($sp)
  lw $v0, 8($fp)       # load lo
  addiu $sp, $sp, -4
  sw $v0, 0($sp)
  jal proc_merge
  addiu $sp, $sp, 12
  j endif_13
else_12:
  li $v0, 0
  sw $v0, -8($fp)       # store to mid
endif_13:
  addiu $sp, $sp, 12      # deallocate locals
  lw $fp, 0($sp)         # restore old $fp
  lw $ra, 4($sp)         # restore $ra
  addiu $sp, $sp, 8      # deallocate $fp + $ra slots
  jr $ra                  # return
