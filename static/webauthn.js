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

function setPasskeyFormPending(form, pending) {
    if (pending) {
        form.setAttribute('aria-busy', 'true');
    } else {
        form.removeAttribute('aria-busy');
    }
    const button = form.querySelector('button[type="submit"]');
    if (button) button.disabled = pending;
}

function showPasskeyFormStatus(form, message, isError) {
    const status = form.querySelector('[data-passkey-status]');
    if (!status) return;
    status.textContent = message;
    status.classList.toggle('is-error', isError);
}

function passkeyLoginDestination(form) {
    const next = form.dataset.next || '/';
    return next.startsWith('/') && !next.startsWith('//') ? next : '/';
}

async function responseMessage(response, fallback) {
    const message = (await response.text()).trim();
    return message || fallback;
}

async function registerPasskey(ev) {
    ev.preventDefault();
    const form = ev.currentTarget;
    const name = document.getElementById('pk-name').value.trim();
    if (!name) return;
    showPasskeyFormStatus(form, '', false);
    setPasskeyFormPending(form, true);

    try {
        if (!window.PublicKeyCredential || !navigator.credentials) {
            throw new Error('Passkeys are not supported in this browser.');
        }
        const r1 = await fetch('/api/v1/user/webauthn/register/start', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: '{}',
        });
        if (!r1.ok) {
            throw new Error(await responseMessage(r1, 'Could not start passkey registration.'));
        }
        const { ceremony_id, challenge } = await r1.json();
        const pk = normalize(challenge);
        const cred = await navigator.credentials.create({ publicKey: pk });
        if (!cred) throw new Error('Passkey registration was cancelled.');
        const r2 = await fetch('/api/v1/user/webauthn/register/finish', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ ceremony_id, name, credential: serializeCred(cred) }),
        });
        if (!r2.ok) {
            throw new Error(await responseMessage(r2, 'Could not finish passkey registration.'));
        }
        form.reset();
        showPasskeyFormStatus(form, 'Passkey registered. Refreshing…', false);
        window.setTimeout(function () {
            location.reload();
        }, 800);
    } catch (error) {
        const cancelled = error && (error.name === 'NotAllowedError' || error.name === 'AbortError');
        let message = 'Passkey registration failed.';
        if (cancelled) {
            message = 'Passkey registration was cancelled or timed out.';
        } else if (error && error.message) {
            message = error.message;
        }
        showPasskeyFormStatus(form, message, true);
        setPasskeyFormPending(form, false);
    }
}

// Passkey LOGIN. Used by the /login page's optional passkey form.
// Submits the user's email, calls /api/v1/auth/passkey/start, drives
// navigator.credentials.get(), then /finish — on success, server sets
// the session cookie and we redirect to /.
async function loginWithPasskey(ev) {
    ev.preventDefault();
    const form = ev.currentTarget;
    // Login form unified the email field — read from #email if the
    // dedicated #pk-login-email isn't there. Safe in either layout.
    const emailEl =
        document.getElementById('pk-login-email') ||
        document.getElementById('email');
    const email = (emailEl && emailEl.value.trim()) || '';
    if (!email) {
        showPasskeyFormStatus(form, 'Enter your email above first.', true);
        if (emailEl) emailEl.focus();
        return;
    }
    showPasskeyFormStatus(form, 'Preparing passkey sign-in…', false);
    setPasskeyFormPending(form, true);

    try {
        if (!window.PublicKeyCredential || !navigator.credentials) {
            throw new Error('Passkeys are not supported in this browser.');
        }
        const r1 = await fetch('/api/v1/auth/passkey/start', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ email }),
        });
        if (!r1.ok) {
            const message = r1.status === 429
                ? 'Too many attempts. Try again later.'
                : 'Passkey sign-in is unavailable.';
            throw new Error(message);
        }
        const { ceremony_id, challenge } = await r1.json();
        const pk = normalize(challenge);
        showPasskeyFormStatus(form, 'Waiting for your passkey…', false);
        const cred = await navigator.credentials.get({ publicKey: pk });
        if (!cred) throw new Error('Passkey sign-in was cancelled.');
        const r2 = await fetch('/api/v1/auth/passkey/finish', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ ceremony_id, credential: serializeCred(cred) }),
        });
        if (!r2.ok) throw new Error('Passkey sign-in failed.');
        showPasskeyFormStatus(form, 'Signed in. Redirecting…', false);
        location.href = passkeyLoginDestination(form);
    } catch (error) {
        const cancelled = error && (error.name === 'NotAllowedError' || error.name === 'AbortError');
        let message = 'Passkey sign-in failed.';
        if (cancelled) {
            message = 'Passkey sign-in was cancelled or timed out.';
        } else if (error && error.message) {
            message = error.message;
        }
        showPasskeyFormStatus(form, message, true);
        setPasskeyFormPending(form, false);
    }
}

document.addEventListener('DOMContentLoaded', function () {
    const f = document.getElementById('passkey-register-form');
    if (f) f.addEventListener('submit', registerPasskey);
    const lf = document.getElementById('passkey-login-form');
    if (lf) lf.addEventListener('submit', loginWithPasskey);
});
