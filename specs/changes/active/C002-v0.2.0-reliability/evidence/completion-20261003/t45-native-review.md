# T45 当前原生载体独立审查

结论：**PASS，限定当前已核的 macOS aarch64 / APFS Data 载体和可构造输入。** 无剩余必须补验的可构造T45义务，可以按此scope结束T45并继续独立完整Spec审查。物理非法UTF8名字的engine遍历仍为 `not_run/environment_blocked_by_carrier`；本报告不授M2、全产品、全部macOS文件系统或旧9项native变体执行。

Reviewer未编写探针或生产代码，只读检查probe脚本/实际记录、合同、既有原生回归及输入SHA；未运行Cargo或重新运行probe。另只读执行df/mount及metadata确认实际载体，并检查自有fixture已回收。准确SHA与字段见同名JSON。

当前候选 `b2dea33aeccb40d3a344aeaa50aa6bd3c8e354ac` 与T57工程的134源码/fixture/config输入逐SHA相同，从4487到当前所有Rust/Cargo/config/fixture无改动。固定engine SHA `66aa6dc7eba731b6a8f00d54fba5d4fec8f13e58695a14c77f7f42d72230b154` 与真实Cargo产物及当前文件相同；实际uname=arm64、Mach-O arm64 engine、Rust/nextest host=aarch64-apple-darwin、`sheltie --version`成功。该基础沿用已独审原生工程run `fca0838a-afa4-436c-af23-0403e735b3b9`：948/948、0skip/LEAK及5compilefail。default/core无I/O边界不因本次证据探针改变。

实际`df /private/tmp`绑定 `/dev/disk3s5`，mount原文为 `/dev/disk3s5 on /System/Volumes/Data (apfs, local, journaled, nobrowse, protect, root data)`；tmp与该mount的st_dev均16777234，与记录一致。成功diskutil按实际device查询得APFS，不把`/`的另一个system-volume记录当tmp载体。初次sandbox DiskManagement不可用exit1、直接tmp参数query exit1、root-volume query及成功carrier-device query各独立保留，不删除旧失败或合并成一次成功查询。

合法场景确执行固定native engine `workbook add`：Unicode外部源目录与Unicode管理根，成功id=two-step，四个源文件的完整SHA由Python hashlib独立计算，实际安装副本逐文件SHA等同、源bytes未变，scope结束后自有根确不存在。此例只验证外部目录/Home的Unicode路径及真实安装，不称全部Unicode文件名、归一化、locale或完整Work生命周期。

非法argv确以bytes参数启动真实子进程，最后DIR含原始FF byte，退出2、stdout为空、stderr是实际invalid UTF-8参数拒绝，指定Home未建。它是前端argv拒绝，不能替代物理文件名遍历。另同源码948原文确有PASS：`sheltie-cli::os_process home_resolution_rejects_unrepresentable_roots_without_selecting_an_alternative`，实际CLI子进程分别覆盖非法SHELTIE_HOME、非法HOME及显式--home优先序。`fsx::tree_tests::directory_reader_rejects_non_utf8_entry_names`只是纯byte-interface测试，未把其PASS写成真实bad-name目录遍历。

物理负例分别调用真实`os.open(O_CREAT|O_EXCL)`和`os.mkdir`创建自有根内的bad-FF普通文件与目录，两个都由OS返回errno92/EILSEQ，created=false。脚本未伪造Metadata/目录，也未调用engine遍历不存在的物理bad对象。故**物理engine fixture未执行**，保持 `not_run/environment_blocked_by_carrier`；此结论只针对这两个实际尝试和当前APFS Data载体，不推断所有macOS文件系统都会拒绝。当前可构造输入及其余Spec/合同审查有独立证据，不被这个载体限制阻断。

旧`mac-native-preflight.json`仍声明d278候选，当前文件SHA匹配声明的b972ca9ca81296a260e8ed2d2e426b6549618a67f357432ca86c5bf4ffad1af0。但它没有找到独立immutable历史原件，且不在b2 Git tree，故**历史byteforbyte保真未独立确认**；修后boundary已明确false。当前native新probe不依赖或移植旧记录，旧文件本身未改，不伪称旧数据已证明当前候选。

本审查允许Owner在明确上述范围后关闭T45，完成Task/治理与证据收尾，再继续完整Spec/M2独审。其他平台为 `excluded_by_user`，usage未知仍null；不增加发布、Host配置或虚拟机安装动作，也不把其他package的实际使用/质量/成本义务转换成已完成。
