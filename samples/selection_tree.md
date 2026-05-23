# `samples/selection.snl` 语法树

## 抽象语法树

```
ProK
├── PheadK  selection
│   ├── VarK
│   │   ├── DecK  ArrayK  [0,4]  IntegerK  a
│   │   ├── DecK  IntegerK  n
│   │   ├── DecK  IntegerK  i
│   │   ├── DecK  IntegerK  j
│   │   ├── DecK  IntegerK  minIdx
│   │   ├── DecK  IntegerK  temp
│   ├── StmLk
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  a  IdV[0]
│   │   │   └── ExpK  Const  64
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  a  IdV[1]
│   │   │   └── ExpK  Const  34
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  a  IdV[2]
│   │   │   └── ExpK  Const  25
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  a  IdV[3]
│   │   │   └── ExpK  Const  12
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  a  IdV[4]
│   │   │   └── ExpK  Const  22
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  n  IdV
│   │   │   └── ExpK  Const  5
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  i  IdV
│   │   │   └── ExpK  Const  0
│   │   ├── StmtK  While
│   │   │   ├── ExpK  Op  <
│   │   │   ├── StmLk
│   │   │   │   ├── StmtK  Assign
│   │   │   │   │   ├── ExpK  minIdx  IdV
│   │   │   │   │   └── ExpK  i  IdV
│   │   │   │   ├── StmtK  Assign
│   │   │   │   │   ├── ExpK  j  IdV
│   │   │   │   │   └── ExpK  Op  +
│   │   │   │   ├── StmtK  While
│   │   │   │   │   ├── ExpK  Op  <
│   │   │   │   │   ├── StmLk
│   │   │   │   │   │   ├── StmtK  If
│   │   │   │   │   │   │   ├── ExpK  Op  <
│   │   │   │   │   │   │   ├── StmLk
│   │   │   │   │   │   │   │   ├── StmtK  Assign
│   │   │   │   │   │   │   │   │   ├── ExpK  minIdx  IdV
│   │   │   │   │   │   │   │   │   └── ExpK  j  IdV
│   │   │   │   │   │   │   ├── StmLk
│   │   │   │   │   │   │   │   ├── StmtK  Assign
│   │   │   │   │   │   │   │   │   ├── ExpK  temp  IdV
│   │   │   │   │   │   │   │   │   └── ExpK  Const  0
│   │   │   │   │   │   ├── StmtK  Assign
│   │   │   │   │   │   │   ├── ExpK  j  IdV
│   │   │   │   │   │   │   └── ExpK  Op  +
│   │   │   │   ├── StmtK  Assign
│   │   │   │   │   ├── ExpK  temp  IdV
│   │   │   │   │   └── ExpK  a  IdV[minIdx]
│   │   │   │   ├── StmtK  Assign
│   │   │   │   │   ├── ExpK  a  IdV[minIdx]
│   │   │   │   │   └── ExpK  a  IdV[i]
│   │   │   │   ├── StmtK  Assign
│   │   │   │   │   ├── ExpK  a  IdV[i]
│   │   │   │   │   └── ExpK  temp  IdV
│   │   │   │   ├── StmtK  Assign
│   │   │   │   │   ├── ExpK  i  IdV
│   │   │   │   │   └── ExpK  Op  +
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  i  IdV
│   │   │   └── ExpK  Const  0
│   │   ├── StmtK  While
│   │   │   ├── ExpK  Op  <
│   │   │   ├── StmLk
│   │   │   │   ├── StmtK  Write
│   │   │   │   │   └── ExpK  a  IdV[i]
│   │   │   │   ├── StmtK  Assign
│   │   │   │   │   ├── ExpK  i  IdV
│   │   │   │   │   └── ExpK  Op  +
└── .
```

## 语法错误

无。
