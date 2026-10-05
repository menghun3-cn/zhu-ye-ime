# Agent Note：更新源镜像 feed 上线 —— zhu-ye-updates（第十二期，T-098）

状态：proposed（待确认）

[English](2026-10-05-update-mirror-feed-live.md) | 中文

## 问题

已交付客户端默认 manifest URL 指向主仓库 `releases/latest`，而只要版本带 prerelease
标记该 URL 就永久 404（见 `2026-10-05-release-closure-batch-one.md` §问题 2，2026-10-05
实证）。第十二期推荐口径（D-76）为独立镜像 feed，使更新源不再依赖"只发 prerelease 仓库"
的 latest 语义。T-098 执行该决策。

## 方案（执行口径）

1. **feed 仓库 `menghun3-cn/zhu-ye-updates`**：release 工作流（或手动发布）把
   `manifest.json + it/med/slang.zyct` 作为 GitHub Release 推送到该仓库。
2. **feed release 一律不加 `--prerelease` 标记**，因此 `releases/latest` 解析到最新
   feed。本条关键认知：*prerelease 是 release 上的显式标记，与 tag 形态无关*——
   `feed-v0.1.2-alpha` 这类 tag 只要不传标记就仍是普通 release。此前"tag 含 `-` 即
   prerelease"的假设不成立；带 `--prerelease` 只是 `release.yml` 自己的约定，且该约定
   **不应用于 feed release**。
3. **URL 结构无需改客户端逻辑**：`releases/latest/download/<资产名>` 在最新 release
   下可服务任意资产；更新器用 manifest URL 目录推出 `base_url`，故
   `https://github.com/menghun3-cn/zhu-ye-updates/releases/latest/download/manifest.json`
   → `.../download/it.zyct` 等天然可用。
4. **`DEFAULT_MANIFEST_URL` 切换到镜像**（`crates/zhu-ye-updater/src/main.rs`）；
   保留 `ZHU_YE_MANIFEST_URL` 覆盖（测试/私有渠道）。
5. **CI 自动化（release.yml publish-updates 步骤）**：`GITHUB_TOKEN` 不能推其他仓库，
   故用 `ZHU_YE_UPDATES_TOKEN` secret（细粒度 PAT，zhu-ye-updates 的
   contents:write）。secret 缺失时步骤告警并跳过——试跑/未配 PAT 阶段不阻塞主发布。
   feed release `--target` 固定到发布 commit，保证可追溯。

## 备选方案

- **feed 仓库 Pages URL**：放弃——多一步部署与最终一致等待；Releases-`latest` 由资产
  CDN 即时服务。
- **保留主仓库 latest URL**：放弃——已实证 404（batch-one note）。
- **feed release 带 prerelease 标记**：放弃——会在 feed 仓库重演 latest 缺陷。
- **改更新器接受 manifest 里的 URL**：放弃——无必要改 manifest schema，`base_url`
  推导已覆盖。

## 验证（2026-10-05，主机）

- 创建 `menghun3-cn/zhu-ye-updates`；手动发布 feed `feed-v0.1.2-alpha`
  （manifest + it/med/slang.zyct），`--target d8685a4…`（镜像 main）。
- `curl`：`releases/latest/download/manifest.json` → **200**（1108 B，schema=1，
  3 包；主仓库此前 302→404）；`releases/latest/download/it.zyct` → **200**，
  956,228 B 与 manifest 记录一致。
- 注入公钥构建的更新器：`check` 从镜像拉取 manifest，**内置公钥验签通过**，
  差异 = it/med/slang。
- 默认 `online_update:false` 保持零网络（探测后已还原 config）。

## 后果

- 信任链不变——只是 URL 移动；私钥仍只存在于 Actions secret。
- feed 仓库卫生：杜绝带 prerelease 标记的 release、禁止密钥材料、保护分支。
- 配置 `ZHU_YE_UPDATES_TOKEN` 前 feed 走手动发布（如上）；配置后 release 工作流
  每次发布自动推送。
- `verify-release-e2e.ps1 -Live` 模式与 VM 验收（apply / `.bak` 回滚 / 篡改拒绝 /
  feed 哈希与 Release 资产一致）留给验收阶段（§17.2）。

## Acceptance criteria

> 对应验收标准 §17.2（更新源）。

- 默认 URL 拉取 manifest 不再 404 —— **已完成**（上文实证）。
- `apply` 验签通过、原子落地、`.bak` 回滚有效；篡改包被拒；feed 哈希与 Release 资产
  一致 —— 待线上/VM e2e。
- `online_update:false` 零网络 —— **已完成**（默认检查）。
- `ZHU_YE_MANIFEST_URL` 覆盖保留 —— 覆盖路径代码未变。

## 风险

- **CDN 瞬时重置**（首次探测曾出现 curl exit 56，立即重试成功）。缓解：更新器
  `--max-time` 限时 + 线上 e2e 有界重试。
- **PAT 缺失/轮换** → 手动发布兜底；步骤已设计为告警+跳过。
- **重发时 feed tag 冲突** —— `gh release create` 失败忽略 + `upload --clobber`
  覆盖资产。

## 取代关系

- 部分执行 `2026-10-05-release-closure-batch-one.md` 的 D-76（"M15-D 时定终选"）：
  M15-D 现定案为 Releases-`latest` + feed release 显式非 prerelease。batch-one note
  保持 active（范围更广：exe/Pages/文档）；交叉引用。
