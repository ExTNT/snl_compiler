# SNL 编译器

## 项目简介

SNL（Small Nested Language）编译器基于 Rust 实现，将 SNL 源程序编译为 MIPS 汇编代码，可在 SPIM/MARS 模拟器上运行。涵盖编译原理课程的完整流程：词法分析 → 语法分析 → 语义分析 → 目标代码生成。

### SNL 语言特性

- **数据类型**：`integer`（整型）、`char`（字符型）、`array[lo..hi] of T`（数组）、`record ... end`（记录）
- **类型别名**：支持 `type Name = Type` 定义命名类型
- **控制结构**：`if/then/else/fi`、`while/do/endwh`
- **比较运算**：`<`（小于）、`=`（等于）
- **算术运算**：`+`、`-`、`*`、`/`
- **输入输出**：`read(x)` 读取、`write(x)` 输出
- **过程**：支持嵌套定义和递归调用，值参数

### SNL 示例程序

```pascal
program factorial
    var integer result;
        integer n;
    procedure fact(integer m);
        var integer temp;
    begin
        if m < 2
        then return(1)
        else
            temp := m - 1;
            fact(temp);
            return(m * result)
        fi
    end
begin
    n := 5;
    fact(n);
    write(result)
end.
```

---

## 编译流程

```
SNL源程序 (.snl)
    │
    ▼
┌─────────────────┐
│  词法分析       │  →  Token 序列     →  *_token.md
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  语法分析       │  →  抽象语法树     →  *_tree.md
│ 递归下降 + LL(1)│
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  语义分析       │  →  符号表         →  *_table.md
│  12种错误检查   │
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  目标代码生成   │  →  MIPS 汇编      →  *.asm
│  全类型支持     │
└─────────────────┘
```

---

## 项目结构

```
snl_compiler/
├── Cargo.toml
├── README.md
├── PROGRAM_FLOW.md          # 程序流程详细说明
├── samples/                 # 17 个 SNL 示例程序
│   ├── hello.snl            #   简单输出
│   ├── arithmetic.snl       #   算术运算
│   ├── control.snl          #   条件与循环
│   ├── factorial.snl        #   递归阶乘
│   ├── fib.snl              #   斐波那契数列
│   ├── gcd.snl              #   最大公约数
│   ├── prime.snl            #   素数判定
│   ├── power.snl            #   幂运算
│   ├── sum.snl              #   等差数列求和
│   ├── char_test.snl        #   字符类型测试
│   ├── int_array.snl        #   整型数组测试
│   ├── char_array.snl       #   字符数组测试
│   ├── rec_test.snl         #   记录测试
│   ├── rec_array.snl        #   记录含数组字段测试
│   ├── bubble.snl           #   冒泡排序
│   ├── selection.snl        #   选择排序
│   ├── insertion.snl        #   插入排序
│   └── expected_output.md   #   所有样例预期输出
└── src/
    ├── main.rs              # 程序入口，命令行接口
    ├── lib.rs               # 模块导出
    ├── error.rs             # 统一错误类型定义
    ├── lexer/               # 词法分析
    │   ├── token.rs         #   Token 种类与结构
    │   ├── keyword.rs       #   关键字查找表
    │   ├── dfa.rs           #   DFA 状态机
    │   └── mod.rs           #   词法分析器驱动
    ├── ast/                 # 抽象语法树
    │   ├── nodes.rs         #   AST 节点类型定义
    │   ├── display.rs       #   层次化输出
    │   └── mod.rs
    ├── parser/              # 语法分析
    │   ├── rd.rs            #   递归下降分析器
    │   ├── grammar.rs       #   文法产生式定义
    │   ├── first_follow.rs  #   FIRST/FOLLOW 集计算
    │   ├── parse_table.rs   #   LL(1) 分析表构建
    │   ├── ll1.rs           #   表驱动 LL(1) 分析器
    │   └── mod.rs
    ├── semantic/            # 语义分析
    │   ├── symbol.rs        #   符号表与作用域栈
    │   ├── analyzer.rs      #   两遍遍历分析器
    │   └── mod.rs
    └── codegen/             # 目标代码生成
        ├── mips.rs          #   MIPS 汇编生成（全类型支持）
        └── mod.rs
```

---

## 各模块功能

### 词法分析 (src/lexer/)

- **实现**: 基于 DFA 的扫描器，最长匹配策略
- **识别**: 21 个关键字、标识符、整型/字符常量、单/双字符分界符、注释
- **测试**: 28 个用例

### 语法分析 (src/parser/)

- **递归下降** (`rd.rs`): 为每个非终结符编写解析函数，恐慌模式错误恢复
- **LL(1) 表驱动** (`ll1.rs`): FIRST/FOLLOW 不动点迭代，左因子化消除冲突
- **测试**: 35 个用例

### 语义分析 (src/semantic/)

- **两遍遍历**: 第一遍构建符号表（嵌套作用域栈），第二遍类型检查
- **12 种语义错误**: 重复定义、未声明、类别错误、下标越界、类型不兼容等
- **测试**: 22 个用例

### 代码生成 (src/codegen/)

- **全类型支持**: integer、char、array、record 及类型别名
- **类型感知存取**: `lb`/`sb` (char) vs `lw`/`sw` (integer)
- **类型感知 I/O**: syscall 12/11 (char) vs 5/1 (integer)
- **选择器地址计算**: `emit_var_address` 处理数组下标、记录字段及嵌套组合
- **测试**: 37 个用例

---

## 编译与运行

### 环境要求

- Rust 工具链 (1.85+)
- MIPS 模拟器 (可选): [SPIM](http://spimsimulator.sourceforge.net/) 或 [MARS](http://courses.missouristate.edu/KenVollmar/MARS/)

### 构建

```bash
cargo build              # debug 模式
cargo build --release    # release 模式
cargo test               # 全部 122 个测试
```

### 编译 SNL 程序

```bash
# 编译 hello.snl → hello.asm (同时生成 _token.md, _tree.md, _table.md)
cargo run -- samples/hello.snl

# 指定输出文件名
cargo run -- samples/factorial.snl -o output.asm
```

### 运行生成的 MIPS 代码

```bash
spim -file samples/hello.asm
# 输出: 42
```

### 分模块测试

```bash
cargo test lexer       # 词法分析 (28 个用例)
cargo test parser      # 语法分析 (35 个用例)
cargo test semantic    # 语义分析 (22 个用例)
cargo test codegen     # 代码生成 (37 个用例)
```

---

## 样例程序

### 基础样例

| 程序 | 输出 | 说明 |
|------|------|------|
| `hello.snl` | `42` | 简单输出 |
| `arithmetic.snl` | `10` `15` | 算术运算 |
| `control.snl` | `11`... | 条件与循环 (死循环) |

### 算法样例

| 程序 | 输出 | 说明 |
|------|------|------|
| `fib.snl` | `55` | 斐波那契数列 |
| `gcd.snl` | `6` | 最大公约数 |
| `prime.snl` | `1` | 素数判定 |
| `power.snl` | `256` | 幂运算 (2^8) |
| `sum.snl` | `55` | 等差数列求和 |
| `factorial.snl` | `120` | 递归阶乘 (5!) |

### 类型测试

| 程序 | 输出 | 说明 |
|------|------|------|
| `char_test.snl` | `A` `B` | char 变量赋值/输出 |
| `int_array.snl` | `2` `6` `10` | 整型数组下标访问 |
| `char_array.snl` | `H` `e` `l` `l` `o` | 字符数组 |
| `rec_test.snl` | `100` `X` | 记录 (int + char) |
| `rec_array.snl` | `0` `20` `40` `5` | 记录含数组字段 |

### 排序算法

| 程序 | 输出 | 说明 |
|------|------|------|
| `bubble.snl` | `12` `22` `25` `34` `64` | 冒泡排序 |
| `selection.snl` | `12` `22` `25` `34` `64` | 选择排序 |
| `insertion.snl` | `12` `22` `25` `34` `64` | 插入排序 |

---

## 错误处理

编译器收集尽可能多的错误后统一报告:

```
=== Syntax Errors ===
  Line 5:10 — Expected ;, found Ident("v2")
  Line 8:5 — Unexpected token End at start of statement

=== Semantic Errors ===
  Line 10:5 — Undeclared identifier 'z'
  Line 12:5 — Assignment type mismatch
```
