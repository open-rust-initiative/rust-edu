## Mini DB by YY

由于我的小组内只有我一人，因此多在本地开发，只有最后一次commit

我尝试在Rust中编写数据库管理系统（DBMS），以了解数据库系统的开发实践。我按Let's Build a Simple Database教程（基于SQLite）来实现它。

由于对rust的学习掌握有限，大部分实现细节都需要参考、拼接github上优秀的rust db项目，但在阅读代码（ctrl cv）的过程中也学会了不少。

感谢老师和学长的付出！

## 实现的功能
- 添加用于拆分节点和更新父节点的测试用例
- 在内部节点上实现拆分。
- 扩展测试用例以确保插入按预期工作，并在过程中debug。
  - 为插入测试添加基于属性的测试。
  - 修复通过基于属性的测试发现的bug。
- 实现按ID查找
- 实现SQLite的删除操作。
- 实现对B+树的删除。
  - 在叶节点上实现删除。
  - 在内部节点上实现删除。
  - 在相邻节点指针/键值＜N后实现合并。
  - 实现内部节点的合并。
- 多线程索引并发控制。
- 支持对B+树的并发插入。
- 支持对B+树的并发选择。
- 支持对B+树的并发删除。
- 测试并发插入+选择；
- 测试并发删除+选择；
- 测试并发插入+删除；
- 测试并发插入+选择+删除；

## 原仓库

- [mini-db-yy origin](https://github.com/Icecens/mini-db-yy)

## References
- [johns code](https://johns.codes/blog/build-a-db/part01)
- [build a simple db](https://cstack.github.io/db_tutorial/)
- github