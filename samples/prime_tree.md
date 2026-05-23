# `samples/prime.snl` 语法树

## 抽象语法树

```
ProK
├── PheadK  prime
│   ├── VarK
│   │   ├── DecK  IntegerK  n
│   │   ├── DecK  IntegerK  d
│   │   ├── DecK  IntegerK  primeFlag
│   │   ├── DecK  IntegerK  remainder
│   ├── StmLk
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  n  IdV
│   │   │   └── ExpK  Const  17
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  d  IdV
│   │   │   └── ExpK  Const  2
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  primeFlag  IdV
│   │   │   └── ExpK  Const  1
│   │   ├── StmtK  While
│   │   │   ├── ExpK  Op  <
│   │   │   ├── StmLk
│   │   │   │   ├── StmtK  Assign
│   │   │   │   │   ├── ExpK  remainder  IdV
│   │   │   │   │   └── ExpK  n  IdV
│   │   │   │   ├── StmtK  While
│   │   │   │   │   ├── ExpK  Op  <
│   │   │   │   │   ├── StmLk
│   │   │   │   │   │   ├── StmtK  Assign
│   │   │   │   │   │   │   ├── ExpK  remainder  IdV
│   │   │   │   │   │   │   └── ExpK  Op  -
│   │   │   │   ├── StmtK  If
│   │   │   │   │   ├── ExpK  Op  <
│   │   │   │   │   ├── StmLk
│   │   │   │   │   │   ├── StmtK  Assign
│   │   │   │   │   │   │   ├── ExpK  d  IdV
│   │   │   │   │   │   │   └── ExpK  Op  +
│   │   │   │   │   ├── StmLk
│   │   │   │   │   │   ├── StmtK  Assign
│   │   │   │   │   │   │   ├── ExpK  primeFlag  IdV
│   │   │   │   │   │   │   └── ExpK  Const  0
│   │   │   │   │   │   ├── StmtK  Assign
│   │   │   │   │   │   │   ├── ExpK  d  IdV
│   │   │   │   │   │   │   └── ExpK  n  IdV
│   │   ├── StmtK  Write
│   │   │   └── ExpK  primeFlag  IdV
└── .
```

## 语法错误

无。
