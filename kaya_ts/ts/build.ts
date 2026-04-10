import * as esbuild from 'esbuild';
import { copy } from 'esbuild-plugin-copy';

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
    plugins: [
        copy({
            assets: {
                from: ['./pkg/kaya_ts_bg.wasm'],
                to: ['.'],
            },
        }),
    ],
});
