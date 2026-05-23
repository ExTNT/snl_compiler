# 编译原理课程设计

## 一、实验内容

设计并实现 **SNL（Small Nested Language）** 程序设计语言的编译程序。

### 四个必做模块

| 模块 | 输入 | 输出 |
|:---|:---|:---|
| **词法分析模块** | SNL源程序 | Token序列 |
| **语法分析模块（递归下降方法）** | Token序列 | 语法树 + 语法错误检查信息 |
| **语法分析模块（LL(1)方法）** | Token序列 | 语法树 + 语法错误检查信息 |
| **语义分析模块** | 语法树或Token序列 | 语义错误信息 |

### 编译流程

```
SNL源程序 → [词法分析模块] → Token序列 → [语法分析模块(递归下降法 / LL(1)法)] → 语法树 → [语义分析模块] → 语义错误信息
```

## 二、实验目的

1. 通过对SNL编译程序的学习和动手实践，使学生可以更加深入、全面地掌握**编译程序的工作原理和实现技术**
2. 培养**大型软件的程序设计方法**

## 三、SNL语言简介

SNL（Small Nested Language）是自行定义的**教学模型语言**，它是一种类似 Pascal 的"高级"程序设计语言。

### 语言特性

- **数据类型**：整型、字符型、数组、记录
- **过程允许嵌套定义**
- **允许递归调用**

### SNL的程序结构

```
程序头
声明部分
    ├── 类型声明部分
    ├── 变量声明部分
    └── 过程声明部分
程序体部分
```

### 示例程序

```pascal
program pp
var   integer  v1;
      char  c;
procedure f();
begin
    v1 := 2
End
Begin
    f();
    write(v1)
end.
```

## 四、SNL语言的单词分类

| 类别 | 说明 | 示例 |
|:---|:---|:---|
| **标识符（ID）** | 字母开头的字母数字串 | `v1`, `f`, `pp` |
| **保留字** | 标识符的子集 | `if`, `repeat`, `read`, `write`, `program`, `var`, `procedure`, `begin`, `end`, `integer`, `char`, `array`, `record`, `type`, `while`, `do`, `if`, `then`, `else`, `fi`, `return`, `read`, `write` 等 |
| **无符号整数（INTC）** | 数字序列 | `2`, `10` |
| **单字符分界符** | 单个字符 | `+`, `-`, `*`, `/`, `<`, `=`, `(`, `)`, `[`, `]`, `.`, `;`, `EOF`, 空白字符 |
| **双字符分界符** | 两个字符 | `:=` |
| **注释头符** | 注释开始 | `{` |
| **注释结束符** | 注释结束 | `}` |
| **字符起始和结束符** | 单引号 | `'` |
| **数组下标界限符** | 两个点 | `..` |

### 符号的巴科斯范式（BNF）定义

```
<<标识符>         ::= 字母 { 字母 | 数字 }
<<无符号整数>     ::= 数字 { 数字 }
<<单字符分界符>   ::= + | - | * | / | ( | ) | [ | ] | ; | . | < | = | EOF | 空白字符
<<双字符分界符>   ::= :=
<<注释头符号>     ::= {
<<注释结束符号>   ::= }
<<字符标示符>     ::= '
<<数组下标界限符> ::= ..
<<字母>           ::= a | b | ... | z | A | B | ... | Z
<<数字>           ::= 0 | 1 | 2 | ... | 9
```

## 五、SNL语言的上下文无关文法

### 5.1 程序结构

```
1)  Program         ::= ProgramHead DeclarePart ProgramBody
2)  ProgramHead     ::= PROGRAM ProgramName
3)  ProgramName     ::= ID
4)  DeclarePart     ::= TypeDec VarDec ProcDec
```

### 5.2 类型声明

```
6)  TypeDec         ::= ε | TypeDeclaration
7)  TypeDeclaration ::= TYPE TypeDecList
8)  TypeDecList     ::= TypeId = TypeName ; TypeDecMore
9)  TypeDecMore     ::= ε | TypeDecList
10) TypeId          ::= ID
```

### 5.3 类型定义

```
12) TypeName        ::= BaseType | StructureType | ID
15) BaseType        ::= INTEGER | CHAR
17) StructureType   ::= ArrayType | RecType
19) ArrayType       ::= ARRAY [ low .. top ] OF BaseType
20) Low             ::= INTC
21) Top             ::= INTC
22) RecType         ::= RECORD FieldDecList END
23) FieldDecList    ::= BaseType IdList ; FieldDecMore
24)                 | ArrayType IdList ; FieldDecMore
25) FieldDecMore    ::= ε | FieldDecList
```

### 5.4 标识符列表

```
27) IdList          ::= ID IdMore
28) IdMore          ::= ε | , IdList
```

### 5.5 变量声明

```
30) VarDec          ::= ε | VarDeclaration
32) VarDeclaration  ::= VAR VarDecList
33) VarDecList      ::= TypeName VarIdList ; VarDecMore
34) VarDecMore      ::= ε | VarDecList
36) VarIdList       ::= ID VarIdMore
37) VarIdMore       ::= ε | , VarIdList
```

### 5.6 过程声明

```
39) ProcDec         ::= ε | ProcDeclaration
41) ProcDeclaration ::= PROCEDURE ProcName ( ParamList ) ; ProcDecPart ProcBody ProcDecMore
42) ProcDecMore     ::= ε | ProcDeclaration
44) ProcName        ::= ID
```

### 5.7 参数列表

```
45) ParamList       ::= ε | ParamDecList
47) ParamDecList    ::= Param ParamMore
48) ParamMore       ::= ε | ; ParamDecList
50) Param           ::= TypeName FormList | VAR TypeName FormList
52) FormList        ::= ID FidMore
53) FidMore         ::= ε | , FormList
```

### 5.8 过程体与程序体

```
55) ProcDecPart     ::= DeclarePart
56) ProcBody        ::= ProgramBody
57) ProgramBody     ::= BEGIN StmList END
```

### 5.9 语句列表

```
58) StmList         ::= Stm StmMore
59) StmMore         ::= ε | ; StmList
```

### 5.10 语句

```
61) Stm             ::= ConditionalStm | LoopStm | InputStm | OutputStm | ReturnStm | ID AssCall
67) AssCall         ::= AssignmentRest | CallStmRest
69) AssignmentRest  ::= VariMore := Exp
70) ConditionalStm  ::= IF RelExp THEN StmList ELSE StmList FI
71) LoopStm         ::= WHILE RelExp DO StmList ENDWH
72) InputStm        ::= READ ( Invar )
73) Invar           ::= ID
74) OutputStm       ::= WRITE ( Exp )
75) ReturnStm       ::= RETURN ( Exp )
76) CallStmRest     ::= ( ActParamList )
```

### 5.11 实参列表

```
77) ActParamList    ::= ε | Exp ActParamMore
79) ActParamMore    ::= ε | , ActParamList
```

### 5.12 表达式

```
81) RelExp          ::= Exp OtherRelE
82) OtherRelE       ::= CmpOp Exp
83) Exp             ::= Term OtherTerm
84) OtherTerm       ::= ε | AddOp Exp
86) Term            ::= Factor OtherFactor
87) OtherFactor     ::= ε | MultOp Term
89) Factor          ::= ( Exp ) | INTC | Variable
92) Variable        ::= ID VariMore
```

### 5.13 变量后缀

```
93) VariMore        ::= ε | [ Exp ] | . FieldVar
96) FieldVar        ::= ID FieldVarMore
97) FieldVarMore    ::= ε | [ Exp ]
```

### 5.14 运算符

```
99)  CmpOp          ::= < | =
101) AddOp          ::= + | -
103) MultOp         ::= * | /
```

## 六、第一个程序：词法分析程序

### 功能
- **输入**：SNL源程序
- **输出**：单词的内部表示序列（Token序列）

### 程序设计步骤

1. **确定单词分类**（见第4节）
2. **单词的正则表达式定义**（词法定义）
3. **构造DFA**（确定有限自动机）
4. **根据DFA生成单词识别函数**
   - 对于给定的当前状态和当前字符，决定下一个状态
   - 如果是结束状态，得到单词的类别和信息
   - 否则，设置为当前状态

### 涉及问题

- 单词分类
- Token表示定义
- 每类单词的构成
- 自动机的实现（状态图方法、转换表方法）
- 特殊情形处理：注释、向前扫描等

## 七、第二个程序：递归下降分析程序

### 功能
- **输入**：Token序列
- **输出**：语法错误检查信息和语法树

### 核心思想

对文法的每一个非终结符（VN）都编写一个分析程序。当根据文法和当前输入符号预测到要用某个VN去匹配输入串时，就调用该VN的分析程序。

### 示例

对于文法：
```
S → A u B
A → a a
B → b b
```

对应的递归子程序结构：
```c
S() {
    A();
    match(u);
    B();
}

A() {
    match(a);
    match(a);
}

B() {
    match(b);
    match(b);
}
```

### 涉及问题

- 当产生式形如 `A → β₁ | β₂ | ... | βₙ` 时，需要通过 **predict集** 确定子程序的调用
- 如何构建语法树及语法树的内部表示

### 语法树输出格式

#### 层次文本输出示例

```
ProK
├── PheadK  p
├── TypeK
│   └── DecK  IntegerK  t1
├── VarK
│   └── DecK  IntegerK  v1  v2
├── ProcDecK  q
│   ├── DecK  value param:  IntegerK  i
│   ├── VarK
│   │   └── DecK  IntegerK  a
│   └── StmLk
│       ├── StmtK  Assign
│       │   ├── ExpK  a  IdV
│       │   └── ExpK  i  IdV
│       └── StmtK  Write
│           └── ExpK  a  IdV
└── StmLk
    ├── StmtK  Read  v1
    ├── StmtK  If
    │   ├── ExpK  Op  <
    │   │   ├── ExpK  v1  IdV
    │   │   └── ExpK  Const  10
    │   ├── StmtK  Assign
    │   │   ├── ExpK  v1  IdV
    │   │   └── ExpK  Op  +
    │   │       ├── ExpK  v1  IdV
    │   │       └── ExpK  Const  10
    │   └── StmtK  Assign
    │       ├── ExpK  v1  IdV
    │       └── ExpK  Op  -
    │           ├── ExpK  v1  IdV
    │           └── ExpK  Const  10
    └── StmtK  Call
        ├── ExpK  q  IdV
        └── ExpK  v1  IdV
```

#### 图形输出

支持语法树的图形化展示。

## 八、第三个程序：LL(1)分析程序

### 功能
- **输入**：Token序列
- **输出**：语法错误检查信息和语法树

### 文法入口

```
Program → ProgramHead DeclarePart ProgramBody .
```

## 九、第四个程序：语义分析程序

### 功能
- **输入**：语法树或Token序列
- **输出**：语义错误信息

### 数据结构

| 结构 | 用途 |
|:---|:---|
| **语法树** | 遍历分析程序结构 |
| **符号表** | 存储标识符属性信息 |

### 算法

1. **建立符号表**：在扫描**声明部分**语法树时，构造符号表项
2. **查表检查语义**：在扫描**语句部分**语法树时，查找符号表项进行语义检查

### SNL的语义错误类型

| 编号 | 错误描述 |
|:---|:---|
| 1 | 标识符的重复定义 |
| 2 | 无声明的标识符 |
| 3 | 标识符为非期望的标识符类别（类型标识符、变量标识符、过程名标识符） |
| 4 | 数组类型下标越界错误 |
| 5 | 数组成员变量和域变量的引用不合法 |
| 6 | 赋值语句的左右两边类型不相容 |
| 7 | 赋值语句左端不是变量标识符 |
| 8 | 过程调用中，形实参类型不匹配 |
| 9 | 过程调用中，形实参个数不相同 |
| 10 | 过程调用语句中，标识符不是过程标识符 |
| 11 | `if` 和 `while` 语句的条件部分不是 bool 类型 |
| 12 | 表达式中运算符的分量的类型不相容 |

### 符号表设计

#### 符号表项属性

- **种类**（类型标识符 / 变量标识符 / 过程名标识符）
- **类型**（整型 / 字符型 / 数组 / 记录）
- **名字**
- **参数个数**（对于过程）
- 其他相关属性

#### 符号表的组织方式

- 顺序表
- 二叉树
- 散列表

#### 特殊考虑

> **符号表的局部化问题**：由于SNL允许过程嵌套定义，需要处理作用域和嵌套层次的符号表管理。
