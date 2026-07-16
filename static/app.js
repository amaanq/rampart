function requestForm(e) {
    const elt = e.detail && e.detail.elt;
    if (!elt) return null;
    return elt.matches('form') ? elt : elt.closest('form');
}

function setFormPending(form, pending) {
    if (!form || !form.dataset.success) return;
    if (pending) {
        form.setAttribute('aria-busy', 'true');
    } else {
        form.removeAttribute('aria-busy');
    }
    form.querySelectorAll('button[type="submit"]').forEach(function (button) {
        button.disabled = pending;
    });
}

function showFormStatus(form, message, isError) {
    if (!form) return false;
    const status = form.querySelector('[data-form-status]');
    if (!status) return false;
    status.textContent = message;
    status.classList.toggle('is-error', isError);
    return true;
}

document.body.addEventListener('htmx:beforeRequest', function (e) {
    const form = requestForm(e);
    if (!form || !form.dataset.success) return;
    showFormStatus(form, '', false);
    setFormPending(form, true);
});

document.body.addEventListener('htmx:afterRequest', function (e) {
    const form = requestForm(e);
    setFormPending(form, false);
    if (!e.detail.successful) return;
    if (form && form.dataset.success) {
        showFormStatus(form, form.dataset.success, false);
        if (form.hasAttribute('data-reset-on-success')) form.reset();
    }
    const t = e.detail.elt;
    if (t && t.dataset && t.dataset.reload === 'no') return;
    location.reload();
});

// Surface 4xx/5xx response bodies as a banner so failed POSTs aren't
// silent. Sl's ApiError responses return the message as text/plain.
function showError(msg) {
    let el = document.getElementById('rampart-error-banner');
    if (!el) {
        el = document.createElement('div');
        el.id = 'rampart-error-banner';
        el.className = 'rampart-error-banner';
        document.body.insertBefore(el, document.body.firstChild);
    }
    el.textContent = msg;
    el.scrollIntoView({behavior: 'smooth', block: 'start'});
}
document.body.addEventListener('htmx:responseError', function (e) {
    const xhr = e.detail.xhr;
    const msg = (xhr.responseText || '').trim() || `${xhr.status} ${xhr.statusText}`;
    const form = requestForm(e);
    setFormPending(form, false);
    if (!showFormStatus(form, msg, true)) showError(msg);
});
document.body.addEventListener('htmx:sendError', function (e) {
    const msg = 'Network error — could not reach server.';
    const form = requestForm(e);
    setFormPending(form, false);
    if (!showFormStatus(form, msg, true)) showError(msg);
});
