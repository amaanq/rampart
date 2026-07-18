function resetNativeForm(form) {
    form.removeAttribute('aria-busy');
    delete form.dataset.submitting;
    const button = form.querySelector('button[type="submit"]');
    if (!button) return;
    if (button.dataset.idleLabel) button.textContent = button.dataset.idleLabel;
    button.disabled = false;
    button.removeAttribute('aria-busy');
}

function submitNativeForm(event) {
    const form = event.currentTarget;
    if (form.dataset.submitting === 'true') {
        event.preventDefault();
        return;
    }

    const button = form.querySelector('button[type="submit"]');
    form.dataset.submitting = 'true';
    form.setAttribute('aria-busy', 'true');
    if (!button) return;
    button.dataset.idleLabel = button.textContent;
    button.textContent = form.dataset.pendingLabel;
    button.disabled = true;
    button.setAttribute('aria-busy', 'true');
}

function nativeForms() {
    return document.querySelectorAll('form[data-native-submit]');
}

document.addEventListener('DOMContentLoaded', function () {
    nativeForms().forEach(function (form) {
        resetNativeForm(form);
        form.addEventListener('submit', submitNativeForm);
    });
});

window.addEventListener('pageshow', function () {
    nativeForms().forEach(resetNativeForm);
});
