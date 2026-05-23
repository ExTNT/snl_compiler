# SNL 编译器程序流程说明

## 1. 整体架构

```mermaid
flowchart TD
    SRC["SNL 源程序 (.snl)"]
    LEX["<b>词法分析 (Lexer)</b>"]
    PARSE["<b>语法分析 (Parser)</b><br/>递归下降 + LL(1)"]
    SEM["<b>语义分析 (Semantic)</b>"]
    CG["<b>代码生成 (Codegen)</b>"]

    TOKEN["token.md"]
    TREE["tree.md"]
    TABLE["table.md"]
    ASM["MIPS 汇编 (.asm)"]

    SRC --> LEX
    LEX -->|"Token 序列"| PARSE
    PARSE -->|"抽象语法树"| SEM
    SEM -->|"符号表"| CG
    CG -->|"MIPS 汇编"| ASM

    LEX -.-> TOKEN
    PARSE -.-> TREE
    SEM -.-> TABLE
    SEM -.->|"语义错误信息"| ERR["语义错误信息"]
```

四个阶段均生成诊断输出 Markdown 文件，便于分步检查编译中间结果。

## 2. 主程序流程 (main.rs)

```mermaid
flowchart TD
    START([开始]) --> READ[读取 .snl 源文件]
    READ --> P1["Phase 1: 词法分析"]
    P1 --> P1_OUT[输出 token.md]
    P1_OUT --> P1_ERR{有错误?}
    P1_ERR -->|是| EXIT1[打印错误并退出]
    P1_ERR -->|通过| P2["Phase 2: 语法分析 (递归下降)"]
    P2 --> P2_OUT[输出 tree.md]
    P2_OUT --> P2_ERR{有错误?}
    P2_ERR -->|是| PRINT[打印错误]
    P2_ERR -->|否| P3["Phase 3: 语义分析 (两遍遍历)"]
    PRINT --> P3
    P3 --> P3_OUT[输出 table.md]
    P3_OUT --> P3_ERR{有错误?}
    P3_ERR -->|是| EXIT2[打印错误并退出]
    P3_ERR -->|通过| P4["Phase 4: MIPS 代码生成"]
    P4 --> WRITE[写入 .asm 文件]
```

## 3. 词法分析模块 (src/lexer/)

**输入**: SNL 源程序字符串
**输出**: Token 序列

```mermaid
flowchart LR
    SRC[源程序字符串] --> DFA[DFA 状态机] --> TOKENS[Token 序列] --> KW[关键字检查]
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

```mermaid
flowchart TD
    TOKENS["Token 序列"] --> PP["parse_program"]
    PP --> PDP["parse_declare_part"]
    PDP --> PTD["parse_type_dec<br/>(类型声明)"]
    PDP --> PVD["parse_var_dec<br/>(变量声明)"]
    PDP --> PPD["parse_proc_dec<br/>(过程声明, 递归)"]
    PP --> PPB["parse_program_body"]
    PPB --> PSL["parse_stm_list"]
    PSL --> PIF["parse_if_stm"]
    PSL --> PW["parse_while_stm"]
    PSL --> PRW["parse_read / parse_write"]
    PSL --> PRET["parse_return_stm"]
    PSL --> PAC["parse_assign_or_call"]
    PSL --> PE["parse_exp<br/>(递归下降处理优先级)"]
```

**表达式优先级** (由松到紧):
```mermaid
flowchart LR
    RE["RelExp (&lt;, =)"] --> EXP["Exp (+, -)"] --> TERM["Term (*, /)"] --> FAC["Factor (常量/变量/括号)"]
```

**恐慌模式错误恢复**: 遇到语法错误时跳过 Token 直到同步符号 (`;`, `end`, `fi`, `endwh`)

### 4.2 LL(1) 分析器 (ll1.rs)

**输入**: Token 序列
**输出**: 语法错误检查信息

```mermaid
flowchart LR
    G["文法定义<br/>(grammar.rs)"] --> FF["FIRST/FOLLOW 计算<br/>(first_follow.rs)"]
    FF --> PT["分析表构建<br/>(parse_table.rs)"]
    PT --> LL1["表驱动解析<br/>(ll1.rs)"]
```

**文法规模**: ~70 条产生式, 40+ 个非终结符

**左因子化**: `Stm → ID AssignRest` 与 `Stm → ID ( ActParamList )` 共用 `Ident` 前缀，引入 `AssCall` 非终结符消除冲突。

## 5. 语义分析模块 (src/semantic/)

**输入**: 抽象语法树 (AST)
**输出**: 语义错误信息 + 符号表

**两遍遍历**:

```mermaid
flowchart TD
    AST["AST"] --> PASS1["第一遍: 符号表构建"]
    PASS1 --> GLOBAL["全局作用域: 类型名、全局变量、过程名"]
    PASS1 --> ENTER["进入过程体: 压入新作用域 (嵌套层级 +1)"]
    PASS1 --> EXIT["退出过程体: 弹出作用域 (快照保存)"]
    AST --> PASS2["第二遍: 语义检查"]
    PASS2 --> E1["1. 标识符重复定义"]
    PASS2 --> E2["2. 无声明的标识符"]
    PASS2 --> E3["3. 标识符类别错误"]
    PASS2 --> E4["4. 数组下标越界"]
    PASS2 --> E5["5. 数组成员/域变量引用不合法"]
    PASS2 --> E6["6. 赋值类型不兼容"]
    PASS2 --> E7["7. 赋值左端非变量"]
    PASS2 --> E8["8. 形实参类型不匹配"]
    PASS2 --> E9["9. 形实参个数不相同"]
    PASS2 --> E10["10. 非过程标识符调用"]
    PASS2 --> E11["11. 条件表达式非整型"]
    PASS2 --> E12["12. 运算符分量类型不兼容"]
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

```mermaid
flowchart TD
    PROG["Program AST"]
    PROG --> DATA[".data 段"]
    DATA --> DATA_GV["全局变量: var_X: .word 0 / .space N"]
    DATA --> DATA_NL["换行串: newline: .asciiz '\n'"]
    PROG --> MAIN["main 过程"]
    MAIN --> MAIN_P["序言: 保存 $ra, 设置 $fp, 分配局部变量"]
    MAIN --> MAIN_B["编译语句体"]
    MAIN --> MAIN_E["尾声: exit syscall (v0=10)"]
    PROG --> PROC["所有 procedure"]
    PROC --> PROC_P["序言: 保存 $fp + $ra, 设置 $fp, 分配局部变量"]
    PROC --> PROC_A["合并局部类型别名 (局部覆盖外层)"]
    PROC --> PROC_PARAM["分配参数 (调用者栈帧, $fp 上方)"]
    PROC --> PROC_B["编译语句体"]
    PROC --> PROC_R["递归编译嵌套过程"]
    PROC --> PROC_E["尾声: 释放局部变量, 恢复 $fp/$ra, jr $ra"]
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

```mermaid
flowchart TD
    subgraph BinOp["二元运算"]
        R["编译右操作数 → $v0"] --> PUSH["push $v0 到栈"]
        PUSH --> L["编译左操作数 → $v0"]
        L --> POP["pop 到 $t0"]
        POP --> EXEC["执行运算: $v0 = $v0 op $t0"]
    end
    subgraph VarAccess["变量访问 (含选择器)"]
        SIMPLE["简单变量"] --> LOAD["emit_load (直接 lw/lb)"]
        COMPLEX["有下标/字段"] --> EVA["emit_var_address → $t0"]
        EVA --> LOAD2["lw/lb $v0, 0($t0)"]
        EVA --> AS["ArraySubscript: 下标→$v0, 减低下界, 乘元素大小, addu $t0"]
        EVA --> FLD["Field: 加字段偏移到 $t0"]
        EVA --> FS["FieldSubscript: 加字段偏移 + 数组下标计算"]
    end
```

结果约定: 表达式结果始终在 $v0 中

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

```mermaid
sequenceDiagram
    participant Caller as 调用者
    participant Callee as 被调用者

    Caller->>Caller: 参数逆序压栈 (addiu + sw)
    Caller->>Callee: jal proc_name
    Callee->>Callee: addiu $sp, $sp, -8
    Callee->>Callee: sw $fp, 0($sp)
    Callee->>Callee: sw $ra, 4($sp)
    Callee->>Callee: move $fp, $sp
    Callee->>Callee: addiu $sp, $sp, -N (分配局部变量)
    Note over Callee: ...执行过程体...
    Callee->>Callee: addiu $sp, $sp, N (释放局部变量)
    Callee->>Callee: lw $fp, 0($sp)
    Callee->>Callee: lw $ra, 4($sp)
    Callee->>Callee: addiu $sp, $sp, 8
    Callee->>Caller: jr $ra (返回)
    Caller->>Caller: addiu $sp, $sp, 4×N (弹出参数)
```

## 7. 数据流总览 (以 factorial 为例)

```mermaid
flowchart TD
    INPUT['''输入: "Program factorial var integer result; ..."''']
    INPUT --> LEX["词法分析"]
    LEX --> LEX_OUT["program→TK::Program<br/>factorial→Ident<br/>var→TK::Var<br/>integer→TK::Integer<br/>..."]
    LEX --> TOKEN_MD["→ token.md"]
    LEX_OUT --> PARSE["语法分析 (AST)"]
    PARSE --> AST_OUT["Program { name: factorial<br/>  decl: DeclarePart { vars: [result, n], procs: [fact] }<br/>  body: StmList [Assign, Call, Write] }"]
    PARSE --> TREE_MD["→ tree.md"]
    AST_OUT --> SEM["语义分析"]
    SEM --> SEM_OUT["✓ 符号表: result(global), n(global), fact(proc, param=m), m(local), temp(local)<br/>✓ 类型检查通过"]
    SEM --> TABLE_MD["→ table.md"]
    SEM_OUT --> CG["代码生成 (.asm)"]
    CG --> ASM_OUT[".data / var_result: .word 0 / var_n: .word 0<br/>.text / main: ... jal proc_fact ...<br/>proc_fact: addiu $sp, $sp, -8 ..."]
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
