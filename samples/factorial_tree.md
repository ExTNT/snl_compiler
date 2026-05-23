# `samples/factorial.snl` 语法树

## 抽象语法树

```
ProK
├── PheadK  factorial
│   ├── VarK
│   │   ├── DecK  IntegerK  result
│   │   ├── DecK  IntegerK  n
│   ├── ProcDecK  fact
│   │   ├── DecK  value param:  IntegerK  m
│   │   ├── StmLk
│   │   │   ├── StmtK  If
│   │   │   │   ├── ExpK  Op  <
│   │   │   │   ├── StmLk
│   │   │   │   │   ├── StmtK  Assign
│   │   │   │   │   │   ├── ExpK  result  IdV
│   │   │   │   │   │   └── ExpK  Const  1
│   │   │   │   ├── StmLk
│   │   │   │   │   ├── StmtK  Call
│   │   │   │   │   │   ├── ExpK  Op  -
│   │   │   │   │   ├── StmtK  Assign
│   │   │   │   │   │   ├── ExpK  result  IdV
│   │   │   │   │   │   └── ExpK  Op  *
│   ├── StmLk
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  n  IdV
│   │   │   └── ExpK  Const  5
│   │   ├── StmtK  Call
│   │   │   ├── ExpK  n  IdV
│   │   ├── StmtK  Write
│   │   │   └── ExpK  result  IdV
└── .
```

## 语法错误

无。
