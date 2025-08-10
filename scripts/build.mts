import { copyFile, cp, readFile, writeFile } from 'node:fs/promises'
import { compile, optimize } from "@tailwindcss/node"
import { Scanner } from '@tailwindcss/oxide'
import { join } from 'node:path'

const baseDir = process.cwd()

async function compileCss() {
  const cssConfig = await readFile(join(baseDir, "main.css"), {
    encoding: 'utf-8'
  })
  const scanner = new Scanner({
    sources: [{
      base: baseDir,
      negated: false,
      pattern: 'index.html'
    }]
  })



  const { build } = await compile(cssConfig, {
    onDependency: console.log,
    base: baseDir
  })

  const { code: cssCompiled } = optimize(build(scanner.scan()), {
    minify: true
  })

  return writeFile(join(baseDir, 'dist', 'main.css'), cssCompiled)
}

await Promise.all([
  compileCss(),
  cp(join(baseDir, 'index.html'), join(baseDir, 'dist', 'index.html')),
  cp(join(baseDir, 'assets'), join(baseDir, 'dist', 'assets'), {
    recursive: true
  })
])
