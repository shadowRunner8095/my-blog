import {  cp, readFile, writeFile } from 'node:fs/promises'
import { compile, optimize } from "@tailwindcss/node"
import { Scanner } from '@tailwindcss/oxide'
import { join } from 'node:path'
import { build } from 'esbuild'

const baseDir = process.cwd()

async function compileJS(){
  return await build({
    bundle: true,
    entryPoints: [join(baseDir, 'shared.ts')],
    minify: true,
    outdir: join(baseDir, 'dist'),
    format: 'esm',
  })
}

async function compileCss() {
  const cssConfig = await readFile(join(baseDir, "main.css"), {
    encoding: 'utf-8',
  })
  const scanner = new Scanner({
    sources: [{
      base: join(baseDir, 'dist'),
      negated: false,
      pattern: 'candidates.txt'
    }],
  })



  const { build } = await compile(cssConfig, {
    onDependency: console.log,
    base: baseDir
  })

  const candidates = scanner.scan();

  const { code: cssCompiled } = optimize(build(candidates), {
    minify: true
  })

  return writeFile(join(baseDir, 'dist', 'main.css'), cssCompiled)
}

await Promise.all([,
  cp(join(baseDir, 'assets'), join(baseDir, 'dist', 'assets'), {
    recursive: true
  }),
  compileJS()
])

await compileCss()