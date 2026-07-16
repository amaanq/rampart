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

function removeRequestTarget(trigger) {
    if (!trigger || !trigger.dataset || !trigger.dataset.removeClosest) return;
    const target = trigger.closest(trigger.dataset.removeClosest);
    if (!target) return;
    const count = trigger.dataset.countTarget
        ? document.querySelector(trigger.dataset.countTarget)
        : null;
    target.remove();
    if (!count) return;
    const value = Number.parseInt(count.textContent, 10);
    if (Number.isFinite(value)) count.textContent = String(Math.max(0, value - 1));
}

function toggleRequestTarget(trigger) {
    if (!trigger || !trigger.dataset) return;
    const selector = trigger.dataset.toggleClosest;
    const className = trigger.dataset.toggleClass;
    if (!selector || !className) return;
    const target = trigger.closest(selector);
    if (!target) return;
    const isSet = target.classList.toggle(className);
    const label = isSet ? trigger.dataset.labelWithClass : trigger.dataset.labelWithoutClass;
    if (label) trigger.textContent = label;
}

function updateMailboxToggle(trigger) {
    if (!trigger || !trigger.hasAttribute('data-mailbox-toggle')) return;
    let enabled;
    try {
        enabled = JSON.parse(trigger.getAttribute('hx-vals')).enabled;
    } catch (_) {
        location.reload();
        return;
    }
    const row = trigger.closest('tr');
    const status = row && row.querySelector('.status-dot');
    if (typeof enabled !== 'boolean' || !status) {
        location.reload();
        return;
    }
    const label = enabled ? 'Enabled' : 'Disabled';
    status.classList.toggle('is-enabled', enabled);
    status.classList.toggle('is-disabled', !enabled);
    status.setAttribute('aria-label', label);
    status.title = label;
    trigger.textContent = enabled ? 'disable' : 'enable';
    trigger.setAttribute('hx-vals', JSON.stringify({enabled: !enabled}));
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
    removeRequestTarget(t);
    toggleRequestTarget(t);
    updateMailboxToggle(t);
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
