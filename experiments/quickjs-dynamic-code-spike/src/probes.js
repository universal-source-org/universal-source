// Fixed diagnostic module; compiled unchanged by the host. No production dispatcher.
const savedEval = eval;
const savedFunction = Function;
const getProto = Object.getPrototypeOf;
const errorPrototype = EvalError.prototype;
const nativeApply = Reflect.apply;
const nativeConstruct = Reflect.construct;
const hasOwn = Object.hasOwn;
const families = [Function, (async function() {}).constructor,
    (function*() {}).constructor, (async function*() {}).constructor];
const body = "globalThis.__dynamic_code_executed = true; return 123";
const evalBody = "globalThis.__dynamic_code_executed = true; globalThis.__pwned = true; 123";

export function home() { return 42; }

export function check() {
    const routes = [
        ["direct eval arithmetic", () => eval("1 + 1")],
        ["direct eval marker", () => eval(evalBody)],
        ["comma eval", () => (0, eval)(evalBody)],
        ["saved eval", () => savedEval(evalBody)],
        ["global eval", () => globalThis.eval(evalBody)],
        ["eval descriptor", () => Object.getOwnPropertyDescriptor(globalThis,"eval").value(evalBody)],
        ["Function descriptor", () => Object.getOwnPropertyDescriptor(globalThis,"Function").value(body)],
        ["eval call", () => savedEval.call(null, evalBody)],
        ["eval apply", () => savedEval.apply(null, [evalBody])],
        ["eval bind", () => savedEval.bind(null)(evalBody)],
        ["reflect eval", () => nativeApply(savedEval, null, [evalBody])],
        ["Function", () => Function(body)()],
        ["new Function", () => new Function(body)()],
        ["saved Function", () => savedFunction(body)()],
        ["ordinary constructor", () => (function() {}).constructor(body)()],
        ["arrow constructor", () => (() => {}).constructor(body)()],
        ["prototype constructor", () => getProto(function() {}).constructor(body)()],
        ["constructor constructor", () => (function() {}).constructor.constructor(body)()],
        ["Function constructor", () => Function.constructor(body)()],
        ["object meta constructor", () => ({}).constructor.constructor(body)()],
        ["bound function constructor", () => (function() {}).bind(null).constructor(body)()],
        ["method constructor", () => ({m(){}}).m.constructor(body)()],
        ["getter constructor", () => Object.getOwnPropertyDescriptor({get x(){}}, "x").get.constructor(body)()],
        ["builtin method constructor", () => Array.prototype.map.constructor(body)()],
        ["error constructor chain", () => Error.constructor(body)()],
        ["generator object chain", () => getProto(getProto((function*() {})())).constructor.constructor(body)],
        ["async generator object chain", () => getProto(getProto((async function*() {})())).constructor.constructor(body)],
        ["async arrow constructor", () => (async () => {}).constructor(body)],
        ["class extends Function", () => { class C extends savedFunction {} return new C(body); }],
        ["malformed eval", () => savedEval("let = ???")],
        ["malformed Function", () => savedFunction("let = ???")],
        ["eval nonstring", () => savedEval(123)],
        ["Function no arguments", () => savedFunction()],
    ];
    for (let i = 0; i < families.length; i++) {
        const ctor = families[i];
        routes.push(
            [`family ${i} call`, () => ctor(body)],
            [`family ${i} new`, () => new ctor(body)],
            [`family ${i} Reflect.construct`, () => nativeConstruct(ctor, [body])],
            [`family ${i} Reflect.apply`, () => nativeApply(ctor, null, [body])],
            [`family ${i} .call`, () => ctor.call(null, body)],
            [`family ${i} .apply`, () => ctor.apply(null, [body])],
            [`family ${i} .bind`, () => ctor.bind(null, body)()],
            [`family ${i} bound construction`, () => new (ctor.bind(null, body))()],
            [`family ${i} meta constructor`, () => ctor.constructor(body)],
            [`family ${i} prototype constructor`, () => ctor.prototype.constructor(body)],
            [`family ${i} constructor descriptor`, () => Object.getOwnPropertyDescriptor(ctor.prototype,"constructor").value(body)]
        );
        if (i > 0) routes.push([`family ${i} parent`, () => getProto(ctor)(body)]);
    }
    for (const [name, attempt] of routes) {
        let error;
        try { attempt(); } catch (e) { error = e; }
        if (!error || getProto(error) !== errorPrototype) throw new Error(`wrong error: ${name}`);
        if (hasOwn(globalThis, "__dynamic_code_executed") || hasOwn(globalThis, "__pwned"))
            throw new Error(`payload executed: ${name}`);
    }
    return routes.length;
}

export function mutate() {
    const define = Object.defineProperty;
    const set = Reflect.set;
    const harmless = () => 9;
    for (const key of ["eval", "Function"]) {
        try { globalThis[key] = harmless; throw new Error("assigned global"); }
        catch (e) { if (!(e instanceof TypeError)) throw e; }
        if (set(globalThis, key, harmless)) throw new Error("writable global");
        try { define(globalThis, key, {value: harmless}); throw new Error("redefined global"); }
        catch (e) { if (!(e instanceof TypeError)) throw e; }
        if (Reflect.deleteProperty(globalThis, key)) throw new Error("deleted global");
    }
    for (const ctor of families) {
        try { ctor.prototype.constructor = harmless; throw new Error("assigned constructor"); }
        catch (e) { if (!(e instanceof TypeError)) throw e; }
        if (set(ctor.prototype, "constructor", harmless)) throw new Error("writable constructor");
        try { define(ctor.prototype, "constructor", {get: () => harmless}); throw new Error("redefined constructor"); }
        catch (e) { if (!(e instanceof TypeError)) throw e; }
        if (Reflect.deleteProperty(ctor.prototype, "constructor")) throw new Error("deleted constructor");
    }
    // Restoration from source-recoverable references only restores guarded identities.
    define(globalThis, "eval", {value: savedEval});
    define(globalThis, "Function", {value: savedFunction});
    for (const ctor of families) define(ctor.prototype, "constructor", {value: ctor});
    // Poison the generic object prototype: a non-null proxy handler would inherit these traps.
    Object.prototype.get = () => savedFunction;
    Object.prototype.apply = () => 999;
    Object.prototype.construct = () => ({});
    Object.prototype.constructor = savedFunction;
    // Source can mutate unrelated intrinsic state, but cannot uncover hidden native targets.
    Function.prototype.hostile = true;
    Object.setPrototypeOf(Function.prototype, {constructor: savedFunction});
    Object.setPrototypeOf(Function, Function.prototype);
    for (let i = 1; i < families.length; i++) Object.setPrototypeOf(families[i], savedFunction);
    // Mutable public helpers are not trusted by the guard or error classifier.
    Reflect.apply = (...args) => nativeApply(...args);
    Reflect.construct = (...args) => nativeConstruct(...args);
    Object.defineProperty = (...args) => define(...args);
    globalThis.EvalError = function FakeEvalError() { throw new Error("wrong constructor"); };
    globalThis.hostileInvocation = true;
    return true;
}
