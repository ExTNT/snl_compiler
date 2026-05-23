# `samples/rec_test.snl` 语法树

## 抽象语法树

```
ProK
├── PheadK  rectest
│   ├── VarK
│   │   ├── DecK  RecordK  IntegerK  id  CharK  tag  r
│   ├── StmLk
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  r  IdV.id
│   │   │   └── ExpK  Const  100
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  r  IdV.tag
│   │   │   └── ExpK  Const  'X'
│   │   ├── StmtK  Write
│   │   │   └── ExpK  r  IdV.id
│   │   ├── StmtK  Write
│   │   │   └── ExpK  r  IdV.tag
└── .
```

## 语法错误

无。
