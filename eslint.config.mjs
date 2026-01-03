import antfu from '@antfu/eslint-config'

// eslint-disable-next-line node/prefer-global/process
const isInEditor = !!(process.env.VSCODE_PID || process.env.JETBRAINS_IDE) && !process.env.CI

export default antfu({
  rules: {},
  pnpm: false,
}, {
  ignores: ['.cursor/**', 'src-native/**'],
})
