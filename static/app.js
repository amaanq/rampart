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
    } else if (path === '/mailboxes' || path.startsWith('/domains') || path === '/settings') {
        href = path.startsWith('/domains') ? '/domains' : path;
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

function recoverFormFromError(form) {
    if (!form) return;
    const clearSelector = form.dataset.clearOnError;
    if (clearSelector) {
        form.querySelectorAll(clearSelector).forEach(function (input) {
            input.value = '';
        });
    }
    const focusSelector = form.dataset.focusOnError;
    if (!focusSelector) return;
    const target = form.querySelector(focusSelector);
    if (target) target.focus();
}

function removeRequestTarget(trigger) {
    if (!trigger || !trigger.dataset || !trigger.dataset.removeClosest) return;
    const target = trigger.closest(trigger.dataset.removeClosest);
    if (!target) return;
    const count = trigger.dataset.countTarget
        ? document.querySelector(trigger.dataset.countTarget)
        : null;
    const table = target.closest('table');
    const shouldMoveFocus = document.activeElement === trigger;
    let nextAction = null;
    if (table && target.matches('tr')) {
        const rows = Array.from(table.querySelectorAll('tbody tr:not(.is-removed)'));
        const index = rows.indexOf(target);
        const nextRow = rows[index + 1] || rows[index - 1];
        nextAction = nextRow && nextRow.querySelector('[hx-delete]');
        target.classList.add('is-removed');
        target.setAttribute('aria-hidden', 'true');
        target.inert = true;
    } else {
        target.remove();
    }
    if (count) {
        const value = Number.parseInt(count.textContent, 10);
        if (Number.isFinite(value)) count.textContent = String(Math.max(0, value - 1));
    }
    if (!table || table.querySelector('tbody tr:not(.is-removed)')) {
        if (shouldMoveFocus && nextAction) nextAction.focus({preventScroll: true});
        return;
    }
    const message = table.dataset.emptyMessage;
    if (!message) return;
    const emptyState = document.createElement('div');
    const emptyClass = table.dataset.emptyClass;
    emptyState.className = emptyClass || 'empty-state';
    if (emptyClass) {
        emptyState.textContent = message;
    } else {
        const copy = document.createElement('p');
        copy.textContent = message;
        emptyState.appendChild(copy);
    }
    table.replaceWith(emptyState);
    if (shouldMoveFocus) {
        emptyState.tabIndex = -1;
        emptyState.focus({preventScroll: true});
    }
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

    const classWith = trigger.dataset.classWithClass;
    const classWithout = trigger.dataset.classWithoutClass;
    if (classWith && classWithout) {
        trigger.classList.remove(classWith, classWithout);
        trigger.classList.add(isSet ? classWith : classWithout);
    }

    const confirmMessage = isSet
        ? trigger.dataset.confirmWithClass
        : trigger.dataset.confirmWithoutClass;
    if (confirmMessage) {
        trigger.setAttribute('hx-confirm', confirmMessage);
    } else if (trigger.hasAttribute('data-confirm-with-class')
        || trigger.hasAttribute('data-confirm-without-class')) {
        trigger.removeAttribute('hx-confirm');
    }
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

    const titleSelector = trigger.dataset.titleTarget;
    if (titleSelector) {
        const titleTarget = row && row.querySelector(titleSelector);
        const title = value ? trigger.dataset.titleTrue : trigger.dataset.titleFalse;
        if (!titleTarget || !title) {
            location.reload();
            return;
        }
        titleTarget.title = title;
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

function setRequestPending(trigger, pending) {
    if (!trigger || !trigger.matches || !trigger.matches('button')) return;
    if (pending) {
        trigger.setAttribute('aria-busy', 'true');
    } else {
        trigger.removeAttribute('aria-busy');
    }
}

document.body.addEventListener('htmx:beforeRequest', function (e) {
    clearError();
    setRequestPending(e.detail.elt, true);
    const form = requestForm(e);
    if (!form) return;
    showFormStatus(form, '', false);
    setFormPending(form, true);
});

document.body.addEventListener('htmx:afterRequest', function (e) {
    setRequestPending(e.detail.elt, false);
    if (e.detail.rampartHandled) return;
    e.detail.rampartHandled = true;
    const form = requestForm(e);
    setFormPending(form, false);
    if (!e.detail.successful) return;
    if (form && form.hasAttribute('data-domain-create')) {
        try {
            const created = JSON.parse(e.detail.xhr.responseText);
            if (created && created.id) {
                location.assign('/domains/' + encodeURIComponent(created.id));
                return;
            }
        } catch (_) {
            // The normal response handling below will surface/reload if the
            // endpoint ever stops returning the created DomainView.
        }
    }
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

function copyText(value, button) {
    if (!navigator.clipboard || !navigator.clipboard.writeText) return;
    navigator.clipboard.writeText(value).then(function () {
        const original = button.textContent;
        button.textContent = 'copied';
        window.setTimeout(function () {
            if (button.isConnected) button.textContent = original;
        }, 900);
    }).catch(function () {
        showError('Could not copy to the clipboard.');
    });
}

document.body.addEventListener('click', function (event) {
    const button = event.target.closest('[data-copy-value]');
    if (!button) return;
    copyText(button.dataset.copyValue, button);
});

function dnsRecordElement(record) {
    const row = document.createElement('div');
    row.className = 'dns-record is-' + record.status + (record.priority == null ? '' : ' has-priority');
    row.dataset.recordId = record.id;

    const status = document.createElement('span');
    status.className = 'dns-record-status';
    status.dataset.recordStatus = '';
    status.textContent = record.status === 'pending' ? 'waiting' : record.status;

    const kind = document.createElement('strong');
    kind.className = 'dns-record-kind';
    kind.textContent = record.kind;

    function field(className, label, value, dataName) {
        const wrapper = document.createElement('div');
        wrapper.className = 'dns-record-field ' + className;
        const caption = document.createElement('span');
        caption.textContent = label;
        const code = document.createElement('code');
        code.dataset[dataName] = '';
        code.textContent = value;
        wrapper.append(caption, code);
        return wrapper;
    }

    function copyButton(value) {
        const button = document.createElement('button');
        button.type = 'button';
        button.className = 'secondary dns-copy';
        button.dataset.copyValue = value;
        button.textContent = 'copy';
        return button;
    }

    const fields = [
        status,
        kind,
        field('dns-record-host', 'name', record.host, 'recordHost'),
        copyButton(record.host)
    ];
    if (record.priority != null) {
        fields.push(
            field('dns-record-priority', 'priority', String(record.priority), 'recordPriority'),
            copyButton(String(record.priority))
        );
    }
    fields.push(
        field('dns-record-value', record.value_label, record.provider_value, 'recordValue'),
        copyButton(record.provider_value)
    );
    row.append(...fields);
    return row;
}

function formatUtc(value) {
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return value;
    return date.toISOString().slice(0, 19).replace('T', ' ') + ' UTC';
}

function initializeDomainSetup() {
    const root = document.querySelector('[data-domain-setup]');
    if (!root) return;
    const checkButton = root.querySelector('[data-dns-check]');
    const error = root.querySelector('[data-dns-check-error]');
    let pollTimer = null;

    function renderSetup(setup) {
        root.dataset.state = setup.state;
        const state = document.querySelector('[data-setup-state]');
        state.className = 'domain-state is-' + setup.state;
        state.textContent = setup.state;
        document.querySelector('[data-setup-summary]').textContent = setup.summary;
        root.querySelectorAll('[data-record-group]').forEach(function (group) {
            group.replaceChildren();
        });
        setup.records.forEach(function (record) {
            const group = root.querySelector('[data-record-group="' + record.group + '"]');
            if (group) group.appendChild(dnsRecordElement(record));
        });
        root.querySelector('[data-dkim-pending]').hidden = !setup.dkim_pending;
        const checked = root.querySelector('[data-checked-at]');
        checked.textContent = setup.checked_at ? 'Last checked ' + formatUtc(setup.checked_at) : 'Not checked yet';
    }

    function schedulePoll() {
        window.clearTimeout(pollTimer);
        if (root.dataset.state === 'setup') {
            pollTimer = window.setTimeout(checkDns, 6000);
        }
    }

    async function checkDns() {
        window.clearTimeout(pollTimer);
        error.textContent = '';
        checkButton.disabled = true;
        checkButton.setAttribute('aria-busy', 'true');
        try {
            const response = await fetch('/api/v1/domain/' + encodeURIComponent(root.dataset.domainId) + '/check', {
                method: 'POST',
                headers: {'Accept': 'application/json'}
            });
            if (!response.ok) throw new Error(await response.text() || 'DNS check failed');
            renderSetup(await response.json());
            schedulePoll();
        } catch (requestError) {
            error.textContent = requestError.message;
        } finally {
            checkButton.disabled = false;
            checkButton.removeAttribute('aria-busy');
        }
    }

    checkButton.addEventListener('click', checkDns);
    root.querySelector('[data-copy-all]').addEventListener('click', function (event) {
        const lines = Array.from(root.querySelectorAll('.dns-record')).map(function (row) {
            const fields = [
                row.querySelector('.dns-record-kind').textContent,
                row.querySelector('[data-record-host]').textContent
            ];
            const priority = row.querySelector('[data-record-priority]');
            if (priority) fields.push(priority.textContent);
            fields.push(row.querySelector('[data-record-value]').textContent);
            return fields.join('\t');
        });
        copyText(lines.join('\n'), event.currentTarget);
    });
    if (root.dataset.state !== 'ready') {
        pollTimer = window.setTimeout(checkDns, 700);
    }
}

initializeDomainSetup();

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

let pendingConfirmation = null;

function clearConfirmation() {
    if (!pendingConfirmation) return;
    const pending = pendingConfirmation;
    pendingConfirmation = null;
    if (!pending.trigger.isConnected) return;
    pending.trigger.textContent = pending.label;
    pending.trigger.classList.remove('is-confirming');
    if (pending.title === null) {
        pending.trigger.removeAttribute('title');
    } else {
        pending.trigger.title = pending.title;
    }
    if (pending.ariaLabel === null) {
        pending.trigger.removeAttribute('aria-label');
    } else {
        pending.trigger.setAttribute('aria-label', pending.ariaLabel);
    }
}

function armConfirmation(message, trigger) {
    clearConfirmation();
    const label = trigger.textContent.trim();
    pendingConfirmation = {
        trigger,
        label,
        title: trigger.getAttribute('title'),
        ariaLabel: trigger.getAttribute('aria-label'),
    };
    trigger.textContent = `${label}?`;
    trigger.classList.add('is-confirming');
    trigger.title = message;
    trigger.setAttribute('aria-label', `Confirm: ${message}`);
    trigger.addEventListener('blur', function () {
        if (pendingConfirmation && pendingConfirmation.trigger === trigger) clearConfirmation();
    }, { once: true });
}

function closeUserMenu(restoreFocus) {
    const menu = document.querySelector('.user-menu[open]');
    if (!menu) return;
    menu.removeAttribute('open');
    if (restoreFocus) menu.querySelector('summary').focus();
}

document.addEventListener('click', function (event) {
    const menu = document.querySelector('.user-menu[open]');
    if (menu && !menu.contains(event.target)) closeUserMenu(false);
});

document.addEventListener('keydown', function (event) {
    if (event.key !== 'Escape') return;
    clearConfirmation();
    closeUserMenu(true);
});

document.body.addEventListener('htmx:confirm', function (e) {
    if (!e.detail.question) return;
    e.preventDefault();
    if (pendingConfirmation && pendingConfirmation.trigger === e.detail.elt) {
        clearConfirmation();
        e.detail.issueRequest(true);
        return;
    }
    armConfirmation(e.detail.question, e.detail.elt);
});

document.body.addEventListener('htmx:responseError', function (e) {
    setRequestPending(e.detail.elt, false);
    const xhr = e.detail.xhr;
    if (xhr.status === 401) {
        const next = location.pathname + location.search;
        const loginUrl = next === '/' ? '/login' : `/login?next=${encodeURIComponent(next)}`;
        location.assign(loginUrl);
        return;
    }
    const msg = (xhr.responseText || '').trim() || `${xhr.status} ${xhr.statusText}`;
    const form = requestForm(e);
    setFormPending(form, false);
    recoverFormFromError(form);
    if (!showFormStatus(form, msg, true)) showError(msg);
});
document.body.addEventListener('htmx:sendError', function (e) {
    setRequestPending(e.detail.elt, false);
    const msg = 'Network error — could not reach server.';
    const form = requestForm(e);
    setFormPending(form, false);
    if (!showFormStatus(form, msg, true)) showError(msg);
});
document.body.addEventListener('htmx:timeout', function (e) {
    setRequestPending(e.detail.elt, false);
    const msg = 'Request timed out — try again.';
    const form = requestForm(e);
    setFormPending(form, false);
    if (!showFormStatus(form, msg, true)) showError(msg);
});
