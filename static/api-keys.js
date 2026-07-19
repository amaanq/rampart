(function () {
    const root = document.querySelector('[data-api-keys]');
    if (!root) return;

    const form = root.querySelector('[data-api-key-create]');
    const status = root.querySelector('[data-api-key-status]');
    const secret = root.querySelector('[data-api-key-secret]');
    const token = root.querySelector('[data-api-key-token]');
    const copy = root.querySelector('[data-api-key-copy]');

    form.addEventListener('submit', async function (event) {
        event.preventDefault();
        const button = form.querySelector('button[type="submit"]');
        const data = new FormData(form);
        const expiryDays = data.get('expiry_days');
        const expiresAt = expiryDays
            ? new Date(Date.now() + Number(expiryDays) * 86400000).toISOString()
            : null;
        button.disabled = true;
        status.textContent = '';
        status.classList.remove('is-error');
        secret.hidden = true;
        try {
            const response = await fetch('/api/v1/user/api-keys', {
                method: 'POST',
                headers: {'Content-Type': 'application/json', 'Accept': 'application/json'},
                body: JSON.stringify({name: data.get('name'), expires_at: expiresAt})
            });
            if (!response.ok) throw new Error(await response.text() || 'Could not create API key');
            const created = await response.json();
            token.textContent = created.token;
            secret.hidden = false;
            status.textContent = 'Extension key created.';
            form.reset();
        } catch (error) {
            status.textContent = error.message;
            status.classList.add('is-error');
        } finally {
            button.disabled = false;
        }
    });

    copy.addEventListener('click', async function () {
        try {
            await navigator.clipboard.writeText(token.textContent);
            copy.textContent = 'copied';
            window.setTimeout(function () {
                copy.textContent = 'copy';
            }, 900);
        } catch (_) {
            status.textContent = 'Could not copy the token.';
            status.classList.add('is-error');
        }
    });
}());
