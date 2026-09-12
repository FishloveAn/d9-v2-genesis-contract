# 创世数据契约 — 0.1.0-rc.5

状态：待下游评审。关联：[D9-380](https://linear.app/d9-network/issue/D9-380)。
这是导出器、native builder、独立验证器共同使用的输入边界。传输 DTO
与现有 RuntimeGenesisConfig 分开；不能把本 JSON 直接当链配置启动。

## 1. 版本、编码与边界

权威机器结构为 [schema.json](schema.json)，由 [model.rs](src/model.rs) 生成。
所有 DTO 层均拒绝未知字段、重复字段和缺字段。Option 必须显式为 null
或值；缺字段不等价于 null。传输层全部 camelCase。

| 类型 | 编码 | 校验 |
|---|---|---|
| Amount | 无前导零的十进制字符串 | 0..u128::MAX；金额、Merit、LP 均不能经 JS Number |
| Millis | 同样的十进制字符串 | 0..u64::MAX；业务时间单位 Unix 毫秒 |
| Count | 同样的十进制字符串 | 0..u64::MAX；不使用浮点数 |
| Address | D9 SS58 prefix 9 | AccountId32 长度、checksum、canonical roundtrip；不接受 prefix 42 别名 |
| Digest | 64 个小写十六进制字符，无 0x | 32 bytes；具体算法由字段定义，不混用链 hash 与 SHA-256 |
| Commit | 40 个小写十六进制字符 | Git source revision；不能只写分支名 |
| assetId/session/decimals/bps | JSON 非负整数 | Rust u32/u8 的范围；这些字段不承载大额金额 |

JSON Schema 描述结构和文本模式；SS58 checksum、整数精确上界和跨字段
不变量还必须通过 Rust decode + validate。仅通过通用 JSON Schema 不等于
契约验证通过。现有嵌套 runtime domain types 使用 snake_case；转换通过
typed adapter 完成，roundtrip 测试锁定其行为。

不允许自发增加“可忽略字段”。任何必填字段、类型、单位、默认语义、
接受/拒绝规则、来源归属或 digest 编码变化，都要升级契约版本并重新评审
全部样本。RC 阶段也必须提交 schema/rules/inventory/golden 的一致变更。

## 2. 顶层结构

| 字段 | 内容与责任 |
|---|---|
| contractVersion | 精确等于 d9-native-genesis/0.1.0-rc.5 |
| purpose | synthetic-fixture 或 migration-input；前者只能用于合成对照 |
| source | 固定 finalized pin 的链 genesisHash、blockNumber/blockHash/stateRoot、timestampMs、runtimeSpecVersion/metadataDigest、规范化源投影及证据引用；由 D9-378 提供 |
| build | nodeCommit、palletsCommit、cargoLockDigest、runtimeConfigDigest、wasmDigest、nativegenCommit |
| state | 提交给 native builder 的迁移状态；字段不得静默省略 |
| changes | 有名字和具体记录的 top-up、refund、D9/asset rehome、reward credit、expiry exclusion 与未决项 |
| bootstrap | sudo、validator 五类 session key、12 个 admin role、SDK 派生 pallet 账户及已决 AMM 参数 |

build 中的所有 digest 都是对对应原始文件 bytes 的 SHA-256。runtimeConfigDigest
约束本 DTO 未展开的完整 V2 runtime 配置，例如 assets metadata/owner/minBalance、
pause、backup validators 和其余治理参数。D9-370 必须在组合前检查实际文件
bytes；本 crate 不读取这些外部文件，也不认证 revisions/ABI。

authority 数据与迁移金额必须形成一份确定的完整输入，再做检查和 native
build。不得把 bootstrap 的测试网 endowment 追加到迁移余额中。需要的资金
通过已有来源和明确的 rehome/credit 表达，不能由构建工具默认铸造。

Session、NodeRegistry、Liveness 应由同一 validator 记录投影。node 29a01f1
当前 FRAME SessionKeys 只有四项：babe、grandpa、im_online、authority_discovery；
契约的 imOnline/discovery 分别映射到后两项。D9SessionKeys 使用
block_authority=babe、grandpa、liveness、discovery；Liveness 使用
account/liveness。因此契约有五类 key，但不能把第五项强塞进当前 FRAME
SessionKeys。目标 node revision 改变时 D9-370 必须重新核验映射。
额外 session key、缺少派生表或写入后
不一致都必须由 D9-370 最终配置检查拒绝，不能退到启动后的 sudo 补写。

## 3. 每类迁移数据

| Source → state | 唯一键 | 单位、转换及 native owner |
|---|---|---|
| balances → balances | account | D9 12 decimals；free+已批准 reserved refund，加明确 top-up/credit/rehome；Balances 自身写引用计数 |
| assets → assets | assetId；每本 holders 按 account | USDT asset 1 从 2→6 decimals，checked ×10000；其他已声明资产保留精度；holderCount/totalSupply 必须核对实际记录 |
| referrals → referrals | child | 完整 (child,parent) 集合；DirectReferralsCount 由 pallet 逐 parent 派生 |
| burnAccounts → burnAccounts | account | D9 raw ×1；source.remainingDue+paid=target.balanceDue=3×burned，target.paid=source.paid |
| merchantAccounts → merchantAccounts | account | greenPoints→merit、两个 relationshipFactors 均 ×10000；redeemedD9 ×1 |
| merchantExpiries → merchantExpiries | account | 原值毫秒，剔除明确列出的已决 orphan rows 后集合相等 |
| votingInterests → votingInterests | account | total 原值，delegated=0；原委托图不携带 |
| locks → locks | account | account/frozenBy/session 原值；pallet 自己 set_freeze，禁止 raw storage 模拟冻结 |
| lp → lp | positions.account | 每个 LP 原始整数 ×1；sum=源 total；不执行 sqrt，不改变 owner，不套 500bps 容差 |
| counters → counters | singleton | burnVolume、merchantVolume、accumulativeRewardPool、lastSession 全部显式；VolumeAtIndex 由 runtime current_volume() 派生 |
| legacyRewardCredits → changes.rewardCredits | account | 逐户源债务；从 rehome 后矿池减去等额总和，发行量中性 |

同一 account 出现在不同 map、不同 assetId 是合法状态。唯一性针对完整
存储键，在全部输入中检查；不能按原始地址文本去重、累加重复行或后写覆盖。

source 是带来源引用的规范化投影，不是原始 RPC 存储 dump。burn 的金额
权威是 main_pool.portfolios；burner.accounts 只补 lastInteraction 和
referralBoostCoefficients，补充源也必须绑定同一 block/codeHash/ABI。
原始 burner-only 残留、缺失 join、未经证实 ABI 和未知新字段必须在 D9-378
显式处理/升级，不得靠构造这个 DTO 把它们悄悄消失。

source.datasets 必须准确覆盖 12 类数据，包括空数据集。recordCount 和
payloadDigest 对应本契约的规范化记录；assets 的顶层记录是 AssetBook，
每本另有 holderCount；lp 的记录是 positions；counters 是单记录。
这些自洽检查之外，D9-307/173 必须把原始源覆盖、总量、pin 和 ABI 与独立
记录核对。由同一个错误 exporter 同时伪造 records/count/hash 不能由本检查
证明为真，也不得通过发布关卡。

计数不固定为历史常量。#23,800,000 的商户 53,918 / 5,018−10=5,008，
与 #23,802,000 的十名 LP 数据来自不同时点，不能拼成真实冻结快照。
实际数量与 exception list 都必须绑定该次 pin。

## 4. 时间、例外和经济边界

- Merchant createdAt 必须非零且不晚于 source.timestampMs；Some(lastConversion)
  同样检查。不使用 genesis Timestamp::Now 或机器墙钟。None 不变为 Some(0)。
- Burn 的 lastBurn、lastInteraction、Some(lastWithdrawal) 均以 source.timestampMs
  为上界。时间顺序与额外异常不得凭理想模型修复。
- DEC-9：过期、超出现行新订阅期限、expiry<createdAt 的已存账户到期记录
  均携带。合成 fixture 的 orphan 只是机制样本，不替代真实十地址批准清单。
- DEC-16：历史推荐环保留；self-referral、duplicate child 仍拒绝。运行期
  遍历防重复付奖另由 D9-192 负责。
- AMM D9 reserve ×1、USDT ×10000，ED 无额外加项。储备底限按 Yvan 2026-09-12 的 D9-190 决策改为各 60 万整币（RC5，取代 RC4 的各 100 万）：
  D9 600000000000000000 raw（12 decimals），USDT 600000000000 raw
  （6 decimals）。这是 V2 流动性保护政策，不是 V1 最小值的单位换算；
  不改变迁移储备金额、LP owner 或 LP 数量，也不构成 oracle 安全保证。
  与 LP-token MINIMUM_LIQUIDITY 和 DEC-17 redemption price floor 分开。
  30bps fee / 500bps post-migration tolerance 使用既定决策；runtime 的
  swap/remove_liquidity 储备检查由 D9-190 实现。
- DEC-17：上线无 redemption price floor。当前 runtime 的删除/禁用由
  D9-185 落地；不能仅凭 bootstrap.redemptionPriceFloor=false 认为代码已改。
- 余额算术顺序：先验证源 totalIssuance，reserved refund→明确 ED top-up→
  所有 rehome 同时扣源、再加目的→source reward credits 从矿池等额扣除。
  asset rehome 单独列出 assetId/from/to/sourceAmount；先移动再缩放。
  任何 drop、缺失资金来源、隐含 endowment 都不在本 RC 的接受集中。

司法锁 X1 按 D9-195 / DEC-20 的 2026-09-03 已决规则处理：
source.locks 保留全部原始行；changes.excludedJudicialLocks 是必填数组，
仅允许地址 wGdkbufhUKmNWjEXPqVdunXUAZqFRw5WSbDamTExHd3rFMq
（0x60a1014576a16a09fdcbfdf69f27a1d3415231be5f27aa1c64928a8dd823176e）。
每条包含 account、reason="missing_system_account"、v1LockAmount="1"、
blockHash（必须等于 source.blockHash）、systemAccountExists=false。
该地址必须存在于 source.locks，同时不在 source.balances 或 state.balances，
且不得重复。state.locks 必须精确等于 source.locks 减去已验证排除行；
未知账户、过期 pin、改变的 marker、重新出现的账户、缺失或重复排除均拒绝。
若后续 pin 该账户已存在且具备资金，则不作排除，保留冻结。

这些字段是同 pin RPC 观察的声明，不能自行证明 System.Account 不存在。
D9-378 / D9-173 必须认证原始 storage absence 和 Balances.Locks 的唯一
council/ amount=1 marker，不能以余额投影缺行代替证明。Exporter 的
account_id_hex / reason / v1_lock_amount 收据由适配层转换为以上字段，
并补入被验证的 pin 与存在性观察。既有 1,968→1,967 是历史测量，
不是数量硬编码；合成 fixture 包含真实获准地址但其 pin、冻结人和观察均为合成。

RC3 的 LOCK 规则与错误消息中的 `council/ amount 1` 是上述条件的简写：
输入校验实际比较 `v1LockAmount == "1"`（解码后的整数等于 1），不会查询
链上 marker。唯一 `council/` marker 的原始存储证明仍由 D9-378 / D9-173
核验。本说明澄清已有条件，保留已确认的 rules.json 与错误消息 bytes。

2026-09-12，Yvan 已通过五项独立 ADR（inventory.json 的完整链接）裁定：

| 项目 | Provenance 与 V2 初值 | 独立交付义务 |
|---|---|---|
| Resolutions | NotMigrated；空 | V1 全量原始历史与无损解码，固定 pin、count/digest，外部永久档案在 cutover 前独立验证；当前 freezes 仍迁移 |
| ProposalFeeVolume | NotMigrated；0 | 只计 V2 实际 proposal-fee inflow；opening VolumeAtIndex 等于 burn+merchant+proposal(0)，不能把历史当新活动 |
| UserNonce | NotMigrated；每用户 0 | V1 历史只归档，与 V2 live records/queues/dedup 分离；相同历史 ID bytes 本身不要求改变 hash 格式 |
| CumulativeBridgedOut | NotMigrated；0 | V2 第一档（25% external / 75% AMM）；只累计成功 finalization 的 V2 gross |
| PendingOutbound | NotMigrated；空 | V1 每项义务在不可逆 cutover 前完成付款/退款或有明确批准且有资金支持的安排；未决阻断 cutover，归档不等于偿债 |

这五个初值没有可导入字段；实际 runtime config 和最终 raw spec 必须由
D9-370 / D9-173 按上述值独立核验。本输入 DTO 的 runtimeConfigDigest
仅绑定 bytes，不能证明 bytes 中的初值正确。未知字段仍拒绝，不得把
V1 的计数或历史塞入 DTO 来绕过 NotMigrated。对应规则为 FRESH_V2。

purpose=migration-input 现在可以通过输入一致性检查，但仍明确报告
releaseGateEvaluated=false。归档、legacy settlement、clean V2 processing、
最终初值和 watermark 的验证仍列为 independentEvidenceRequired，不能由
输入字符串或本检查结果证明完成。任何新的 changes.unresolved 或未来
null inventory 分类仍 fail closed；null 不是第四种 Provenance。真实排除名单、
合约资产去向和债务归属仍须绑定适用的已有/新裁定。

## 5. 不变量的责任分层

| 层 | 拒绝内容 | 负责人 |
|---|---|---|
| Decode | 错字段/类型/Option、非法整数、错误 SS58 | 本契约 |
| Input conformance | 重复、源/目标字段错配、单位、显式 delta、已付历史、账本/储备不等 | 本契约提供共享参考；每条责任见 rules.json |
| Pallet genesis | 存储唯一性、checked sum、非零时间锚点、delegated=0、freeze 效果、LP 全量写入 | D9-183/184/190 及各 pallet，不得因 precheck 已存在而省略 |
| Provenance | Runtime StorageInfo 和实际 JSON field coverage、source/WASM/pins/全部配置与最终 artifact | D9-307 / D9-370 |
| Independent fidelity | 解码最终 raw spec，独立与经批准的固定源比较；之后链上读回 | D9-173 |
| Release | 两机 runtime/state 重现、真实数据演练、下游评审和未决项裁定 | 各既有发布关卡 |

独立 verifier 可以复用字段定义、编码和 golden fixtures，不能调用 builder
的同一转换函数来生成自己的“预期值”后宣称独立证明。本 crate 的 validate
是输入参考检查，不能替代 D9-173 的独立输出读取。

## 6. 摘要和最终 artifact 绑定

Canonical JSON：UTF-8、无空白/末尾换行、对象 key 字典序，递归排序；
数组保留顺序；整数字符串保持原字面值；不做 Unicode normalization。

- inputDigest = SHA256("d9-genesis-input-v1\0" || canonical(input))。
- datasetDigest = SHA256("d9-genesis-dataset-v1\0" || datasetName || NUL ||
  每条记录的 [u64 big-endian byteLength || canonical(row)])。
  顶层 row bytes 先按字节字典序排序；不删除重复，row 内数组仍保序。
- contractDigest = 对 {domain:"d9-genesis-contract-bundle-v1", version,
  schema, inventory, rules} 使用上述 inputDigest 算法。
- 文件 digest（raw spec、report、Cargo.lock、WASM、完整 runtime config）
  都是 SHA256(exact bytes)，不复用 dataset 协议。

字段名中的 source block/genesis hash 保持该链原生 hash 算法的 bytes，
只是以 64 hex 表示；不得拿文件 SHA-256 替代链 block hash。

[ArtifactBinding](binding.schema.json) 是外置 receipt，必须在 authority 和
迁移状态全部合成之后记录。它绑定 contractVersion/contractDigest/inputDigest、
build identity、rawSpecDigest、checkerReportDigest。最终 spec 不包含对自身
bytes 的 hash，避免循环。D9-307 的嵌入 d9Provenance 扩展与本外置 receipt
须由 D9-370 一起核对，不能变成第二个绕过入口。

verify_binding 只验证所给 bytes 的 hash 与输入关系；不判断 raw spec
是否能启动、不确认 report 内容代表哪一级批准、不验证源真实性。D9-307
必须验证 report 的类型和检查结果，D9-173 必须独立读最终 state。

## 7. 已接受代码的复用

D9-307 修复补丁按其附件取回，原始 patch SHA-256：
`c9003f5ea1bac14870a7deb15cbc15bbb8633b983f90de5c2d1c8f7d6ab5fd16`。
来源 tip `c8b123881d27671615906f3102e30c932580e491`，基线 `aec1167`。
本分支先单独恢复其 317 行修改，再叠加本契约；没有改造该 flat
MigratedItem API、没有第四种 Provenance、没有新增 runtime storage。

独立仓库校验 inventory.json 的传输声明结构；pallet 仓的 adapter 使用原有
validate_manifest 独立检查已分类条目。对当前
D9 源码 storage 名称的测试能发现增删漂移，但不能证明所有 SDK storage、
宏展开、字段路径、跨 pallet 派生关系或其业务分类已经获认可。


## 8. 独立仓库归属 — 2026-09-11

本仓是共享契约、schema、规则和样本的唯一维护位置。业务 pallet 类型转换、
d9-core 的声明校验桥接、读取 pallet 源码的覆盖测试迁回 d9-v2-pallets 的
d9-genesis-adapter。原有 Rust typed adapter 的责任与字段没有改变。
消费者固定引用本仓 commit；不得依赖另一工作区的相对路径或保留规范副本。
RC3 保留 RC2 的 X1 schema，更新版本、规则、inventory 和 fixture/binding digests；消费者必须重新固定 commit 并评审全部样本。
