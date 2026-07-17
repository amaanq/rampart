function requestForm(e) {
    const elt = e.detail && e.detail.elt;
    if (!elt) return null;
    return elt.matches('form') ? elt : elt.closest('form');
}

function markCurrentNavigation() {
    const path = location.pathname;
    let href = null;
    let scope = 'header nav';
    if (path === '/' || path.startsWith('/aliases/')) {
        href = '/';
    } else if (path === '/mailboxes' || path === '/domains' || path === '/settings') {
        href = path;
    } else if (path === '/admin/users' || path === '/admin/domains') {
        href = path;
        scope = '.user-menu-panel';
    }
    if (!href) return;
    const link = document.querySelector(`${scope} a[href="${href}"]`);
    if (link) link.setAttribute('aria-current', 'page');
}

markCurrentNavigation();

function setFormPending(form, pending) {
    if (!form || (!form.dataset.success && !form.querySelector('[data-form-status]'))) return;
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

function updateBooleanToggle(trigger) {
    if (!trigger || !trigger.dataset || !trigger.dataset.booleanToggle) return;
    const property = trigger.dataset.booleanToggle;
    let value;
    try {
        value = JSON.parse(trigger.getAttribute('hx-vals'))[property];
    } catch (_) {
        location.reload();
        return;
    }
    if (typeof value !== 'boolean') {
        location.reload();
        return;
    }
    const buttonLabel = value ? trigger.dataset.labelTrue : trigger.dataset.labelFalse;
    const buttonLabelTarget = trigger.dataset.labelTarget
        ? trigger.querySelector(trigger.dataset.labelTarget)
        : trigger;
    if (buttonLabel && buttonLabelTarget) buttonLabelTarget.textContent = buttonLabel;
    trigger.setAttribute('hx-vals', JSON.stringify({[property]: !value}));

    const row = trigger.closest('tr');
    const textSelector = trigger.dataset.textTarget;
    if (textSelector) {
        const textTarget = row && row.querySelector(textSelector);
        const text = value ? trigger.dataset.textTrue : trigger.dataset.textFalse;
        if (!textTarget || !text) {
            location.reload();
            return;
        }
        textTarget.textContent = text;
    }

    const statusSelector = trigger.dataset.statusTarget;
    if (!statusSelector) return;
    const status = row && row.querySelector(statusSelector);
    if (!status) {
        location.reload();
        return;
    }
    const statusLabel = value ? 'Enabled' : 'Disabled';
    status.classList.toggle('is-enabled', value);
    status.classList.toggle('is-disabled', !value);
    status.setAttribute('aria-label', statusLabel);
    status.title = statusLabel;
}

function completeRequestTrigger(trigger) {
    if (!trigger || !trigger.dataset || !trigger.dataset.successLabel) return;
    trigger.textContent = trigger.dataset.successLabel;
    if (trigger.hasAttribute('data-disable-on-success')) trigger.disabled = true;
}

document.body.addEventListener('htmx:beforeRequest', function (e) {
    clearError();
    const form = requestForm(e);
    if (!form) return;
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
        if (form.dataset.successRedirect) {
            setFormPending(form, true);
            window.setTimeout(function () {
                location.assign(form.dataset.successRedirect);
            }, 800);
        }
    }
    const t = e.detail.elt;
    removeRequestTarget(t);
    toggleRequestTarget(t);
    updateBooleanToggle(t);
    completeRequestTrigger(t);
    if (t && t.dataset && t.dataset.reload === 'no') return;
    location.reload();
});

function clearError() {
    const el = document.getElementById('rampart-error-banner');
    if (el) el.remove();
}

function showError(msg) {
    clearError();
    const el = document.createElement('div');
    el.id = 'rampart-error-banner';
    el.className = 'rampart-error-banner';
    el.setAttribute('role', 'alert');

    const message = document.createElement('span');
    message.className = 'rampart-error-message';
    message.textContent = msg;

    const dismiss = document.createElement('button');
    dismiss.type = 'button';
    dismiss.className = 'rampart-error-dismiss';
    dismiss.setAttribute('aria-label', 'Dismiss error');
    dismiss.textContent = '×';
    dismiss.addEventListener('click', clearError);

    el.append(message, dismiss);
    document.body.appendChild(el);
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
