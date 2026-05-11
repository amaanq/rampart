// Reload on success; opt out with data-reload="no".
document.body.addEventListener('htmx:afterRequest', function (e) {
    if (!e.detail.successful) return;
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
    showError(msg);
});
document.body.addEventListener('htmx:sendError', function () {
    showError('Network error — could not reach server.');
});
