import * as esbuild from 'esbuild'

await esbuild.build({
  entryPoints: ['./ts/kaya.ts'],
  bundle: true,
  minify: true,
  sourcemap: true,
  outdir: './dist',
  outfile: 'kaya.js',
  banner: {
    ts: '// hithere',
  }
})
