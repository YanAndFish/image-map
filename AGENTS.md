# Repository Guidelines

## 项目结构与模块组织
- `src/`：TypeScript 代码目录（当前为空）。
- `src-native/`：Rust crate（`src-native/src/main.rs`）。
- 主要配置：`eslint.config.mjs`、`tsconfig.json`、`pnpm-lock.yaml`。

## 构建、测试与开发命令
- `pnpm install`：安装依赖（仅允许 pnpm）。
- `pnpm lint` / `pnpm lint:fix`：ESLint 检查与修复。
- `pnpm tsc`：类型检查（noEmit）。
- `pnpm update-deps`：更新依赖并在 `src-native` 执行 `cargo update`。
- Rust：`cd src-native && cargo build/run/test`。

## 编码规范与命名约定
- TS/JS 遵循 ESLint（@antfu）；格式问题优先 `pnpm lint:fix`。
- Rust 使用 `rustfmt`（`tab_spaces=2`，执行 `cargo fmt`）。
- 命名遵循语言惯例；新增函数/类型需简洁注释，结构体/接口成员逐一注释。

## 测试指南
- 当前未配置测试框架或测试目录；新增测试需补充脚本并在 PR 说明运行方式。

## 提交与 Pull Request 指南
- 提交信息沿用 Conventional Commits（示例：`chore: init`）。
- PR 需说明变更目的、影响范围、验证方式（如 `pnpm lint`、`pnpm tsc`、`cargo test`）。