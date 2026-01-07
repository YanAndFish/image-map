#!/usr/bin/env node

import process from 'node:process'

import { main } from '../dist/cli.mjs'

try {
  // eslint-disable-next-line antfu/no-top-level-await
  await main(process.argv.slice(2))
}
catch (err) {
  const message = err instanceof Error
    ? (err.stack ?? err.message)
    : String(err)
  process.stderr.write(`${message}\n`)
  process.exit(1)
}
