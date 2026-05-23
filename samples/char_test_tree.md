# `samples/char_test.snl` 语法树

## 抽象语法树

```
ProK
├── PheadK  chartest
│   ├── VarK
│   │   ├── DecK  CharK  c
│   │   ├── DecK  CharK  d
│   ├── StmLk
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  c  IdV
│   │   │   └── ExpK  Const  'A'
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  d  IdV
│   │   │   └── ExpK  Const  'B'
│   │   ├── StmtK  Write
│   │   │   └── ExpK  c  IdV
│   │   ├── StmtK  Write
│   │   │   └── ExpK  d  IdV
└── .
```

## 语法错误

无。
