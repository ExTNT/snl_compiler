# SNL 编译器程序流程说明

## 1. 整体架构

```
SNL 源程序 (.snl)
    │
    ▼
┌─────────────────┐
│词法分析 (Lexer) │  →  Token 序列 ──→ token.md
└─────────────────┘
    │
    ▼
┌─────────────────┐
│语法分析 (Parser)│  →  抽象语法树 ──→ tree.md
│递归下降 + LL(1) │
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ 语义分析        │  →  符号表 ────→ table.md
│  (Semantic)     │  →  语义错误信息
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ 代码生成        │  →  MIPS 汇编 (.asm)
│ (Codegen)       │
└─────────────────┘
```

四个阶段均生成诊断输出 Markdown 文件，便于分步检查编译中间结果。

## 2. 主程序流程 (main.rs)

```
开始
  │
  ▼
读取 .snl 源文件
  │
  ▼
Phase 1: 词法分析 ──→ 输出 token.md
  │ 有错误? ──→ 打印错误并退出
  │ 通过
  ▼
Phase 2: 语法分析 (递归下降) ──→ 输出 tree.md
  │ 有错误? ──→ 打印错误
  │ (语法错误不阻止后续阶段)
  ▼
Phase 3: 语义分析 (两遍遍历) ──→ 输出 table.md
  │ 有错误? ──→ 打印错误并退出
  │ 通过
  ▼
Phase 4: MIPS 代码生成
  │
  ▼
写入 .asm 文件
```

## 3. 词法分析模块 (src/lexer/)

**输入**: SNL 源程序字符串
**输出**: Token 序列

```
源程序字符串 → DFA 状态机 → Token 序列 → 关键字检查
```

**DFA 状态转换**:

| 状态 | 触发字符 | 下一状态 | 说明 |
|------|---------|---------|------|
| Start | 字母 | InIdent | 标识符/关键字 |
| Start | 数字 | InNumber | 整型常量 |
| Start | `:` | InAssign | 赋值符 `:=` |
| Start | `.` | InRange | 范围符 `..` |
| Start | `{` | InComment | 注释 |
| Start | `'` | InChar | 字符常量 |
| Start | 其他 | Done | 单字符 Token |

**最长匹配策略**: DFA 持续读入直到遇到不能继续的字符，回溯到上一个接受状态。

**识别的 Token 种类**: 21 个关键字、标识符、整型常量、字符常量、单/双字符分界符、EOF

## 4. 语法分析模块 (src/parser/)

### 4.1 递归下降分析器 (rd.rs)

**输入**: Token 序列
**输出**: 抽象语法树 (AST) + 语法错误信息

```
Token 序列 → parse_program
              ├── parse_declare_part
              │   ├── parse_type_dec    (类型声明)
              │   ├── parse_var_dec     (变量声明)
              │   └── parse_proc_dec    (过程声明, 递归)
              └── parse_program_body
                  └── parse_stm_list
                      ├── parse_if_stm
                      ├── parse_while_stm
                      ├── parse_read / parse_write
                      ├── parse_return_stm
                      ├── parse_assign_or_call
                      └── parse_exp (递归下降处理优先级)
```

**表达式优先级** (由松到紧):
```
RelExp (<, =)  >  Exp (+, -)  >  Term (*, /)  >  Factor (常量/变量/括号)
```

**恐慌模式错误恢复**: 遇到语法错误时跳过 Token 直到同步符号 (`;`, `end`, `fi`, `endwh`)

### 4.2 LL(1) 分析器 (ll1.rs)

**输入**: Token 序列
**输出**: 语法错误检查信息

```
文法定义 (grammar.rs) → FIRST/FOLLOW 计算 (first_follow.rs)
    → 分析表构建 (parse_table.rs) → 表驱动解析 (ll1.rs)
```

**文法规模**: ~70 条产生式, 40+ 个非终结符

**左因子化**: `Stm → ID AssignRest` 与 `Stm → ID ( ActParamList )` 共用 `Ident` 前缀，引入 `AssCall` 非终结符消除冲突。

## 5. 语义分析模块 (src/semantic/)

**输入**: 抽象语法树 (AST)
**输出**: 语义错误信息 + 符号表

**两遍遍历**:

```
AST → 第一遍: 符号表构建
       ├── 全局作用域: 类型名、全局变量、过程名
       ├── 进入过程体: 压入新作用域 (嵌套层级 +1)
       └── 退出过程体: 弹出作用域 (快照保存)

    → 第二遍: 语义检查
       ├── 1.  标识符重复定义
       ├── 2.  无声明的标识符
       ├── 3.  标识符类别错误
       ├── 4.  数组下标越界
       ├── 5.  数组成员/域变量引用不合法
       ├── 6.  赋值类型不兼容
       ├── 7.  赋值左端非变量
       ├── 8.  形实参类型不匹配
       ├── 9.  形实参个数不相同
       ├── 10. 非过程标识符调用
       ├── 11. 条件表达式非整型
       └── 12. 运算符分量类型不兼容
```

**作用域快照**: 每次 `exit_scope()` 前克隆当前作用域，保存到 `scope_snapshots` 列表中，用于诊断输出。

**词法作用域**: 查找标识符时从最内层作用域向外搜索。

## 6. 目标代码生成模块 (src/codegen/mips.rs)

### 6.1 类型系统

支持 SNL 全部四种数据类型:

| 类型 | 分配大小 | 存取指令 | I/O 系统调用 |
|------|---------|---------|------------|
| `integer` | 4 字节 | `lw` / `sw` | read: 5, write: 1 |
| `char` | 4 字节 | `lb` / `sb` | read: 12, write: 11 |
| `array[lo..hi] of T` | (hi-lo+1) × elem_size | 下标计算 + `lw`/`sw`/`lb`/`sb` | — |
| `record ... end` | 字段大小之和 | 偏移量 + `lw`/`sw`/`lb`/`sb` | — |

**关键实现**:
- `CodegenType`: 内部类型表示, 含 `size_of()`, `element_byte_size()`, `field_offsets()`
- `emit_var_address()`: 运行时计算 `VarAccess` (含数组下标和记录字段选择器) 地址到 `$t0`
- 类型别名通过 `build_type_alias_map()` 解析
- 数组元素大小: integer → `sll` 乘以 4, char → 不位移 (字节偏移)

### 6.2 总体流程

```
Program AST
  ├── 生成 .data 段
  │   ├── 全局变量: var_X: .word 0 (4字节) 或 .space N (数组/记录)
  │   └── 换行串: newline: .asciiz "\n"
  ├── 生成 main 过程
  │   ├── 序言: 保存 $ra, 设置 $fp, 分配局部变量
  │   ├── 编译语句体
  │   └── 尾声: exit syscall (v0=10)
  └── 编译所有 procedure
      ├── 序言: 保存 old $fp + $ra, 设置 $fp, 分配局部变量
      ├── 合并局部类型别名 (局部覆盖外层)
      ├── 分配参数 (在调用者栈帧中, $fp 上方)
      ├── 编译语句体
      ├── 递归编译嵌套过程
      └── 尾声: 释放局部变量, 恢复 $fp/$ra, jr $ra
```

### 6.3 MIPS 栈帧布局

```
main 栈帧:
  $fp → [保存的 $ra]       0($fp)
         [局部变量...]      -4($fp), -8($fp), ...

procedure 栈帧:
  $fp → [保存的旧 $fp]     0($fp)
         [保存的 $ra]       4($fp)
         [局部变量...]      -8($fp), -12($fp), ...
         参数 (调用者压入)   8($fp) = 第一个参数
```

### 6.4 变量访问策略

| 变量层级 | 存储位置 | 访问方式 |
|---------|---------|---------|
| 全局变量 (level 0) | `.data` 段 | `la $t8, var_X` → `lw/sw/lb/sb $v0, 0($t8)` |
| 局部变量/参数 (level > 0) | 栈上 `$fp` 相对 | `lw/sw/lb/sb $v0, offset($fp)` |

### 6.5 表达式编译

```
二元运算:
  编译右操作数 → $v0 → push $v0 到栈
  编译左操作数 → $v0
  pop 到 $t0 → 执行运算: $v0 = $v0 op $t0

结果约定: 表达式结果始终在 $v0 中

变量访问 (含选择器):
  简单变量 → emit_load (直接 lw/lb)
  有下标/字段 → emit_var_address → $t0 → lw/lb $v0, 0($t0)
    ArraySubscript: 计算下标 exp → $v0, 减低下界, 乘元素大小, addu $t0
    Field: 加字段偏移到 $t0
    FieldSubscript: 加字段偏移 + 数组下标计算
```

### 6.6 语句编译对照表

| SNL 语句 | MIPS 实现 |
|---------|----------|
| `x := exp` | 编译 RHS → $v0, 简单变量用 emit_store, 有选择器则保存 RHS → emit_var_address → 恢复 RHS → sw/sb |
| `if cond then A else B fi` | 编译 cond → slt/beq, `beqz` 跳 else, `j` 跳过 else |
| `while cond do body endwh` | loop 标签 + `beqz` 跳 endloop + `j loop` |
| `read(x)` | syscall: v0=5 (int) 或 v0=12 (char), 结果存入变量 |
| `write(exp)` | 编译 exp → $a0, syscall: v0=1 (int) 或 v0=11 (char), 加 v0=4 输出换行 |
| `return(exp)` | 编译 exp, 结果留在 $v0 |
| `proc(args)` | 参数逆序压栈, `jal proc_name`, 调用后弹栈 |

### 6.7 过程调用完整序列

```
调用者:
  参数逆序压栈 (addiu + sw)
  jal proc_name

被调用者:
  addiu $sp, $sp, -8        # $fp + $ra 空间
  sw $fp, 0($sp)            # 保存旧 $fp
  sw $ra, 4($sp)            # 保存 $ra
  move $fp, $sp             # 设置新帧指针
  addiu $sp, $sp, -N        # 分配局部变量
  ...执行过程体...
  addiu $sp, $sp, N         # 释放局部变量
  lw $fp, 0($sp)            # 恢复旧 $fp
  lw $ra, 4($sp)            # 恢复 $ra
  addiu $sp, $sp, 8         # 释放 $fp + $ra
  jr $ra                    # 返回

调用者:
  addiu $sp, $sp, 4×N       # 弹出参数
```

## 7. 数据流总览 (以 factorial 为例)

```
输入: "program factorial var integer result; ... begin ... end."
  │
  ▼
词法分析:
  program → TK::Program
  factorial → Ident
  var → TK::Var
  integer → TK::Integer
  ...
  │
  ▼ 输出 token.md
语法分析 (AST):
  Program { name: factorial
    decl: DeclarePart { vars: [result, n], procs: [fact] }
    body: StmList [Assign, Call, Write] }
  │
  ▼ 输出 tree.md
语义分析:
  ✓ 符号表: result(global), n(global), fact(proc, param=m), m(local), temp(local)
  ✓ 类型检查通过
  │
  ▼ 输出 table.md
代码生成 (.asm):
  .data
  var_result: .word 0
  var_n: .word 0
  .text
  main:
    ...
    jal proc_fact
    ...
  proc_fact:
    addiu $sp, $sp, -8
    sw $fp, 0($sp)
    sw $ra, 4($sp)
    ...
```

## 8. 关键设计决策

### 8.1 `$fp` 被调用者保存

`$fp` (寄存器 `$30`/`$s8`) 属于被调用者保存寄存器。过程序言保存旧的 `$fp`，尾声恢复，是支持嵌套/递归调用时参数正确寻址的关键。

### 8.2 全局变量放在 `.data` 段

全局变量在 `.data` 段声明为带标签的数据，通过 `la $t8, var_X` 访问，避免了静态链 (static link) 的复杂实现。数组和记录使用 `.space N` 分配，且添加 `.align 2` 确保字对齐。

### 8.3 `$v0` 作为表达式结果寄存器

所有表达式计算结果统一放在 `$v0` 中。对于乘法运算，使用 `$t7` 作为中间目标寄存器 (`mul $t7, $v0, $t0; move $v0, $t7`)。

### 8.4 类型信息从 AST 派生

代码生成器不依赖语义分析器的符号表（其作用域在分析完成后已弹出），而是从 AST 声明节点直接推导类型信息，通过 `build_type_alias_map()` + `type_desig_to_codegen()` 解析。

### 8.5 `$t0` 保存/恢复

`emit_var_address` 使用 `$t0` 追踪运行时地址。由于表达式编译 (`compile_exp`) 的 Binary 分支也会使用 `$t0` 作为临时寄存器，在数组下标和记录字段下标的表达式求值前，必须将 `$t0` 压栈保存，求值后恢复。
