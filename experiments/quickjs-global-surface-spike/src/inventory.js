(() => {
    const roots = {globalThis};
    for (const k of Object.getOwnPropertyNames(globalThis)) {
        const v = Object.getOwnPropertyDescriptor(globalThis,k).value;
        if (v && (typeof v === 'object' || typeof v === 'function')) {
            roots[k] = v;
            if (v.prototype) roots[k+'.prototype'] = v.prototype;
        }
    }
    roots.IteratorPrototype = Object.getPrototypeOf(Object.getPrototypeOf([][Symbol.iterator]()));
    roots.ArrayIteratorPrototype = Object.getPrototypeOf([][Symbol.iterator]());
    roots.StringIteratorPrototype = Object.getPrototypeOf(''[Symbol.iterator]());
    roots.MapIteratorPrototype = Object.getPrototypeOf(new Map().entries());
    roots.SetIteratorPrototype = Object.getPrototypeOf(new Set().values());
    roots.RegExpStringIteratorPrototype = Object.getPrototypeOf(''.matchAll(/a/g));
    roots.AsyncFunction = (async function(){}).constructor;
    roots.GeneratorFunction = (function*(){}).constructor;
    roots.AsyncGeneratorFunction = (async function*(){}).constructor;
    roots.GeneratorPrototype = Object.getPrototypeOf(Object.getPrototypeOf((function*(){})()));
    roots.AsyncGeneratorPrototype = Object.getPrototypeOf(Object.getPrototypeOf((async function*(){})()));
    roots.AsyncIteratorPrototype = Object.getPrototypeOf(roots.AsyncGeneratorPrototype);
    for (const name of ['AsyncFunction','GeneratorFunction','AsyncGeneratorFunction']) {
        roots[name+'.prototype'] = roots[name].prototype;
    }
    roots.ArrayUnscopables = Array.prototype[Symbol.unscopables];
    const out = {};
    for (const [name,v] of Object.entries(roots)) {
        out[name] = Reflect.ownKeys(v).map(k => {
            const d = Object.getOwnPropertyDescriptor(v,k);
            return [String(k), 'value' in d ? typeof d.value : 'accessor',
                d.writable ?? null, d.enumerable, d.configurable];
        });
    }
    return JSON.stringify(out);
})()
