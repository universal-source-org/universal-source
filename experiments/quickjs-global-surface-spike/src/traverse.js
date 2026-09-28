// Trusted test-only witness. Captured forbidden identities never enter the package.
(allowedGlobals => {
    const keys=Reflect.ownKeys, desc=Object.getOwnPropertyDescriptor, proto=Object.getPrototypeOf;
    const forbidden=new Map();
    function remember(v,label) {
        if(v && (typeof v==='function'||typeof v==='object')) forbidden.set(v,label);
    }
    for(const k of keys(globalThis)) {
        if(typeof k!=='string'||!allowedGlobals.includes(k)) remember(desc(globalThis,k).value,String(k));
    }
    for(const [o,ks] of [[Function.prototype,['fileName','lineNumber','columnNumber']],
        [Error.prototype,['stack']],[Error,['captureStackTrace','prepareStackTrace','stackTraceLimit']],
        [Math,['random']],[String.prototype,['localeCompare','toLocaleLowerCase','toLocaleUpperCase']],
        [Object.prototype,['toLocaleString']],[Array.prototype,['toLocaleString']],[Number.prototype,['toLocaleString']]]) {
        for(const k of ks) {
            const d=desc(o,k); if(!d)continue;
            for(const edge of ['value','get','set']) remember(d[edge],k+'.'+edge);
        }
    }
    remember(Symbol,'original Symbol with extension constants');
    remember(eval,'native eval');
    for(const C of [Function,(async function(){}).constructor,(function*(){}).constructor,
        (async function*(){}).constructor]) remember(C,'native compiler');
    return (mustComplete, injected) => {
        const roots=[globalThis,()=>{},async()=>{},function*(){},async function*(){},
            [][Symbol.iterator](),''[Symbol.iterator](),new Map().entries(),new Set().values(),
            ''.matchAll(/a/g),(function*(){})(),(async function*(){})(),new Error('x'),injected];
        for(const f of [()=>null.x,()=>unknownBinding,()=>new Array(-1),()=>new RegExp('[')]) {
            try{f();}catch(e){roots.push(e);}
        }
        const queue=roots.map(v=>[v,0,'root']),seen=new Set();
        let edges=0,maxDepth=0;
        for(let i=0;i<queue.length;i++) {
            const [v,depth,path]=queue[i];
            if(!v || (typeof v!=='object'&&typeof v!=='function') || seen.has(v))continue;
            if(forbidden.has(v))return 'forbidden:'+forbidden.get(v)+':'+path;
            if(depth>16 || seen.size>=4096)throw new Error('traversal bound exceeded');
            seen.add(v);maxDepth=Math.max(maxDepth,depth);
            const add=(child,label)=>{edges++;queue.push([child,depth+1,path+'.'+label]);};
            add(proto(v),'[[Prototype]]');
            for(const k of keys(v)) {
                const d=desc(v,k);
                for(const edge of ['value','get','set']) if(edge in d)add(d[edge],String(k)+':'+edge);
            }
        }
        if(!mustComplete)throw new Error('negative control did not find forbidden identity');
        return `clean:nodes=${seen.size},edges=${edges},depth=${maxDepth}`;
    };
})
