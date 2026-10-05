# Agent Note：本地镜像的更新器 e2e——网络无关的验收通道（T-098）

Status: proposed

[English](2026-10-05-updater-e2e-sandbox.md) | 中文

## 问题

§17.2（更新源线上五项 e2e：check / apply / 篡改拒绝 / 回滚 / 一致性）通常需要 GitHub
真实网络与真实 feed。2026-10-05 本机到 GitHub 通道抖动（连接重置、21s 连接超时、
连续 5/5 失败），验收 VM 又不可达（旧 IP 与网段内唯一 SMB 主机均拒绝）。但机制级验收
无论如何必须完成，且不能触碰用户真实配置。

## 方案

采用本地镜像演练作为标准的"网络无关"更新器 e2e 通道：

1. **隔离用户配置文件**：把 `APPDATA`/`LOCALAPPDATA` 指到 `target/release-e2e/` 下的
   临时目录，写最小 `config.json`（`{"online_update":true}`），不碰真实 `%APPDATA%`。
2. **本地 feed 镜像**：用微型 `HttpListener` 在 127.0.0.1 提供发布快照
   （`target/release-assets/<ver>/packs/`）；`ZHU_YE_MANIFEST_URL` 指向本地使其走与
   真实通道完全相同的代码路径（覆盖机制本就是文档化特性，§17.2「覆盖机制保留」）。
3. **篡改+回滚断言**（2026-10-05 全 8 项 PASS）：
   - 镜像中包被篡改 → `apply` 非零退出，报「包内容校验失败: 包 it 内容哈希不一致」，
     `packs/` 无任何落地；
   - 恢复镜像 → `apply` 成功，落地包哈希与 manifest 一致；
   - 人为破坏已装包 → `apply` 视作过期，先备份为 `<id>.zyct.bak` 再原子恢复原版哈希
     （`.bak` 保留损坏副本作证据——即回滚储备）。
4. **线上一致性**：GitHub 可用时 `curl -fsSL` 下载 feed 四个文件与本地快照逐一比对
   SHA-256（2026-10-05 4/4 一致：manifest 7b7752f6…、it 57bc745f…、med 489c1e76…、
   slang d1eb2c1a…）。
5. **仓库卫生**：`gh api` 查镜像仓库——`releases/latest` 显示 `"prerelease":false`
   （tag 含 `-alpha` 但 release 未标记 prerelease，`releases/latest` 语义仍成立）；
   `main` 分支保护通过 `gh api -X PUT …/branches/main/protection` 配置
   `required_pull_request_reviews.required_approving_review_count=1`
   （配置前 HTTP 404 证实未受保护）。

## 验收标准

- 演练全部发生在 `target/release-e2e/` 内，`APPDATA`/`LOCALAPPDATA` 已重定向
  （不写真实用户环境）。
- 篡改包 → 非零退出、明确的哈希不一致报错、`packs/` 无落地。
- 恢复镜像 → `apply` 成功，落地包哈希与 manifest 一致。
- 破坏已装包 → 重新 `apply` 恢复原版、`<id>.zyct.bak` 保留损坏副本、恢复原哈希。
- GitHub 可用时线上 4/4 SHA-256 比对通过；否则明确说明推迟到发布轮。
- 演练结束后 HTTP 服务器 job 已关闭。

## 备选方案

- **等 GitHub 通道稳定后直接跑真实网络 e2e**：否决——连续五次失败使等待无上界，
  且 §17.6 要求当场回填验收表而非整体延后。
- **用真实 manifest URL + 缓存包字节演练**：否决——篡改演练需要提供被改的字节，
  必须由可控端点承担；本地镜像是唯一确定性做法。
- **用更新器单测充当 e2e 证据**：不够——验收需要真实 CLI 二进制的端到端行为，
  包括落盘 staging 与原子 rename。

## 后果

- GitHub/CDN 或 VM 抖动时，更新器机制级 e2e 不得静默跳过——本地镜像演练就是回退
  通道；真实网络全量 `apply` 在 v0.1.2 正式发布时随 CI 复跑一次。
- `target/release-e2e/` 作为该演练的可重复运行之处（`http-server.ps1`、
  `tamper-e2e.ps1`）；按 `target/` 规则不入库。
- 更新器本身有两层防护（manifest 签名 + 逐包 SHA-256），演练因此通过；失败信息精确
  给出哈希，是未来验收记录的有用证据。

## 风险

- 本地镜像若与真实 feed 不一致会产生假 PASS；演练必须搭配线上 4/4 哈希比对
  （或明确说明线上核对推迟到发布轮）。
- HttpListener 仅监听本地端口；演练结束必须关掉服务器 job（托管后台任务），不得常驻。
