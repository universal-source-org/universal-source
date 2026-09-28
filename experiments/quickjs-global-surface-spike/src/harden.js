// Fixed trusted bootstrap, before package compilation/evaluation. Not production code.
// Arguments are private host-created policy data; no temporary global is installed.
((allowedGlobals, allowedIntrinsics) => {
    'use strict';
    const ownKeys = Reflect.ownKeys, descriptor = Object.getOwnPropertyDescriptor;
    const define = Object.defineProperty, proto = Object.getPrototypeOf;
    const TypeErrorOriginal = TypeError;
    const NativeProxy = Proxy;
    // Symbol.dispose/asyncDispose are nonconfigurable on the native constructor.
    // A bound native Symbol retains call/new semantics without those own properties.
    // Replace BOTH global and prototype constructor links, leaving no native alias.
    const originalSymbol = Symbol;
    const symbolFacade = originalSymbol.bind(undefined);
    for (const key of allowedIntrinsics.Symbol) {
        define(symbolFacade, key, descriptor(originalSymbol, key));
    }
    define(originalSymbol.prototype, 'constructor', {
        value: symbolFacade, writable: true, configurable: true, enumerable: false
    });
    globalThis.Symbol = symbolFacade;

    // Error families internally produced by the VM must not recover extension constructors.
    // Standard inherited Error properties remain; source cannot recover native InternalError.
    for (const ctor of [InternalError, SuppressedError]) {
        for (const key of ['name', 'message', 'constructor']) {
            if (!Reflect.deleteProperty(ctor.prototype, key)) throw new Error('error extension');
        }
    }
    const roots = {};
    for (const name of allowedGlobals) {
        const v = globalThis[name];
        if (v && (typeof v === 'function' || typeof v === 'object') && name !== 'globalThis') {
            roots[name] = v;
            if (v.prototype) roots[name+'.prototype'] = v.prototype;
        }
    }
    roots.IteratorPrototype = proto(proto([][Symbol.iterator]()));
    roots.ArrayIteratorPrototype = proto([][Symbol.iterator]());
    roots.StringIteratorPrototype = proto(''[Symbol.iterator]());
    roots.MapIteratorPrototype = proto(new Map().entries());
    roots.SetIteratorPrototype = proto(new Set().values());
    roots.RegExpStringIteratorPrototype = proto(''.matchAll(/a/g));
    roots.AsyncFunction = (async function(){}).constructor;
    roots.GeneratorFunction = (function*(){}).constructor;
    roots.AsyncGeneratorFunction = (async function*(){}).constructor;
    roots.GeneratorPrototype = proto(proto((function*(){})()));
    roots.AsyncGeneratorPrototype = proto(proto((async function*(){})()));
    roots.AsyncIteratorPrototype = proto(roots.AsyncGeneratorPrototype);
    for (const name of ['AsyncFunction','GeneratorFunction','AsyncGeneratorFunction']) {
        roots[name+'.prototype'] = roots[name].prototype;
    }
    roots.ArrayUnscopables = Array.prototype[Symbol.unscopables];
    // Hidden Proxy handlers keep native method name/length and hide bootstrap closures.
    const handler = Object.create(null);
    handler.apply = () => { throw new TypeErrorOriginal('ambient method disabled'); };
    Object.freeze(handler);
    const restricted = [[Math,'random'],[String.prototype,'localeCompare'],
        [String.prototype,'toLocaleLowerCase'],[String.prototype,'toLocaleUpperCase'],
        [Object.prototype,'toLocaleString'],[Array.prototype,'toLocaleString'],
        [Number.prototype,'toLocaleString'],[BigInt.prototype,'toLocaleString']];
    // Pinned BigInt inherits Object.prototype.toLocaleString; give it its required own guard.
    for (const [object,key] of restricted) {
        define(object,key,{value:new NativeProxy(object[key],handler),
            writable:false,configurable:false,enumerable:false});
    }
    // Explicit ES2023 key policy, independently reviewed against edition 14.
    // Unknown keys are removed; a separate pristine descriptor snapshot rejects engine drift.
    for (const [name, keys] of Object.entries(allowedIntrinsics)) {
        const object = roots[name];
        if (!object) throw new Error('missing intrinsic: '+name);
        for (const key of ownKeys(object)) {
            if (!keys.includes(String(key)) && !Reflect.deleteProperty(object,key)) {
                throw new Error('unremovable intrinsic: '+name+'.'+String(key));
            }
        }
        if (JSON.stringify(ownKeys(object).map(String).sort()) !== JSON.stringify([...keys].sort())) {
            throw new Error('intrinsic key mismatch: '+name);
        }
    }
    for (const key of ownKeys(globalThis)) {
        if (typeof key !== 'string' || !allowedGlobals.includes(key)) {
            if (!Reflect.deleteProperty(globalThis,key)) throw new Error('global deletion failed');
        }
    }
    if (JSON.stringify(ownKeys(globalThis).sort()) !== JSON.stringify([...allowedGlobals].sort())) {
        throw new Error('global allowlist mismatch');
    }
})
