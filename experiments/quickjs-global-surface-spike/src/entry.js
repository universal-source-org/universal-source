import {value} from './helper.js';
// First package statement sees only hardened globals, no native alias window.
globalThis.packageEvaluated = true;
const dateAlias = typeof Date, proxyAlias = typeof Proxy, randomAlias = Math.random;
const initializationError = new Error('initialization');
if (dateAlias !== 'undefined' || proxyAlias !== 'undefined' || 'stack' in initializationError)
    throw new Error('hardening order');
try { randomAlias(); throw new Error('random at initialization'); }
catch(e) { if (!(e instanceof TypeError)) throw e; }
export async function home(input, context) {
    const binding = context.services.testPrivate.probe;
    if (Object.getPrototypeOf(context)!==null || Object.getPrototypeOf(context.services)!==null ||
        Object.getPrototypeOf(context.services.testPrivate)!==null) throw new Error('wrapper prototype');
    if (!Object.isFrozen(context) || !Object.isFrozen(context.services) ||
        !Object.isFrozen(context.services.testPrivate)) throw new Error('mutable wrapper');
    if (Reflect.ownKeys(binding).some(k=>!['length','name'].includes(k))) throw new Error('native properties');
    if (!Function.prototype.toString.call(binding).includes('[native code]')) throw new Error('native source');
    try { binding.constructor('return process')(); throw new Error('native constructor'); }
    catch(e) { if(!(e instanceof EvalError) || 'stack' in e) throw e; }
    let nativeError;
    try { binding({}); } catch(e) { nativeError=e; }
    if(!(nativeError instanceof TypeError) || 'stack' in nativeError) throw new Error('host conversion error');
    const n = binding(40);
    const same = binding.call({fakeAuthority:true}, 40);
    if (n!==42 || same!==42) throw new Error('binding receiver');
    const data = JSON.parse(JSON.stringify({items:[1,2,3].map(x=>x*2)}));
    const map = new Map([['value',value]]), set = new Set([1,1,2]);
    const wm = new WeakMap(), ws = new WeakSet(), key = {};
    wm.set(key,7);ws.add(key);
    const s=Symbol('local'); const boxed=Object(s);
    function add(a,b){return a+b;}
    class Local {constructor(x){this.x=x;}}
    const result = await Promise.resolve(add(map.get('value'),input));
    if (set.size!==2 || wm.get(key)!==7 || !ws.has(key) || boxed.valueOf()!==s ||
        new Local(3).x!==3 || !/a+/.test('aa') || data.items.join(',')!=='2,4,6' ||
        BigInt(2)+3n!==5n || 'abc'.slice(1)!=='bc' || !Number.isFinite(result))
        throw new Error('language preservation');
    return result;
}
