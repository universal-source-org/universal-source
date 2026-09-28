// Trusted, fixed host bootstrap. Runs once on a pristine realm before any package code.
// This is only dynamic-compilation hardening, not the complete RFC global profile.
(() => {
    "use strict";
    const define = Object.defineProperty;
    const setPrototype = Object.setPrototypeOf;
    const NativeProxy = Proxy;
    const NativeEvalError = EvalError;
    const original = [Function, (async function() {}).constructor,
        (function*() {}).constructor, (async function*() {}).constructor];
    // Null-prototype handler prevents later Object.prototype poisoning of absent traps.
    // Captured native error constructor avoids later mutation of the global binding.
    const handler = Object.create(null);
    handler.apply = () => { throw new NativeEvalError("dynamic compilation disabled"); };
    handler.construct = () => { throw new NativeEvalError("dynamic compilation disabled"); };
    Object.freeze(handler);
    const guarded = original.map(ctor => new NativeProxy(ctor, handler));
    // Specialized constructors inherit from Function itself, not Function.prototype.
    // Without replacing these parent links, getPrototypeOf would leak native Function.
    for (let i = 1; i < original.length; i++) setPrototype(original[i], guarded[0]);
    for (let i = 0; i < original.length; i++) {
        define(original[i].prototype, "constructor", {
            value: guarded[i], writable: false, configurable: false, enumerable: false
        });
    }
    define(globalThis, "Function", {
        value: guarded[0], writable: false, configurable: false, enumerable: false
    });
    define(globalThis, "eval", {
        value: new NativeProxy(eval, handler),
        writable: false, configurable: false, enumerable: false
    });
})();
