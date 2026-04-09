import init, { parse, parse_partial, draw_program_png } from '../pkg/kaya_ts.js';

export type Options = {
    startOnLoad?: boolean,
    showPartial?: boolean,
    scale?: number,
    theme?: string,
}

export type ErrorInformation = {
    msg: string,
    row: number,
    col: number,
}

export type Response = {
    error: null | ErrorInformation,
    imgUri: null | string;
}

let ready: boolean = false;
let options: Options = {
    startOnLoad: true,
    showPartial: true,
    scale: 1.0,
    theme: 'dark',
};

export function setOptions(opts: Options) {
    Object.assign(options, opts);
}

export async function initIfNeeded() {
    if (!ready) {
        // init wasm (only need to do this once)
        await init();
        ready = true;
    }
}

export async function initialize(opts: Options | null) {
    if (opts !== null) {
        Object.assign(options, opts);
    }
    console.log('hi from initialize');
    await initIfNeeded();
}

/// Given Uint8Array with PNG data, construct a Data URI for IMG tags to use for it
function createDataURI(data: Uint8Array) {
    return URL.createObjectURL(new Blob([data], { type: 'image/png' }));
}

export async function render(src: string) {
    await initIfNeeded();
    // Try to parse, if it fails then try with partial parse
    let error = null;
    let res = parse(src);
    let prg = null;
    if (res.Success !== undefined) {
        prg = res.Success;
    } else {
        error = {
            msg: res.Error[0],
            row: res.Error[1][0],
            col: res.Error[1][1],
        };
        if (!options.showPartial) {
            return { error, imgUri: null };
        }
        let res2 = parse_partial(src);
        if (res2.Success === undefined) {
            // if we get here something went wrong in partial parse
            error = {
                msg: res2.Error[0],
                row: res2.Error[1][0],
                col: res2.Error[1][1],
            };
            return { error, imgUri: null };
        }
        // Use partial parse for rendering
        prg = res2.Success;
    }
    const pngData = draw_program_png(prg, options.scale || 1.0, options.theme || "dark");
    const imgUri = createDataURI(pngData);
    return { error, imgUri };
}
