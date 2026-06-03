# FastLink 发布指南

**日期**: 2026-06-02  
**版本**: v26.5-20260603  
**状�?*: 准备发布

---

## 📦 发布前检查清�?
### �?已完成整�?
- [x] 更新 `.gitignore` - 添加全面忽略规则
- [x] 移除 `FastLinkMC` - 作为独立子项目排�?- [x] 清理临时文件 - 无日�?备份文件被跟�?- [x] 验证构建 - `cargo build --release` 成功
- [x] 提交所有更�?- main 分支已准备就�?
---

## 🚀 手动发布�?GitHub

### 方法 1: 通过 GitHub 网站创建 (推荐)

1. **访问 GitHub**
   - 打开 https://github.com
   - 登录账号 StarsailsClover

2. **创建新仓�?*
   - 点击右上�?"+" �?"New repository"
   - Repository name: `FastLink`
   - Description: "High-performance P2P networking protocol suite"
   - 选择 "Public"
   - 勾�?"Add a README" (可�?
   - 点击 "Create repository"

3. **推送本地代�?*

```bash
# 在本�?FastLink 目录中执�?cd C:\Users\Sails\Documents\Workspace\NormalWorkspace\Coding\FastLink

# 添加新的远程仓库地址
git remote remove origin
git remote add origin https://github.com/StarsailsClover/FastLink.git

# 推�?main 分支
git push -u origin main

# 推送标�?git push origin v26.5-20260603
git push origin v26.5-20260531
```

---

### 方法 2: 使用 GitHub CLI (如果已登�?

```bash
# 登录 GitHub CLI
gh auth login

# 创建仓库并推�?cd C:\Users\Sails\Documents\Workspace\NormalWorkspace\Coding\FastLink
gh repo create StarsailsClover/FastLink --public --source=. --remote=origin --push
```

---

## 🏷�?创建 GitHub Release

### 步骤

1. **访问仓库页面**
   - https://github.com/StarsailsClover/FastLink

2. **创建 Release**
   - 点击 "Releases" �?"Create a new release"
   - 选择标签: `v26.5-20260603`
   - Title: "FastLink v26.5-20260603 Pre-Release"

3. **填写发布说明**

```markdown
## FastLink v26.5-20260603 Pre-Release 🚀

### Highlights
- �?All 7 core libraries implemented
- �?All 6 sub-protocols functional  
- �?CLI application ready
- �?Full build passing
- �?Test compilation fixed
- �?Comprehensive documentation

### Changes since v0.1.0
- Added rustls workspace dependencies
- Fixed test compilation errors
- Added AUDIT_REPORT.md and DEVELOPMENT_GUIDE.md
- Updated version to 0.2.0-alpha
- Updated .gitignore with comprehensive rules

### Installation
```bash
git clone https://github.com/StarsailsClover/FastLink.git
cd FastLink
git checkout v26.5-20260603
cargo build --release
```

### Known Issues
- Some test logic needs refinement
- Minor compiler warnings for unused fields

### Documentation
- [AUDIT_REPORT.md](AUDIT_REPORT.md) - Project audit
- [DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md) - Development guide
- [PRE_RELEASE_v26.5-20260603.md](PRE_RELEASE_v26.5-20260603.md) - Release notes

### Status
Ready for testing and development
```

4. **上传构建产物** (可�?
   - �?`target/release/fastlink-cli.exe` 上传�?Release

5. **发布设置**
   - 勾�?"This is a pre-release"
   - 点击 "Publish release"

---

## 📋 发布检查清�?
发布前确�?
- [ ] GitHub 仓库已创�?- [ ] 代码已推送到 main 分支
- [ ] 标签 v26.5-20260603 已推�?- [ ] GitHub Release 已创�?- [ ] Pre-release 标记已勾�?- [ ] 文档链接有效

---

## 🔒 安全建议

### 确保未上传敏感信�?- [x] �?`.env` 文件
- [x] 无密钥文�?(`.pem`, `.key`, `.cert`)
- [x] 无日志文�?(`.log`)
- [x] 无个人配置文�?- [x] �?IDE 特定文件 (`.vscode/`, `.idea/`)
- [x] 无构建产�?(`target/`)

### 已保护的文件
通过 `.gitignore` 已排�?
- 所�?`target/` 目录
- 所�?`Cargo.lock` (库项�?
- IDE 配置文件
- 操作系统文件
- 环境文件
- 日志文件

---

## 📊 仓库信息

| 项目 | 详情 |
|------|------|
| **名称** | FastLink |
| **描述** | High-performance P2P networking protocol suite |
| **语言** | Rust |
| **许可�?* | MIT OR Apache-2.0 |
| **版本** | v26.5-20260603 |
| **分支** | main (默认) |

---

## 🆘 故障排除

### 问题: 推送被拒绝

**解决**:
```bash
# 强制推�?(如果确定覆盖远程)
git push --force-with-lease origin main
```

### 问题: 认证失败

**解决**:
```bash
# 使用 GitHub Token
git remote set-url origin https://<TOKEN>@github.com/StarsailsClover/FastLink.git
```

### 问题: 文件太大

**解决**:
```bash
# 检查大文件
git rev-list --objects --all | git cat-file --batch-check='%(objecttype) %(objectname) %(objectsize) %(rest)' | awk '/^blob/ {print $3, $4}' | sort -rn | head -20
```

---

## �?发布成功�?
发布完成�?
1. 验证仓库主页显示正常
2. 检�?Release 页面可访�?3. 测试克隆命令:
   ```bash
   git clone https://github.com/StarsailsClover/FastLink.git
   ```
4. 分享发布链接

---

**发布�?*: SailsClover  
**协助**: 小跃 (StepFun AI)  
**日期**: 2026-06-02
