# `samples/factorial.snl` 符号表

## 作用域结构

SNL 为过程声明使用嵌套作用域。符号表组织为哈希映射栈，每个作用域级别一个。查找时从最内层作用域向外层遍历。

总作用域数: 2

| 作用域 | 级别 | 名称 | 种类 | 类型 | 参数 | 行号 |
|--------|------|------|------|------|------|------|
| 过程级别 1 | 1 | m | VarId | integer | | 5 |
| 全局 | 0 | fact | ProcId | | (m: integer) | 5 |
| 全局 | 0 | factorial | ProcId | | | 1 |
| 全局 | 0 | n | VarId | integer | | 3 |
| 全局 | 0 | result | VarId | integer | | 2 |

## 语义错误

无。
