# T33冻结候选历史变异

产品提交06c3af3，验证clone5344b265。runtime第一阶段1773项完整，所有requested/listed/outcomes无漏无重：1205caught、209compiler unviable、359missed。core720保留其原run身份，core4当前workspace为3caught/1missed。

T34确认并修复R22–R24后改变源码，旧pipeline在runtime workspace目录创建前由source guard停止exit1，原文pipeline-exit-before-t34.*保留。此停止不是安全跳过，不代表新候选或M1通过。旧执行、raw与逐成员SHA保留；raw.tar.gz无损归档，loose索引可读。历史脚本只作原文，不是继续执行入口。
