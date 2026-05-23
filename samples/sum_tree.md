# `samples/sum.snl` 语法树

## 抽象语法树

```
ProK
├── PheadK  sum
│   ├── VarK
│   │   ├── DecK  IntegerK  n
│   │   ├── DecK  IntegerK  total
│   │   ├── DecK  IntegerK  i
│   ├── StmLk
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  n  IdV
│   │   │   └── ExpK  Const  10
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  total  IdV
│   │   │   └── ExpK  Const  0
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  i  IdV
│   │   │   └── ExpK  Const  1
│   │   ├── StmtK  While
│   │   │   ├── ExpK  Op  <
│   │   │   ├── StmLk
│   │   │   │   ├── StmtK  Assign
│   │   │   │   │   ├── ExpK  total  IdV
│   │   │   │   │   └── ExpK  Op  +
│   │   │   │   ├── StmtK  Assign
│   │   │   │   │   ├── ExpK  i  IdV
│   │   │   │   │   └── ExpK  Op  +
│   │   ├── StmtK  Write
│   │   │   └── ExpK  total  IdV
└── .
```

## 语法错误

无。
