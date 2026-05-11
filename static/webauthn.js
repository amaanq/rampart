// WebAuthn passkey registration. Moved out of templates/settings.html so
// strict CSP (`script-src 'self'`) can block inline scripts; the
// `<form id="passkey-register-form">` in the template wires its submit
// handler here via addEventListener (NOT inline `onsubmit=`, which is
// also blocked by `script-src 'self'`).

function b64urlToBuf(s) {
    s = s.replace(/-/g, '+').replace(/_/g, '/');
    while (s.length % 4) s += '=';
    return Uint8Array.from(atob(s), c => c.charCodeAt(0));
}

function bufToB64url(buf) {
    let s = btoa(String.fromCharCode.apply(null, new Uint8Array(buf)));
    return s.replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

// webauthn-rs emits bytes as plain arrays in some places and base64url
// strings in others. Walk the challenge structure and convert known byte
// fields to ArrayBuffers for navigator.credentials.create().
function normalize(challenge) {
    const pk = challenge.publicKey;
    pk.challenge = b64urlToBuf(pk.challenge);
    pk.user.id = b64urlToBuf(pk.user.id);
    if (pk.excludeCredentials) {
        pk.excludeCredentials = pk.excludeCredentials.map(c => ({ ...c, id: b64urlToBuf(c.id) }));
    }
    if (pk.allowCredentials) {
        pk.allowCredentials = pk.allowCredentials.map(c => ({ ...c, id: b64urlToBuf(c.id) }));
    }
    return pk;
}

function serializeCred(cred) {
    return {
        id: cred.id,
        rawId: bufToB64url(cred.rawId),
        type: cred.type,
        response: {
            attestationObject: cred.response.attestationObject ? bufToB64url(cred.response.attestationObject) : undefined,
            clientDataJSON: bufToB64url(cred.response.clientDataJSON),
            authenticatorData: cred.response.authenticatorData ? bufToB64url(cred.response.authenticatorData) : undefined,
            signature: cred.response.signature ? bufToB64url(cred.response.signature) : undefined,
            userHandle: cred.response.userHandle ? bufToB64url(cred.response.userHandle) : undefined,
        },
        clientExtensionResults: cred.getClientExtensionResults ? cred.getClientExtensionResults() : {},
    };
}

async function registerPasskey(ev) {
    ev.preventDefault();
    const name = document.getElementById('pk-name').value.trim();
    if (!name) return;
    const r1 = await fetch('/api/v1/user/webauthn/register/start', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: '{}',
    });
    if (!r1.ok) { alert('register start failed'); return; }
    const { ceremony_id, challenge } = await r1.json();
    const pk = normalize(challenge);
    const cred = await navigator.credentials.create({ publicKey: pk });
    const r2 = await fetch('/api/v1/user/webauthn/register/finish', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ ceremony_id, name, credential: serializeCred(cred) }),
    });
    if (r2.ok) {
        location.reload();
    } else {
        alert('register finish failed: ' + await r2.text());
    }
}

// Inline replacements for settings.html's old `hx-on::after-request="..."`
// blocks on the password and email-change forms. The strict-CSP refactor
// strips those attributes; we listen here instead and key off the form's
// id so we don't trigger on every successful request.
document.body.addEventListener('htmx:afterRequest', function (e) {
    const id = e.detail.elt && e.detail.elt.id;
    if (id === 'password-change-form') {
        if (e.detail.successful) {
            alert("password changed; you'll need to log in again");
            location.href = '/login';
        } else {
            alert('failed: ' + e.detail.xhr.responseText);
        }
    } else if (id === 'email-change-form') {
        alert(e.detail.successful ? 'confirmation email sent' : 'failed');
    }
});

// Passkey LOGIN. Used by the /login page's optional passkey form.
// Submits the user's email, calls /api/v1/auth/passkey/start, drives
// navigator.credentials.get(), then /finish — on success, server sets
// the session cookie and we redirect to /.
async function loginWithPasskey(ev) {
    ev.preventDefault();
    // Login form unified the email field — read from #email if the
    // dedicated #pk-login-email isn't there. Safe in either layout.
    const emailEl =
        document.getElementById('pk-login-email') ||
        document.getElementById('email');
    const email = (emailEl && emailEl.value.trim()) || '';
    const status = document.getElementById('pk-login-status');
    if (!email) {
        if (status) status.textContent = 'enter your email above first';
        if (emailEl) emailEl.focus();
        return;
    }
    if (status) status.textContent = '';
    const r1 = await fetch('/api/v1/auth/passkey/start', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email }),
    });
    if (!r1.ok) {
        if (status) status.textContent = 'passkey unavailable';
        return;
    }
    const { ceremony_id, challenge } = await r1.json();
    const pk = normalize(challenge);
    let cred;
    try {
        cred = await navigator.credentials.get({ publicKey: pk });
    } catch (e) {
        if (status) status.textContent = 'cancelled';
        return;
    }
    const r2 = await fetch('/api/v1/auth/passkey/finish', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ ceremony_id, credential: serializeCred(cred) }),
    });
    if (r2.ok) {
        location.href = '/';
    } else if (status) {
        status.textContent = 'auth failed';
    }
}

document.addEventListener('DOMContentLoaded', function () {
    const f = document.getElementById('passkey-register-form');
    if (f) f.addEventListener('submit', registerPasskey);
    const lf = document.getElementById('passkey-login-form');
    if (lf) lf.addEventListener('submit', loginWithPasskey);
});
