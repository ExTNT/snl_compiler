# `samples/gcd.snl` 语法树

## 抽象语法树

```
ProK
├── PheadK  gcd
│   ├── VarK
│   │   ├── DecK  IntegerK  a
│   │   ├── DecK  IntegerK  b
│   │   ├── DecK  IntegerK  done
│   ├── StmLk
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  a  IdV
│   │   │   └── ExpK  Const  48
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  b  IdV
│   │   │   └── ExpK  Const  18
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  done  IdV
│   │   │   └── ExpK  Const  0
│   │   ├── StmtK  While
│   │   │   ├── ExpK  Op  <
│   │   │   ├── StmLk
│   │   │   │   ├── StmtK  If
│   │   │   │   │   ├── ExpK  Op  <
│   │   │   │   │   ├── StmLk
│   │   │   │   │   │   ├── StmtK  Assign
│   │   │   │   │   │   │   ├── ExpK  b  IdV
│   │   │   │   │   │   │   └── ExpK  Op  -
│   │   │   │   │   ├── StmLk
│   │   │   │   │   │   ├── StmtK  If
│   │   │   │   │   │   │   ├── ExpK  Op  <
│   │   │   │   │   │   │   ├── StmLk
│   │   │   │   │   │   │   │   ├── StmtK  Assign
│   │   │   │   │   │   │   │   │   ├── ExpK  a  IdV
│   │   │   │   │   │   │   │   │   └── ExpK  Op  -
│   │   │   │   │   │   │   ├── StmLk
│   │   │   │   │   │   │   │   ├── StmtK  Assign
│   │   │   │   │   │   │   │   │   ├── ExpK  done  IdV
│   │   │   │   │   │   │   │   │   └── ExpK  Const  1
│   │   ├── StmtK  Write
│   │   │   └── ExpK  a  IdV
└── .
```

## 语法错误

无。
