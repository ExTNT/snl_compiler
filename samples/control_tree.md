# `samples/control.snl` 语法树

## 抽象语法树

```
ProK
├── PheadK  control
│   ├── VarK
│   │   ├── DecK  IntegerK  n
│   ├── StmLk
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  n  IdV
│   │   │   └── ExpK  Const  10
│   │   ├── StmtK  If
│   │   │   ├── ExpK  Op  <
│   │   │   ├── StmLk
│   │   │   │   ├── StmtK  Assign
│   │   │   │   │   ├── ExpK  n  IdV
│   │   │   │   │   └── ExpK  Op  +
│   │   │   ├── StmLk
│   │   │   │   ├── StmtK  Assign
│   │   │   │   │   ├── ExpK  n  IdV
│   │   │   │   │   └── ExpK  Op  -
│   │   ├── StmtK  While
│   │   │   ├── ExpK  Const  1
│   │   │   ├── StmLk
│   │   │   │   ├── StmtK  Write
│   │   │   │   │   └── ExpK  n  IdV
└── .
```

## 语法错误

无。
