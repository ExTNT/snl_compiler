# `samples/int_array.snl` 语法树

## 抽象语法树

```
ProK
├── PheadK  intarr
│   ├── VarK
│   │   ├── DecK  ArrayK  [1,5]  IntegerK  a
│   │   ├── DecK  IntegerK  i
│   ├── StmLk
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  i  IdV
│   │   │   └── ExpK  Const  1
│   │   ├── StmtK  While
│   │   │   ├── ExpK  Op  <
│   │   │   ├── StmLk
│   │   │   │   ├── StmtK  Assign
│   │   │   │   │   ├── ExpK  a  IdV[i]
│   │   │   │   │   └── ExpK  Op  *
│   │   │   │   ├── StmtK  Assign
│   │   │   │   │   ├── ExpK  i  IdV
│   │   │   │   │   └── ExpK  Op  +
│   │   ├── StmtK  Write
│   │   │   └── ExpK  a  IdV[1]
│   │   ├── StmtK  Write
│   │   │   └── ExpK  a  IdV[3]
│   │   ├── StmtK  Write
│   │   │   └── ExpK  a  IdV[5]
└── .
```

## 语法错误

无。
