# C002-T56 完整准备源码复核

**需补1项保全断言，其余当前10新+1旧复用设计/窗口通过。** F-T56-01仅在missing-pending-root警告case：当前saved.is_dir不能证明实际A及邻居内容/metadata或SQL5表保留，需同case在cleanup前后独立全tree及5表比较。warning全为unpublished A且不含completed B关系正确。不要求其它合法cleanup放开权限的入口mode零改；不是产品红。

真实CLI B先completedadd，A随后Start trueCOMMIT后exit70，健康A先恢复再B exactreplay；坏A仅真实起始输入改坏，frozenWorkbook不变，原响应资格仍真实。按protocol顶层committed/request_id/original与error.detail pending_request/cause断言，B原响应全相等、pending A身份、5表和A tree保持，published0。compileAPI及错误wire fixture失败原文保留，不反写产品红。

全局pending形状来自真实登记变更5项；合法未提交孤儿的业务原件仍在，坏globalindex在cleanup之前拒绝。UUID三个名字与owner.internal_id同步改变避免前层错关联，noUUID必须unknown保留，合法UUID清理可由旧CLI故障解除后的oldcontainer消失/单请求提交证明，不造第二镜像test。已清completeddelete unit仅no-fault healthy，不叫OR捕获；专属CLI子进程配置cleanup-before-remove，成功重放无本delete-id marker告警，Store5表保持。

FS首4项此前scope保留：实际CREATE|EXCL元数据+write后模型sync failure+失败补偿rendezvous；同A unlink/错误B保全/markerIo创建A保留/3op owner-beforecontainer顺序。仅直接FSAPI，不说schema/真实交易或断电；由OwnedTempDir和原scope Release/join回收。CLI真实Exit70/output等待回收、环境仅子进程，没有sleep或假成功。newtest简化仅local variable/JSON排版，productionprefix和其余wholefiles一致。

当前11/11合并baseline（10新增+1复用）原文SHA与source见JSON，2准确静态证明见t56-static-review。修补此断言后需冻结新source及必要基线；最终15个映射/实际失败/工程和M2均未授PASS。
