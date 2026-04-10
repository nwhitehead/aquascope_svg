import * as esbuild from 'esbuild';

await esbuild.build({
    entryPoints: ['./ts/kaya.ts'],
    bundle: true,
    format: 'esm',
    minify: true,
    sourcemap: true,
    outdir: './dist',
    banner: {
        js: '// hithere',
    },
});
