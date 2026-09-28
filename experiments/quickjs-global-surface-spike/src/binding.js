// Fixed authority-leak test. No service name or signature is being proposed.
(binding => {
    const wrapper=Object.freeze(Object.assign(Object.create(null),{probe:binding}));
    return Object.freeze(Object.assign(Object.create(null),{services:
        Object.freeze(Object.assign(Object.create(null),{testPrivate:wrapper}))}));
})
