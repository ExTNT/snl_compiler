# `samples/rec_array.snl` 语法树

## 抽象语法树

```
ProK
├── PheadK  recarr
│   ├── VarK
│   │   ├── DecK  RecordK  ArrayK  [0,4]  IntegerK  data  IntegerK  count  r
│   │   ├── DecK  IntegerK  i
│   ├── StmLk
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  r  IdV.count
│   │   │   └── ExpK  Const  5
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  i  IdV
│   │   │   └── ExpK  Const  0
│   │   ├── StmtK  While
│   │   │   ├── ExpK  Op  <
│   │   │   ├── StmLk
│   │   │   │   ├── StmtK  Assign
│   │   │   │   │   ├── ExpK  r  IdV.data[i]
│   │   │   │   │   └── ExpK  Op  *
│   │   │   │   ├── StmtK  Assign
│   │   │   │   │   ├── ExpK  i  IdV
│   │   │   │   │   └── ExpK  Op  +
│   │   ├── StmtK  Write
│   │   │   └── ExpK  r  IdV.data[0]
│   │   ├── StmtK  Write
│   │   │   └── ExpK  r  IdV.data[2]
│   │   ├── StmtK  Write
│   │   │   └── ExpK  r  IdV.data[4]
│   │   ├── StmtK  Write
│   │   │   └── ExpK  r  IdV.count
└── .
```

## 语法错误

无。
