// Trusted setup only; not production code. Evaluated on a pristine realm before
// hardening to capture the native RegExp surface. Returns an installer that runs
// after hardening and before any package source. No package receives the
// installer, the host functions or any captured native.
(() => {
    'use strict';
    const NativeRegExp = RegExp, RegExpPrototype = RegExp.prototype, NativeProxy = Proxy;
    const NativeError = Error, NativeTypeError = TypeError;
    const apply = Reflect.apply, construct = Reflect.construct;
    const setPrototypeOf = Reflect.setPrototypeOf, defineProperty = Reflect.defineProperty;
    const getOwnPropertyDescriptor = Reflect.getOwnPropertyDescriptor;
    const freeze = Object.freeze, is = Object.is, trunc = Math.trunc;
    const symMatch = Symbol.match, symMatchAll = Symbol.matchAll, symSearch = Symbol.search;
    const symSplit = Symbol.split, symSpecies = Symbol.species;
    const StringPrototype = String.prototype, stringIndexOf = StringPrototype.indexOf;
    const nativeCompile = RegExpPrototype.compile;
    const nativeSplit = RegExpPrototype[symSplit], nativeMatchAll = RegExpPrototype[symMatchAll];
    const nativeMatch = StringPrototype.match, nativeStringMatchAll = StringPrototype.matchAll;
    const nativeSearch = StringPrototype.search;

    return (checkpoint, admit, readmit, brand, isConstructor, markUncatchable) => {
        const stop = new NativeError('host stop');
        markUncatchable(stop);

        // Every source-visible entry checks the trusted deadline/cancel/latch state first.
        const enter = () => { if (!checkpoint()) throw stop; };
        // A missing index would otherwise read Array.prototype.
        const arg = (args, i) => (i < args.length ? args[i] : undefined);
        const isObject = v => v !== null && (typeof v === 'object' || typeof v === 'function');
        const toStr = v => `${v}`;
        const contains = (s, c) => apply(stringIndexOf, s, [c]) >= 0;
        const fail = message => { throw new NativeTypeError(message); };
        const isRegExp = v => {
            if (!isObject(v)) return false;
            const matcher = v[symMatch];
            if (matcher !== undefined) return !!matcher;
            return brand(v);
        };
        const toLength = v => {
            const n = +v;
            if (!(n > 0)) return 0;
            return n > 9007199254740991 ? 9007199254740991 : trunc(n);
        };

        // Native allocation from primitive strings: its IsRegExp/ToString observe nothing.
        const compileAdmitted = (P, F, proto) => {
            if (!admit(P.length)) throw stop;
            const obj = construct(NativeRegExp, [P, F], NativeRegExp);
            if (proto !== RegExpPrototype) setPrototypeOf(obj, proto);
            return obj;
        };
        // Copies [[OriginalSource]]/[[OriginalFlags]] and bytecode; no Get, no compile.
        const cloneGenuine = pattern => {
            const copy = construct(NativeRegExp, [''], NativeRegExp);
            apply(nativeCompile, copy, [pattern]);
            return copy;
        };

        // ES2023 22.2.4.1 with one coercion of each input, in specification order.
        const regExpFrom = (pattern, flags, newTarget, callForm) => {
            enter();
            const patternIsRegExp = isRegExp(pattern);
            if (callForm && patternIsRegExp && flags === undefined &&
                is(newTarget, pattern.constructor)) {
                return pattern;
            }
            const genuine = brand(pattern);
            let P, F = flags;
            if (!genuine) {
                if (patternIsRegExp) {
                    P = pattern.source;
                    if (flags === undefined) F = pattern.flags;
                } else {
                    P = pattern;
                }
            }
            let proto = newTarget.prototype;
            if (!isObject(proto)) proto = RegExpPrototype;
            let obj;
            if (!genuine) {
                const Pstr = P === undefined ? '' : toStr(P);
                const Fstr = F === undefined ? '' : toStr(F);
                return compileAdmitted(Pstr, Fstr, proto);
            } else if (F === undefined) {
                obj = cloneGenuine(pattern);
            } else {
                // Internal source was admitted when created; only the flags change.
                const Fstr = toStr(F);
                if (!readmit()) throw stop;
                const copy = cloneGenuine(pattern);
                setPrototypeOf(copy, null);
                obj = construct(NativeRegExp, [copy, Fstr], NativeRegExp);
            }
            if (proto !== RegExpPrototype) setPrototypeOf(obj, proto);
            return obj;
        };
        const facade = new NativeProxy(NativeRegExp, freeze({
            __proto__: null,
            apply: (target, thisArg, args) => regExpFrom(arg(args, 0), arg(args, 1), facade, true),
            construct: (target, args, newTarget) =>
                regExpFrom(arg(args, 0), arg(args, 1), newTarget, false),
        }));

        // ES2023 22.1.3.13 / .14 / .22 with RegExpCreate (22.2.3.1) through the gate.
        const stringMethod = (key, createFlags, requireGlobal) => (target, thisArg, args) => {
            enter();
            if (thisArg === undefined || thisArg === null) fail('String method on null or undefined');
            const regexp = arg(args, 0);
            if (regexp !== undefined && regexp !== null) {
                if (requireGlobal && isRegExp(regexp)) {
                    const flags = regexp.flags;
                    if (flags === undefined || flags === null) fail('flags is null or undefined');
                    if (!contains(toStr(flags), 'g')) fail('matchAll requires a global RegExp');
                }
                const matcher = regexp[key];
                if (matcher !== undefined && matcher !== null) {
                    if (typeof matcher !== 'function') fail('matcher is not callable');
                    return apply(matcher, regexp, [thisArg]);
                }
            }
            const S = toStr(thisArg);
            const Pstr = regexp === undefined ? '' : toStr(regexp);
            const rx = compileAdmitted(Pstr, createFlags, RegExpPrototype);
            return apply(rx[key], rx, [S]);
        };

        // Annex B.2.4.1: brand-checked `this`; genuine patterns copy internal state natively.
        const compile = (target, thisArg, args) => {
            enter();
            const pattern = arg(args, 0), flags = arg(args, 1);
            if (!brand(thisArg) || brand(pattern)) return apply(nativeCompile, thisArg, args);
            const Pstr = pattern === undefined ? '' : toStr(pattern);
            const Fstr = flags === undefined ? '' : toStr(flags);
            if (!admit(Pstr.length)) throw stop;
            return apply(nativeCompile, thisArg, [Pstr, Fstr]);
        };

        const speciesConstructor = (O, defaultConstructor) => {
            const C = O.constructor;
            if (C === undefined) return defaultConstructor;
            if (!isObject(C)) fail('constructor is not an object');
            const S = C[symSpecies];
            if (S === undefined || S === null) return defaultConstructor;
            if (isConstructor(S)) return S;
            return fail('species is not a constructor');
        };
        // Private receiver for the native algorithm: its species yields the prepared object.
        const helper = (prepared, flags, lastIndex) => {
            function Species() { return prepared; }
            return {
                __proto__: null,
                constructor: { __proto__: null, [symSpecies]: Species },
                flags,
                lastIndex,
            };
        };

        // ES2023 22.2.6.14 steps 1-8 here; the native loop then runs on the prepared splitter.
        const split = (target, rx, args) => {
            enter();
            if (!isObject(rx)) fail('RegExp.prototype[Symbol.split] on non-object');
            const S = toStr(arg(args, 0));
            const C = speciesConstructor(rx, facade);
            const flags = toStr(rx.flags);
            const newFlags = contains(flags, 'y') ? flags : flags + 'y';
            const splitter = construct(C, [rx, newFlags]);
            return apply(nativeSplit, helper(splitter, flags, 0), [S, arg(args, 1)]);
        };
        // ES2023 22.2.6.9 steps 1-7 here; the native sets lastIndex and builds the iterator.
        const matchAll = (target, R, args) => {
            enter();
            if (!isObject(R)) fail('RegExp.prototype[Symbol.matchAll] on non-object');
            const S = toStr(arg(args, 0));
            const C = speciesConstructor(R, facade);
            const flags = toStr(R.flags);
            const matcher = construct(C, [R, flags]);
            const lastIndex = toLength(R.lastIndex);
            return apply(nativeMatchAll, helper(matcher, flags, lastIndex), [S]);
        };

        const wrap = (native, trap) => new NativeProxy(native, freeze({ __proto__: null, apply: trap }));
        const replace = (object, key, value) => {
            const d = getOwnPropertyDescriptor(object, key);
            d.value = value;
            if (!defineProperty(object, key, d)) fail('install failed');
        };
        replace(globalThis, 'RegExp', facade);
        replace(RegExpPrototype, 'constructor', facade);
        replace(RegExpPrototype, 'compile', wrap(nativeCompile, compile));
        replace(RegExpPrototype, symSplit, wrap(nativeSplit, split));
        replace(RegExpPrototype, symMatchAll, wrap(nativeMatchAll, matchAll));
        replace(StringPrototype, 'match', wrap(nativeMatch, stringMethod(symMatch, '', false)));
        replace(StringPrototype, 'matchAll', wrap(nativeStringMatchAll, stringMethod(symMatchAll, 'g', true)));
        replace(StringPrototype, 'search', wrap(nativeSearch, stringMethod(symSearch, '', false)));
    };
})()
