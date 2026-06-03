# FastLink 发布指南

**日期**: 2026-06-02  
**版本**: v0.2.0-alpha  
**状态**: 准备发布

---

## 📦 发布前检查清单

### ✅ 已完成整理

- [x] 更新 `.gitignore` - 添加全面忽略规则
- [x] 移除 `FastLinkMC` - 作为独立子项目排除
- [x] 清理临时文件 - 无日志/备份文件被跟踪
- [x] 验证构建 - `cargo build --release` 成功
- [x] 提交所有更改 - main 分支已准备就绪

---

## 🚀 手动发布到 GitHub

### 方法 1: 通过 GitHub 网站创建 (推荐)

1. **访问 GitHub**
   - 打开 https://github.com
   - 登录账号 StarsailsClover

2. **创建新仓库**
   - 点击右上角 "+" → "New repository"
   - Repository name: `FastLink`
   - Description: "High-performance P2P networking protocol suite"
   - 选择 "Public"
   - 勾选 "Add a README" (可选)
   - 点击 "Create repository"

3. **推送本地代码**

```bash
# 在本地 FastLink 目录中执行
cd C:\Users\Sails\Documents\Workspace\NormalWorkspace\Coding\FastLink

# 添加新的远程仓库地址
git remote remove origin
git remote add origin https://github.com/StarsailsClover/FastLink.git

# 推送 main 分支
git push -u origin main

# 推送标签
git push origin v0.2.0-alpha
git push origin v26.5-20260531
```

---

### 方法 2: 使用 GitHub CLI (如果已登录)

```bash
# 登录 GitHub CLI
gh auth login

# 创建仓库并推送
cd C:\Users\Sails\Documents\Workspace\NormalWorkspace\Coding\FastLink
gh repo create StarsailsClover/FastLink --public --source=. --remote=origin --push
```

---

## 🏷️ 创建 GitHub Release

### 步骤

1. **访问仓库页面**
   - https://github.com/StarsailsClover/FastLink

2. **创建 Release**
   - 点击 "Releases" → "Create a new release"
   - 选择标签: `v0.2.0-alpha`
   - Title: "FastLink v0.2.0-alpha Pre-Release"

3. **填写发布说明**

```markdown
## FastLink v0.2.0-alpha Pre-Release 🚀

### Highlights
- ✅ All 7 core libraries implemented
- ✅ All 6 sub-protocols functional  
- ✅ CLI application ready
- ✅ Full build passing
- ✅ Test compilation fixed
- ✅ Comprehensive documentation

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
git checkout v0.2.0-alpha
cargo build --release
```

### Known Issues
- Some test logic needs refinement
- Minor compiler warnings for unused fields

### Documentation
- [AUDIT_REPORT.md](AUDIT_REPORT.md) - Project audit
- [DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md) - Development guide
- [PRE_RELEASE_v0.2.0-alpha.md](PRE_RELEASE_v0.2.0-alpha.md) - Release notes

### Status
Ready for testing and development
```

4. **上传构建产物** (可选)
   - 将 `target/release/fastlink-cli.exe` 上传到 Release

5. **发布设置**
   - 勾选 "This is a pre-release"
   - 点击 "Publish release"

---

## 📋 发布检查清单

发布前确认:
- [ ] GitHub 仓库已创建
- [ ] 代码已推送到 main 分支
- [ ] 标签 v0.2.0-alpha 已推送
- [ ] GitHub Release 已创建
- [ ] Pre-release 标记已勾选
- [ ] 文档链接有效

---

## 🔒 安全建议

### 确保未上传敏感信息
- [x] 无 `.env` 文件
- [x] 无密钥文件 (`.pem`, `.key`, `.cert`)
- [x] 无日志文件 (`.log`)
- [x] 无个人配置文件
- [x] 无 IDE 特定文件 (`.vscode/`, `.idea/`)
- [x] 无构建产物 (`target/`)

### 已保护的文件
通过 `.gitignore` 已排除:
- 所有 `target/` 目录
- 所有 `Cargo.lock` (库项目)
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
| **许可证** | MIT OR Apache-2.0 |
| **版本** | v0.2.0-alpha |
| **分支** | main (默认) |

---

## 🆘 故障排除

### 问题: 推送被拒绝

**解决**:
```bash
# 强制推送 (如果确定覆盖远程)
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

## ✅ 发布成功后

发布完成后:
1. 验证仓库主页显示正常
2. 检查 Release 页面可访问
3. 测试克隆命令:
   ```bash
   git clone https://github.com/StarsailsClover/FastLink.git
   ```
4. 分享发布链接

---

**发布者**: SailsClover  
**协助**: 小跃 (StepFun AI)  
**日期**: 2026-06-02
