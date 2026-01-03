import { defineConfig } from 'bumpp'

// 统一发布配置：递归更新子包版本，创建提交与 tag，但默认不推送远端。
export default defineConfig({
  recursive: true,
  commit: true,
  tag: true,
  push: false,
})
