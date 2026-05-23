# `samples/char_array.snl` 语法树

## 抽象语法树

```
ProK
├── PheadK  chararr
│   ├── VarK
│   │   ├── DecK  ArrayK  [0,4]  CharK  s
│   │   ├── DecK  IntegerK  i
│   ├── StmLk
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  s  IdV[0]
│   │   │   └── ExpK  Const  'H'
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  s  IdV[1]
│   │   │   └── ExpK  Const  'e'
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  s  IdV[2]
│   │   │   └── ExpK  Const  'l'
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  s  IdV[3]
│   │   │   └── ExpK  Const  'l'
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  s  IdV[4]
│   │   │   └── ExpK  Const  'o'
│   │   ├── StmtK  Assign
│   │   │   ├── ExpK  i  IdV
│   │   │   └── ExpK  Const  0
│   │   ├── StmtK  While
│   │   │   ├── ExpK  Op  <
│   │   │   ├── StmLk
│   │   │   │   ├── StmtK  Write
│   │   │   │   │   └── ExpK  s  IdV[i]
│   │   │   │   ├── StmtK  Assign
│   │   │   │   │   ├── ExpK  i  IdV
│   │   │   │   │   └── ExpK  Op  +
└── .
```

## 语法错误

无。
