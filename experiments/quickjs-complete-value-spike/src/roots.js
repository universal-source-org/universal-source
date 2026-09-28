// Trusted pristine setup only. Never invoked on a source result or after source runs.
// Retain identities, not source-visible markers. Bounded, descriptor-only traversal.
(() => {
    // Iterator and specialized-function shared prototypes are not all reachable
    // through global own properties. Pristine witnesses seed those intrinsic edges.
    const seeds = [globalThis, () => {}, async () => {}, function* () {},
        async function* () {}, [][Symbol.iterator](), ''[Symbol.iterator](),
        new Map().entries(), new Set().values(), ''.matchAll(/x/g),
        (function* () {})(), (async function* () {})()];
    const seen = new Set(), pending = seeds.map(value => [value, 0]), roots = [];
    while (pending.length) {
        const [value, depth] = pending.shift();
        if (value === null || (typeof value !== 'object' && typeof value !== 'function') || seen.has(value)) continue;
        if (depth > 16 || seen.size >= 4096) throw new Error('intrinsic root bound');
        seen.add(value); roots.push(value);
        pending.push([Object.getPrototypeOf(value), depth + 1]);
        for (const key of Reflect.ownKeys(value)) {
            const d = Object.getOwnPropertyDescriptor(value, key);
            for (const field of ['value', 'get', 'set']) {
                if (Object.prototype.hasOwnProperty.call(d, field)) pending.push([d[field], depth + 1]);
            }
        }
    }
    return roots;
})()
