import init, { parse, parse_partial, draw_program_png } from '../pkg/kaya_ts.js';

export type Options = {
    startOnLoad?: boolean,
    showPartial?: boolean,
    showErrors?: boolean,
    verbose?: boolean;
    scale?: number,
    theme?: string,
    querySelector?: string,
    nodes?: [HTMLElement],
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

const defaultOptions: Options = {
    startOnLoad: true,
    showPartial: true,
    showErrors: true,
    verbose: false,
    scale: 1.0,
    theme: 'dark',
    querySelector: '.kaya',
};

function computeOptions(opts: Options | null): Options {
    let options = {};
    Object.assign(options, defaultOptions);
    if (opts !== null) {
        Object.assign(options, opts);
    }
    return options;
}

let loadedOpts = computeOptions(null);
async function handleLoaded() {
    await init();
    await run(loadedOpts);
}

export async function initialize(opts: Options | null) {
    const options = computeOptions(opts);
    if (options.startOnLoad === false) {
        document.removeEventListener("DOMContentLoaded", handleLoaded);
    }
    loadedOpts = options;
}

document.addEventListener("DOMContentLoaded", handleLoaded);

export async function run(opts: Options | null) {
    const options = computeOptions(opts);
    await init();
    const nodes = (options.nodes === undefined) ? document.querySelectorAll(options.querySelector || '.kaya') : options.nodes;
    for (const elem of nodes) {
        if (options.verbose) {
            console.log('KAYA: Rendering', elem, options);
        }
        const src = elem.innerHTML;
        const response = await render(src, options);
        if (response.error) {
            if (options.showErrors) {
                if (options.verbose) {
                    console.log('KAYA: Error', response.error.msg);
                }
                const err = document.createElement('pre');
                err.innerHTML = response.error.msg;
                err.classList.add("error");
                elem.replaceWith(err);
                // There might also be partial image to show
                const img = document.createElement('img');
                img.src = response.imgUri || '';
                err.after(img);
                return;
            }
            // If we are not showing errors on page, at least log it to console
            console.log('KAYA: Error', response.error.msg);
            // Keep going below
        }
        // Create img and replace <pre> element with img
        const img = document.createElement('img');
        img.src = response.imgUri || '';
        img.classList.add('kaya-rendered');
        elem.replaceWith(img);
    }
}

/// Given Uint8Array with PNG data, construct a Data URI for IMG tags to use for it
function createDataURI(data: Uint8Array) {
    return URL.createObjectURL(new Blob([data], { type: 'image/png' }));
}

export async function render(src: string, opts: Options | null) {
    const options = computeOptions(opts);
    await init();
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

export default {
    initialize, run, render,
};
