# 公共 HTTPS APT（GitHub Pages）

> 状态：2026-10-20。工作流和本地验证已完成；部署需要仓库配置签名 secret 并启用 Pages。

## 目标

把签名 APT 仓库发布到：

```
https://allenzhang45.github.io/rpci/
```

客户端：

```bash
sudo curl -fsSL \
  https://allenzhang45.github.io/rpci/rpci-ime-archive-keyring.gpg \
  -o /usr/share/keyrings/rpci-ime-archive-keyring.gpg

echo 'deb [signed-by=/usr/share/keyrings/rpci-ime-archive-keyring.gpg] https://allenzhang45.github.io/rpci stable main' \
  | sudo tee /etc/apt/sources.list.d/rpci-ime.list

sudo apt update
sudo apt install rpci-ime
```

## 工作流

文件：[.github/workflows/apt-pages.yml](../.github/workflows/apt-pages.yml)

触发：

- GitHub Release `published`
- 手动 `workflow_dispatch`，输入 tag（默认 `v0.1.1`）

流程：

1. 下载对应 Release 的 `rpci-ime_<version>_amd64.deb`
2. 从 `APT_GPG_PRIVATE_KEY` 导入签名密钥
3. 用 `build-repo.sh` 构建 `site/` 签名仓库
4. 复制安装页和 `.nojekyll`
5. 用 `gpgv` 验证 `InRelease`
6. 上传并部署 GitHub Pages

## 必须配置

### 1. 签名密钥

本地已生成一套专用密钥（私钥被 `.gitignore` 的 `/dist/` 忽略）：

```
dist/apt-signing/private.asc   # 不要提交/分享
dist/apt-signing/public.asc
key id: 60E2CBC13ED5EF3A
```

在 GitHub 仓库添加 Actions secrets：

```
APT_GPG_PRIVATE_KEY = dist/apt-signing/private.asc 的内容
APT_GPG_KEY_ID      = 60E2CBC13ED5EF3A
```

可以用 GitHub UI，或已登录的 `gh`：

```bash
gh secret set APT_GPG_PRIVATE_KEY < dist/apt-signing/private.asc
gh secret set APT_GPG_KEY_ID --body 60E2CBC13ED5EF3A
```

私钥不要贴进 issue、聊天、提交或 Pages 仓库。

### 2. GitHub Pages

仓库 **Settings → Pages → Build and deployment → Source** 选择 **GitHub Actions**。

然后运行 **Actions → APT Pages → Run workflow**，tag 填 `v0.1.1`；或重新发布一个 Release。

## 本地验证

已用同一套签名密钥、Release `.deb` 和 HTTP 静态服务验证：

```
apt-get update
Get:1 http://127.0.0.1:8766 stable InRelease
Get:2 http://127.0.0.1:8766 stable/main amd64 Packages
Candidate: 0.1.1
```

`gpgv` 验证通过，`index.html` 和公钥可访问。

## 限制

- Pages 部署需要上面的 secret 和 Pages 设置，当前环境无法代用户配置 GitHub secrets。
- Pages 使用 HTTPS；APT 客户端用 `signed-by` 校验仓库元数据。
- 私钥泄露后必须更换密钥并更新所有客户端的公钥。
- 目前只发布 amd64。
