# `samples/fib.snl` 语法树

## 抽象语法树

```
ProK
├── PheadK  fib
│   ├── VarK
│   │   ├── DecK  IntegerK  n
│   │   ├── DecK  IntegerK  a
│   │   ├── DecK  IntegerK  b
│   │   ├── DecK  IntegerK  i
│   │   ├── DecK  IntegerK  temp
│   ├── StmLk
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  n  IdV
│   │   │   └── ExpK  Const  10
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  a  IdV
│   │   │   └── ExpK  Const  0
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  b  IdV
│   │   │   └── ExpK  Const  1
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  i  IdV
│   │   │   └── ExpK  Const  0
│   │   ├── StmtK  While
│   │   │   ├── ExpK  Op  <
│   │   │   ├── StmLk
│   │   │   │   ├── StmtK  Assign
│   │   │   │   │   ├── ExpK  temp  IdV
│   │   │   │   │   └── ExpK  Op  +
│   │   │   │   ├── StmtK  Assign
│   │   │   │   │   ├── ExpK  a  IdV
│   │   │   │   │   └── ExpK  b  IdV
│   │   │   │   ├── StmtK  Assign
│   │   │   │   │   ├── ExpK  b  IdV
│   │   │   │   │   └── ExpK  temp  IdV
│   │   │   │   ├── StmtK  Assign
│   │   │   │   │   ├── ExpK  i  IdV
│   │   │   │   │   └── ExpK  Op  +
│   │   ├── StmtK  Write
│   │   │   └── ExpK  a  IdV
└── .
```

## 语法错误

无。
