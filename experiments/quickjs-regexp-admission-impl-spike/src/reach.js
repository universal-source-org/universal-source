// Trusted test-only witness, derived from the global-surface traversal. Forbidden
// native identities are captured before installation and held only by this host
// closure; the walk covers prototypes and every value/get/set descriptor edge.
(forbiddenPairs => {
    const keys = Reflect.ownKeys, desc = Object.getOwnPropertyDescriptor, proto = Object.getPrototypeOf;
    const forbidden = new Map(forbiddenPairs);
    return mustComplete => {
        const roots = [globalThis, /a/g, new RegExp('a', 'y'), RegExp('b'), class extends RegExp {},
            ''.matchAll('a'), ''.matchAll(/a/g), 'aa'.match('a'), 'a,b'.split(/,/), /a/[Symbol.split]('a'),
            new (class extends RegExp {})('c'), () => {}, async () => {}, function* () {},
            RegExp.prototype[Symbol.matchAll].call({ constructor: undefined, flags: 'g', lastIndex: 0,
                [Symbol.match]: true, source: 'a' }, 'aa')];
        for (const f of [() => new RegExp('['), () => 'a'.match('['), () => /a/.compile('['),
            () => RegExp.prototype.compile.call({}), () => RegExp.prototype[Symbol.split].call(1, ''),
            () => ''.matchAll(/a/), () => RegExp.prototype[Symbol.split].call({ constructor: 1 }, '')]) {
            try { f(); } catch (e) { roots.push(e); }
        }
        const queue = roots.map(v => [v, 0, 'root']), seen = new Set();
        let edges = 0, maxDepth = 0;
        for (let i = 0; i < queue.length; i++) {
            const [v, depth, path] = queue[i];
            if (!v || (typeof v !== 'object' && typeof v !== 'function') || seen.has(v)) continue;
            if (forbidden.has(v)) return 'forbidden:' + forbidden.get(v) + ':' + path;
            if (depth > 16 || seen.size >= 8192) throw new Error('traversal bound exceeded');
            seen.add(v); maxDepth = Math.max(maxDepth, depth);
            const add = (child, label) => { edges++; queue.push([child, depth + 1, path + '.' + label]); };
            add(proto(v), '[[Prototype]]');
            for (const k of keys(v)) {
                const d = desc(v, k);
                for (const edge of ['value', 'get', 'set']) if (edge in d) add(d[edge], String(k) + ':' + edge);
            }
        }
        if (!mustComplete) throw new Error('negative control did not find forbidden identity');
        return `clean:nodes=${seen.size},edges=${edges},depth=${maxDepth}`;
    };
})
