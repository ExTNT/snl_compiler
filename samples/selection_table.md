# `samples/selection.snl` 符号表

## 作用域结构

SNL 为过程声明使用嵌套作用域。符号表组织为哈希映射栈，每个作用域级别一个。查找时从最内层作用域向外层遍历。

总作用域数: 1

| 作用域 | 级别 | 名称 | 种类 | 类型 | 参数 | 行号 |
|--------|------|------|------|------|------|------|
| 全局 | 0 | a | VarId | array[0,4] of integer | | 2 |
| 全局 | 0 | i | VarId | integer | | 3 |
| 全局 | 0 | j | VarId | integer | | 3 |
| 全局 | 0 | minIdx | VarId | integer | | 3 |
| 全局 | 0 | n | VarId | integer | | 3 |
| 全局 | 0 | selection | ProcId | | | 1 |
| 全局 | 0 | temp | VarId | integer | | 3 |

## 语义错误

无。
