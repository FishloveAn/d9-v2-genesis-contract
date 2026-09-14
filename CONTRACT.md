# 创世数据契约 — 0.1.0-rc.7

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
| threshold | JSON 非负整数 | Rust u16（0..65535）；multisig 阈值，校验另要求 2 ≤ threshold ≤ n |

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
| contractVersion | 精确等于 d9-native-genesis/0.1.0-rc.7 |
| purpose | synthetic-fixture 或 migration-input；前者只能用于合成对照 |
| chain | RC7：network（mainnet/testnet）、id、name、chainType（Development/Local/Live），与 purpose 绑定；见第 9 节 |
| custody | RC7：ceremonyNonce（32 字节小写 hex），每个 custody signatory 的 proof-of-possession 都绑定它；见 9.6 |
| source | 固定 finalized pin 的链 genesisHash、blockNumber/blockHash/stateRoot、timestampMs、runtimeSpecVersion/metadataDigest、规范化源投影及证据引用；由 D9-378 提供 |
| build | nodeCommit、palletsCommit、cargoLockDigest、runtimeConfigDigest、wasmDigest、nativegenCommit |
| state | 提交给 native builder 的迁移状态；字段不得静默省略 |
| changes | 有名字和具体记录的 top-up、refund、D9/asset rehome、reward credit、expiry exclusion 与未决项 |
| bootstrap | sudo 与 usdtOwner 的 k-of-n multisig、validator 五类 session key、12 个 admin role 的 multisig、SDK 派生 pallet 账户、已决 AMM 参数及完整 assetIds 集合 |

build 中的所有 digest 都是对对应原始文件 bytes 的 SHA-256。runtimeConfigDigest
约束本 DTO 未展开的完整 V2 runtime 配置，例如 assets metadata/sufficient/minBalance、
pause、backup validators 和其余治理参数。RC7 起资产 owner 不再只由该 digest 约束：
asset 1 owner 必须等于 bootstrap.usdtOwner.address，其余已声明资产 owner 必须等于
bootstrap.sudo.address（CUSTODY，第 9 节），资产 ID 集合由 bootstrap.assetIds 声明。D9-370 必须在组合前检查实际文件
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
  同样检查，并要求 lastConversion >= createdAt（Yvan 2026-09-13 决定）。相等及 None 有效；逆序记录拒绝整个输入，不修正、不丢弃、不保留例外。不使用 genesis Timestamp::Now 或机器墙钟。None 不变为 Some(0)。
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

## 9. RC7 托管、链身份与资产集合 — 2026-09-14

依据 [D9-400](https://linear.app/d9-network/issue/D9-400) 中 Yvan 2026-09-14 02:28 UTC
的裁定（S-2 执行 DEC-21 multisig 托管）以及审计 S-6、评审 R-4。RC6 输入不再兼容：
`bootstrap.sudo` 与 `bootstrap.admins[].account` 由单一地址改为 multisig 结构。

### 9.1 Multisig 托管（CUSTODY）

| 字段 | 结构 | 校验 |
|---|---|---|
| bootstrap.sudo | MultisigAuthority | 见下 |
| bootstrap.usdtOwner | MultisigAuthority | asset 1 的 owner；与 d9-v2-tools derive_admins 的 usdt-owner 角色对应 |
| bootstrap.admins[] | {pallet: AdminPallet, multisig: MultisigAuthority} | pallet 为 12 个固定 slug 之一（未知 slug 解码时拒绝）；12 个 pallet 恰好各一次（admin_coverage） |
| MultisigAuthority | {address, threshold: u16, signatories: Signatory[]} | 2 ≤ threshold ≤ n ≤ 20；signatories 按 AccountId32 字节严格升序（不重复）；address 必须等于 multisig_account(signatories, threshold)；身份检查见下表；每个 signatory 须有 PoP（9.6） |
| Signatory | {address, evidence} | address 即 sr25519/ed25519 公钥；evidence 为恰好一个键的外部标记枚举 |
| evidence.signature | {scheme: "sr25519"\|"ed25519", signature: 64 字节小写 hex} | 只接受 raw proof；未知 scheme（包括 ecdsa）、`message` 字段或长度错误在解码时拒绝 |
| evidence.enclaveAttested | {attestationDocument: COSE_Sign1 字节的标准 base64（带填充），expectedPcr0: 96 位小写 hex（48 字节 SHA-384），popMessageSha256: 64 位小写 hex} | hex 长度在解码时检查；文档非空、规范 base64、解码后 ≤ 16 KiB，PCR0 非全零，popMessageSha256 == SHA-256(custody_pop_message)，expectedPcr0 ∈ BLESSED_SIGNER_PCR0；本契约不验证文档本身 |

| 拒绝代码 | 条件 |
|---|---|
| multisig_threshold_too_low | threshold < 2（1-of-n 等于单钥权限） |
| multisig_signatory_limit | n > 20（runtime MaxSignatories） |
| multisig_threshold_exceeds_signatories | threshold > n |
| multisig_signatory_order | signatories 未按字节严格升序或重复 |
| custody_pop_weak_key | 任一 signatory、multisig address、validators[].account 或 session key 是可被任何人伪造签名的公钥：全零（sr25519 Ristretto 单位元，也是 ed25519 order-4 点），或任何 ed25519 small-order 点的编码（含非规范编码）；在签名验证之前检查，与 scheme 无关 |
| ed25519_key_not_prime_order | scheme 为 ed25519 的 signatory 或 grandpa session key 不是曲线点、不是规范编码、带 torsion 分量或为单位元（torsion twin `A + T` 用 `A` 的私钥即可签名，会让一个私钥冒充多个 signatory） |
| multisig_self_signatory | signatory 等于其 multisig address |
| dev_key_authority | multisig address/signatory、validators[].account 或任一 session key（babe、grandpa、imOnline、discovery、liveness）等于开发密钥。以下每个 URI 均含 sr25519 与 ed25519 公钥及 ecdsa 账户 blake2_256(压缩公钥)，共 183 个：sp-keyring 48.0.0 的 //Alice、//Bob、//Charlie、//Dave、//Eve、//Ferdie 及各自 //stash、//One、//Two，sp-core DEV_PHRASE 根密钥；d9 自身提交的 authority SURI：//LocalValidator1..6（d9-v2-node e13a19d `runtime/src/genesis_config_presets.rs` 的 LOCAL_DEV_*_PUBS 与 `local-keys/README.md`）、//Mainnet0..5//{stash,babe,imon,audi,live,grandpa}、//OCWTest//{Babe,Grandpa,Liveness,Discovery} |
| multisig_nested_signatory | signatory 等于本文档中另一个角色的 multisig address |
| authority_pallet_account | multisig address、signatory 或 validators[].account 等于 ammAccount、miningPoolAccount 或以 b"modl" 开头的 PalletId 账户 |
| authority_session_key | multisig address、signatory 或任一 validators[].account（含该 validator 自身）等于任一 validator 的 session key |
| invalid_session_key | session key 为零，或同一 key 出现在任一 validator 的两个 slot 中（同一 validator 内或跨 validator） |
| authority_validator_account | multisig address 或 signatory 等于任一 validators[].account |
| multisig_address_not_derived | address 不等于 multisig_account(signatories, threshold) |
| authority_role_not_distinct | sudo.address 等于 usdtOwner.address 或任一 admins[].multisig.address |
| sudo_signatory_not_independent | sudo 的任一 signatory 同时是 usdtOwner 或任一 admin 的 signatory（related_path 指向该 admin 侧 signatory） |
| rehome_source_not_allowed | changes.rehomes[].from 或 changes.assetRehomes[].from 是 custody address/signatory、validator 账户、session key、miningPoolAccount 或 ammAccount（related_path 指向该身份，见 9.7） |
| rehome_destination_not_allowed | rehome 目标是开发密钥、可伪造公钥、custody address/signatory、validator 账户或 session key，或 D9 目标不是 miningPoolAccount/ammAccount、资产目标不是 ammAccount（消息写明身份类型，匹配到文档内身份时 related_path 指向它，见 9.7） |
| custody_attested_quorum | 某角色中 enclaveAttested signatory 数量 ≥ threshold（最多 threshold − 1 个） |
| custody_pop_invalid | 签名未能以该 signatory 公钥验证 9.6 的 raw 消息（任何 network、chain.id、role、multisig address、signatory 或 nonce 变化都会使其失效） |
| custody_pop_attestation_malformed | enclaveAttested 的文档为空、非规范 base64、解码后超过 16 KiB，或 expectedPcr0 全为零 |
| custody_pop_attestation_binding | popMessageSha256 不等于该 signatory、role 与仪式的 SHA-256(custody_pop_message) |
| custody_attestation_pcr0_not_blessed | expectedPcr0 不在 `BLESSED_SIGNER_PCR0` 中；该集合目前为空，因此所有 enclaveAttested signatory 都被拒绝 |

契约绑定地址（Yvan 2026-09-14 裁定 CS-1 = A）：公开函数
`multisig_account(signatories, threshold)` 计算 pallet_multisig `multi_account_id` 的规则
blake2_256(SCALE(b"modlpy/utilisuba", 升序 Vec<AccountId32>, u16 threshold))，已对照
pallet-multisig 45.0.0 `src/lib.rs:634-638`（d9-v2-node 锁定的版本；48.0.0 `src/lib.rs:645-649` 逐字节相同）。validate 对 sudo、usdtOwner 与每个 admin 要求
address 等于该推导。这是有意保留的第二份实现：d9-v2-tools `d9-bootstrap derive-admins`
保留自己的小副本，因为密钥生成二进制不得依赖本 crate（会引入 sp-core）。两份实现固定于同一组
`@polkadot/util-crypto` golden vectors，并由 d9-v2-tools `d9-genesis-composition`
`custody::cross_check_derivations`（tools PR #72，尚未合并）交叉校验二者。producer 再把 sudo.key、
各 `<pallet>.admin` 与 asset 1 owner 投影为这些地址；其他已声明资产的 owner 等于
sudo.address（node MainnetAssetOwners）。

multisig 托管适用于所有 purpose 与阶梯的每一级（Keel、Χ、Genie、Ψ、Ω）；没有单钥演练例外
（Yvan 2026-09-14）。开发密钥与可伪造公钥的拒绝同样对所有 purpose 生效，合成 fixture 也不例外。

角色区分（CS-4，Yvan 2026-09-14）：sudo.address 必须不同于 usdtOwner.address 和每个
admins[].multisig.address。允许：12 个 pallet admin 共用一个 multisig；usdtOwner 等于某个
admin multisig。

sudo 与 admin 完全独立（Yvan 2026-09-14 04:34 UTC，取代 03:33 允许 sudo/admin 共享 signatory 的裁定）：
sudo.signatories 与 usdtOwner 及每个 admin 的 signatories 不得相交（usdtOwner 视为 admin 侧）。
同一 signatory 仍可出现在多个 admin 侧 multisig 中。该不相交条件已涵盖 quorum containment（CS2-3）。
由于 ed25519 torsion twin 已被拒绝，不能用同一私钥的另一 AccountId32 绕过这一条件。

公钥安全（CR3-01 与决定 A，Yvan 2026-09-14 05:46 UTC）：签名验证之前，本契约拒绝全零 sr25519 公钥
（schnorrkel 0.11.5 经 curve25519-dalek 4.1.3 `CompressedRistretto::decompress` 规范解码，单位元只有这一种编码），
拒绝任何 ed25519 small-order 点编码（curve25519-dalek 4.1.3 `CompressedEdwardsY::decompress` 接受非规范
y 与符号位，`is_small_order`），并要求 scheme 为 ed25519 的 key 是规范编码、torsion-free、非单位元的点
（`compress`、`is_torsion_free`）。本契约显式拒绝的公钥只有：全零 sr25519 公钥、任何 ed25519 small-order
编码，以及 scheme 为 ed25519 的非规范、带 torsion 或单位元的 key。其他无私钥账户（pure proxy、派生账户、
本文档之外的 multisig）之所以无法通过，只是因为无人能为其产生有效签名；PoP 证明私钥存在且被持有，不证明
持有人就是预定的保管人，后者仍是仪式证据。

由 enclave 持有的 custody signatory（sudo-signer enclave 是 DEC-21 sudo 参与者，以后也可能有 admin-signer）
使用 enclaveAttested 证据（Yvan 2026-09-14 04:57 UTC，任何角色都可使用）：signer 的专用 attest 模式从 KMS
解密 seed、推导公钥，请求 NSM attestation document，其 `user_data = SHA-256(custody_pop_message(...))`，
然后清零并退出；enclave 不产生签名，其 extrinsic allowlist 不变。决定 B：每个角色的 enclaveAttested
signatory 最多 threshold − 1 个，因此至少需要一个经本契约验证的签名。决定 C：`expectedPcr0` 必须属于
`BLESSED_SIGNER_PCR0`；该集合只由新的契约 RC 在测量值记入 `d9-v2-docs` `docs/operations/pcr0-ledger`
之后加入，在 mainnet sudo-signer attest-mode EIF（CUS-221）构建并认可之前为空，因此 enclave 证明的 custody
目前无法通过。本契约只检查结构、消息绑定与 PCR0 集合，不声称验证 attestation；通过验证的 enclaveAttested
signatory 会出现在报告的 `pendingAttestationVerifications` 中，`check` 变为
`input-contract-conformance;attestation-verification-pending`，并附加 independentEvidenceRequired：producer 用
d9-enclave-common `attest_verify` 验证 attestation document（COSE 签名、以文档自身时间戳校验到固定 AWS Nitro
root 的证书链、PCR0 == expectedPcr0、user_data == popMessageSha256），且 expectedPcr0 等于 PCR0 ledger 中记录的
受认可 signer 度量值。COSE/X.509 栈不进入本 crate。

仪式要求（决定 D）：保管人用离线、专用的工具对 raw 规范消息签名，签名前工具必须显示解码后的各字段；
不接受浏览器或 dApp `signRaw` 产生的 proof。

### 9.2 链身份与 purpose 绑定（NETWORK）

| 规则 | 拒绝代码 |
|---|---|
| migration-input 必须 chainType=Live；Development、Local 拒绝 | chain_purpose_binding |
| network=mainnet 只允许 purpose=migration-input | chain_purpose_binding |
| id 为 1–64 个 [a-z0-9_]；name 非空、仅可打印 ASCII（0x20–0x7E）、无首尾空白 | chain_identity_label |
| mainnet 的 id/name 不得包含 test、dev、local、rehears、fixture、synthetic（子串匹配，不区分大小写）；testnet 的 id/name 必须包含 test | chain_identity_label |

Χ 演练使用真实数据但不是 mainnet：testnet 标签的 migration-input（Live）有效，
本契约不设“migration-input ⇒ mainnet”。尚无已裁定的 mainnet chain id，因此只做
标签一致性检查，不固定具体 id。chainType 的 Custom 变体在解码时拒绝。
producer 必须要求 bootstrap manifest 的 network 与 chain spec 的 id、name、chainType
等于本声明，并拒绝任何 bootNodes 条目及非 null 的 telemetryEndpoints。
标签一致不证明 spec 实际部署到哪个网络。

### 9.3 资产 ID 集合（ASSET_SET）

`bootstrap.assetIds` 是完整的 V2 pallet-assets ID 集合，严格升序，并包含每个
state.assets 的 assetId；没有迁移账本的新资产只在此声明。违反时代码为
asset_id_set。本契约不建模 assets 定义与 metadata（sufficient、minBalance、name、symbol；
owner 由 9.1 约束），因此精确集合相等由 producer 执行：`assets.assets` 与
`assets.metadata` 的 ID 集合都必须等于 assetIds，不得多也不得少，迁移资产的
metadata decimals 等于 state 账本。

### 9.4 解码前的版本检查

`parse` 返回 `Result<ContractInput, ParseError>`。它在严格类型解码之前读取顶层 `contractVersion`
字符串；若可读且不等于 d9-native-genesis/0.1.0-rc.7，返回 `ParseError::UnsupportedVersion { found }`，
严格解码失败则返回 `ParseError::Decode(message)`。调用方按变体匹配，不解析字符串。真实 RC6 文档因此报告
版本不符，而不是 `chain` 未知字段等类型错误。无法读取版本（非法 JSON、缺失、非字符串、
重复字段）时仍交由严格解码拒绝。validate 对已构造的输入继续检查版本。

### 9.5 报告与 producer 独立证据

`ContractReport` 新增 `pendingAttestationVerifications`：每个 enclaveAttested signatory 一项
（path、role、multisig、signatory、expectedPcr0、popMessageSha256）。列表为空时 `check` 为
`input-contract-conformance`；非空时为 `input-contract-conformance;attestation-verification-pending`，
并且只有此时才附加 attestation 验证义务。

independentEvidenceRequired 的 D9-400 项：producer 从这些契约地址投影 sudo.key、每个 `<pallet>.admin` 与资产
owner；每个 signatory 密钥属于其预定保管人（身份与保管记录；本契约已验证签名 PoP 并拒绝可伪造与非
prime-order 公钥，但无法判断有效 proof 来自谁）；assets.assets 与 assets.metadata 的 ID 集合等于 assetIds；
chain spec id/name/chainType 与 manifest network 等于 chain，且无 bootNodes、telemetryEndpoints 为 null；
存在 enclaveAttested signatory 时还需上述 attestation 验证。输入检查通过不代表这些已被证明。

### 9.6 Custody proof-of-possession（PoP）

验证顺序：结构、可伪造公钥、ed25519 prime-order、开发密钥、保留身份、地址推导、角色区分、sudo 独立性、
rehome 账户之后，才检查 enclave 证明数量上限并逐个验证 PoP，因此既有拒绝代码不变。唯一的消息构造函数是公开的
`custody_pop_message(network, chain_id, role: CustodyRole, multisig_address, signatory, nonce) -> Vec<u8>`，
role 为 `CustodyRole::{Sudo, UsdtOwner, Admin(AdminPallet)}`，其 `label()` 写入消息：

```text
message = b"D9-V2-CUSTODY-POP/1"                    (19 字节，无长度前缀)
       || field(network label)   "mainnet" | "testnet"
       || field(chain.id)        UTF-8
       || field(role label)      "sudo" | "usdtOwner" | "admin/<pallet>"
       || field(multisig address) 32 字节 AccountId32
       || field(signatory)       32 字节 AccountId32（sr25519/ed25519 公钥）
       || field(ceremonyNonce)   32 字节
field(x) = u64 big-endian len(x) || x              （与 dataset_digest 相同的长度编码）
```

签名直接覆盖 message 本身（决定 D：只接受 raw proof，没有 `<Bytes>` 包装形式）。
sr25519 用 sp-core 43.0.0 `sr25519::Pair::verify`（signing context `b"substrate"`，`src/sr25519.rs:49,265-269`）；
ed25519 用 `ed25519::Pair::verify`（`src/ed25519.rs:118-124`）。

字节级示例（单元测试 `custody_pop_message_matches_the_documented_byte_vector` 锁定）：network=testnet，
chain.id=`d9_testnet_fixture`，role=`CustodyRole::Admin(AdminPallet::D9Amm)`（`admin/d9-amm`），
multisig=`11`×32，signatory=`22`×32，nonce=`33`×32，共 200 字节：

```text
44392d56322d435553544f44592d504f502f31                                   "D9-V2-CUSTODY-POP/1"
0000000000000007 746573746e6574                                          "testnet"
0000000000000012 64395f746573746e65745f66697874757265                    "d9_testnet_fixture"
000000000000000c 61646d696e2f64392d616d6d                                "admin/d9-amm"
0000000000000020 1111111111111111111111111111111111111111111111111111111111111111
0000000000000020 2222222222222222222222222222222222222222222222222222222222222222
0000000000000020 3333333333333333333333333333333333333333333333333333333333333333
```

`enclaveAttested` 证据的 `user_data` 规则：`user_data` 必须恰好是 32 字节
`custody_pop_message_sha256(network, chain.id, role, multisig address, signatory, ceremonyNonce)`，
即上述 message 的 SHA-256，并与 `popMessageSha256` 相等。示例消息的 SHA-256 为
`f1e23b3ad4e34e4fdb7d135f4f088a2301dae85465a3aeea27edcb9b2ccf9281`（同一单元测试锁定）。complete fixture
全部使用签名证据；attestation 负例使用标为 `synthetic-not-a-real-attestation` 的合成文档，只覆盖本契约侧
检查，不是真实 Nitro attestation。接受路径只在单元测试中用测试专用的 PCR0 值演示。

合成 fixture 的 42 个 signatory、两个 grandpa key 以及 torsion-twin 负例由提交的
`examples/custody_pop_fixture.rs` 以 OS 随机数新生成（每个角色 2-of-3：两个 sr25519、一个 ed25519，全部 raw），
在内存中签名后只写出公钥、地址、签名与 nonce；seed 从不写入或打印，运行结束前扫描给定目录确认 seed 不存在。
torsion twin 的签名由 `tests/support/torsion.rs`（仅用于负例的攻击构造）生成，并在写出前以 sp-core ed25519 验证。

### 9.7 Rehome 账户（Yvan 2026-09-14 04:34 UTC；round-3 CR3-04、CS3-5）

V2 d9-merchant 没有 pallet 账户；商户兑付、节点奖励与 burn 提现都由 mining pool 支付；mining pool
不持有资产。因此：`changes.rehomes[].to` 只能等于 `bootstrap.miningPoolAccount` 或
`bootstrap.ammAccount`；`changes.assetRehomes[].to` 只能等于 `bootstrap.ammAccount`（这两个账户已在
composition 中按 runtime PalletId 推导校验）。其他目标需要新的契约 RC。所有目标拒绝都使用同一代码
rehome_destination_not_allowed，消息写明匹配到的身份类型（开发密钥、可伪造公钥、custody multisig/signatory、
validator 账户、validator session key 或其他账户），匹配到文档内身份时 related_path 指向它。
来源（`from`）不得是 custody multisig/signatory、validator 账户、session key、mining pool 或 AMM 账户，代码
rehome_source_not_allowed（CHOICE：单独代码，因为 pool 与 AMM 是合法目标但绝不是合法来源）。
topUps、reserveRefunds、rewardCredits 的接收方都由源数据精确推导（第 4 节），不是自由字段，因此不另作检查。
