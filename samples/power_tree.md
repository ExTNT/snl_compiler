# `samples/power.snl` 语法树

## 抽象语法树

```
ProK
├── PheadK  power
│   ├── VarK
│   │   ├── DecK  IntegerK  base
│   │   ├── DecK  IntegerK  exp
│   │   ├── DecK  IntegerK  result
│   │   ├── DecK  IntegerK  i
│   ├── StmLk
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  base  IdV
│   │   │   └── ExpK  Const  2
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  exp  IdV
│   │   │   └── ExpK  Const  8
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  result  IdV
│   │   │   └── ExpK  Const  1
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  i  IdV
│   │   │   └── ExpK  Const  0
│   │   ├── StmtK  While
│   │   │   ├── ExpK  Op  <
│   │   │   ├── StmLk
│   │   │   │   ├── StmtK  Assign
│   │   │   │   │   ├── ExpK  result  IdV
│   │   │   │   │   └── ExpK  Op  *
│   │   │   │   ├── StmtK  Assign
│   │   │   │   │   ├── ExpK  i  IdV
│   │   │   │   │   └── ExpK  Op  +
│   │   ├── StmtK  Write
│   │   │   └── ExpK  result  IdV
└── .
```

## 语法错误

无。
