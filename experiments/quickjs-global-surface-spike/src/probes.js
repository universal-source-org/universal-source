// Fixed diagnostic package, compiled unchanged. Captures only post-hardening aliases.
const TypeErrorPrototype = TypeError.prototype;
const getProto = Object.getPrototypeOf;
const ownKeys = Reflect.ownKeys;
const define = Object.defineProperty;
const apply = Reflect.apply;
const initError = new Error('module initialization');
const restricted = [
    [Math, 'random', []], [String.prototype, 'localeCompare', ['j']],
    [String.prototype, 'toLocaleLowerCase', []], [String.prototype, 'toLocaleUpperCase', []],
    [Object.prototype, 'toLocaleString', []], [Array.prototype, 'toLocaleString', []],
    [Number.prototype, 'toLocaleString', []], [BigInt.prototype, 'toLocaleString', []]
];
const saved = restricted.map(([o,k,args]) => [o[k],args]);
function assert(v, message) { if (!v) throw new Error(message); }
function denied(f) {
    let error;
    try { f(); } catch(e) { error = e; }
    assert(error && getProto(error) === TypeErrorPrototype, 'expected native TypeError');
    assert(error.stack === undefined && !('stack' in error), 'guard stack');
}
export function clocks() {
    for (const f of [()=>Date.now(),()=>new Date(),()=>Date(),
        ()=>new Date(0).getTimezoneOffset(),()=>new Date(0).toLocaleString(),
        ()=>new Date(0).toString(),()=>performance.now(),()=>performance.timeOrigin]) {
        let error; try {f();} catch(e) {error=e;}
        assert(error instanceof ReferenceError,'clock binding remains');
    }
    for(const k of ['Date','performance','setTimeout','setInterval','queueMicrotask']) {
        assert(!(k in globalThis) && !ownKeys(globalThis).includes(k),'clock own key');
    }
    return true;
}
export function restrictedMethods() {
    for(const [o,k,args] of restricted) denied(()=>apply(o[k],o,args));
    for(const [f,args] of saved) {
        denied(()=>apply(f,null,args));
        denied(()=>f.call(null,...args));
        denied(()=>f.bind(null)(...args));
    }
    for(const k of ['crypto','random','randomBytes','getRandomValues','randomUUID','std','os'])
        assert(!(k in globalThis),'other randomness');
    assert(typeof Intl === 'undefined','Intl');
    denied(()=>'i'.toLocaleUpperCase());
    denied(()=>(1234.5).toLocaleString());
    denied(()=>(1234n).toLocaleString());
    assert('i'.toUpperCase()==='I' && 'I'.toLowerCase()==='i' && 'a'<'b','deterministic alternatives');
    return true;
}
export function extensions() {
    const absent = ['InternalError','SuppressedError','Iterator','DisposableStack','AsyncDisposableStack',
        'DOMException','btoa','atob','escape','unescape','Proxy','WeakRef','FinalizationRegistry',
        'ArrayBuffer','SharedArrayBuffer','DataView','Atomics','WebAssembly','Int8Array','Uint8Array',
        'Uint8ClampedArray','Int16Array','Uint16Array','Int32Array','Uint32Array','BigInt64Array',
        'BigUint64Array','Float16Array','Float32Array','Float64Array'];
    for(const k of absent) assert(!(k in globalThis),'extension global '+k);
    for(const [o,keys] of [[Error,['isError','captureStackTrace','stackTraceLimit','prepareStackTrace']],
        [Function.prototype,['fileName','lineNumber','columnNumber']],
        [Symbol,['dispose','asyncDispose']],[Math,['sumPrecise','f16round']],
        [Object,['groupBy']],[Map,['groupBy']],[Promise,['try','withResolvers']],
        [JSON,['rawJSON','isRawJSON']],[RegExp,['escape']],[Array,['fromAsync']],
        [String.prototype,['isWellFormed','toWellFormed']]]) {
        for(const k of keys) assert(!Object.hasOwn(o,k),'extension property '+k);
    }
    const ip = getProto(getProto([][Symbol.iterator]()));
    assert(ownKeys(ip).length===1 && ownKeys(ip)[0]===Symbol.iterator,'Iterator extension edge');
    assert(Object(Symbol()).constructor === Symbol,'Symbol constructor facade');
    assert(ownKeys(Symbol).every(k=>k!=='dispose'&&k!=='asyncDispose'),'Symbol keys');
    try {new Symbol(); throw new Error('Symbol construct');} catch(e) {assert(e instanceof TypeError,'Symbol new');}
    assert(Symbol.keyFor(Symbol.for('probe')) === 'probe','Symbol registry');
    return true;
}
function noStack(e) {
    assert(e.stack === undefined && !('stack' in e),'stack visible');
    for(let p=e;p!==null;p=getProto(p)) {
        assert(!ownKeys(p).includes('stack'),'stack descriptor');
        assert(!Object.getOwnPropertyDescriptor(p,'stack'),'stack accessor');
    }
}
export function stacks() {
    noStack(initError);
    for(const C of [Error,TypeError,EvalError,RangeError,ReferenceError,SyntaxError,URIError,AggregateError]) {
        const e = C===AggregateError ? new C([], 'x') : new C('x');
        noStack(e);
        class Sub extends C {}
        noStack(C===AggregateError ? new Sub([], 'sub') : new Sub('sub'));
        try {throw e;} catch(caught) {noStack(caught);}
    }
    function outer() { function inner() {return new Error('nested');} return inner(); }
    noStack(outer());
    const faults = [()=>null.x,()=>missingBinding,()=>new Array(-1),()=>new RegExp('['),
        ()=>decodeURI('%'),()=>eval('1'),()=>Math.random()];
    for(const f of faults) {
        let e;try{f();}catch(caught){e=caught;}
        assert(e,'fault missing'); noStack(e);
    }
    function recurse(){return 1 + recurse();}
    try {recurse();} catch(e) {
        noStack(e);
        assert(e.constructor===RangeError && e.name==='RangeError','pinned stack overflow family');
    }
    // Deleted engine setters cannot be recovered by installing same-named ordinary data.
    let calls=0;
    Error.prepareStackTrace = () => {calls++;return 'forbidden';};
    Error.stackTraceLimit = 100;
    noStack(new Error('after restoration')); assert(calls===0,'prepare hook restored');
    delete Error.prepareStackTrace; delete Error.stackTraceLimit;
    return true;
}
export function mutations() {
    for(const [o,k] of restricted) {
        const f = o[k];
        assert(!Reflect.set(o,k,()=>0),'write restricted');
        assert(!Reflect.deleteProperty(o,k),'delete restricted');
        denied(()=>define(o,k,{value:()=>0}));
        denied(()=>define(o,k,{get:()=>()=>0}));
        define(o,k,{value:f}); // Restores only the guard.
        Object.setPrototypeOf(f,null);
        denied(()=>apply(f,null,[]));
    }
    globalThis.foo = 1;
    define(globalThis,'localMemory',{value:2,configurable:true});
    assert(delete globalThis.localMemory,'ordinary local deletion');
    for(const k of ['Date','Intl','WebAssembly','Proxy','performance','crypto']) {
        define(globalThis,k,{value:()=>7,writable:true,configurable:true});
        assert(globalThis[k]()===7,'harmless replacement');
        delete globalThis[k];
    }
    for(const k of ['eval','Function']) {
        assert(!Reflect.deleteProperty(globalThis,k),'protected compilation global');
        denied(()=>define(globalThis,k,{value:()=>7}));
    }
    // Own locked guards resist inherited replacements; public binding replacement only
    // creates local source behavior, never the original random function or clock.
    const math = Math;
    globalThis.Math = {random:()=>7};
    assert(Math.random()===7,'source data');
    globalThis.Math = math;
    const nativeTypeError = TypeError;
    globalThis.TypeError = function FakeTypeError() {throw new Error('replaced global');};
    Object.prototype.apply = () => 99;
    Object.prototype.construct = () => ({});
    Object.prototype.get = () => 99;
    for(const [f,args] of saved) denied(()=>apply(f,null,args));
    globalThis.TypeError = nativeTypeError;
    delete Object.prototype.apply; delete Object.prototype.construct; delete Object.prototype.get;
    return true;
}
export function reflection() {
    function sourceFunction(){return 42;}
    const stringify = Function.prototype.toString;
    assert(stringify.call(sourceFunction).includes('return 42'),'source reflection');
    for(const f of [eval,Function,(async function(){}).constructor,Math.random,Array.prototype.map]) {
        const text = stringify.call(f);
        assert(text.includes('[native code]'),'guard/native formatting');
        assert(!text.includes('@host/') && !text.includes('/fixture/') && !text.includes('handler'),'privileged source');
        assert(f.fileName===undefined && f.lineNumber===undefined && f.columnNumber===undefined,'function locations');
    }
    return true;
}
