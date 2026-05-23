# `samples/arithmetic.snl` 语法树

## 抽象语法树

```
ProK
├── PheadK  arithmetic
│   ├── VarK
│   │   ├── DecK  IntegerK  x
│   │   ├── DecK  IntegerK  y
│   ├── StmLk
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  x  IdV
│   │   │   └── ExpK  Const  10
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  y  IdV
│   │   │   └── ExpK  Op  +
│   │   ├── StmtK  Write
│   │   │   └── ExpK  x  IdV
│   │   ├── StmtK  Write
│   │   │   └── ExpK  y  IdV
└── .
```

## 语法错误

无。
