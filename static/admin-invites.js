(function () {
    const root = document.querySelector('[data-admin-invites]');
    if (!root) return;

    const form = root.querySelector('[data-invite-create]');
    const status = root.querySelector('[data-invite-status]');
    const secret = root.querySelector('[data-invite-secret]');
    const urlInput = root.querySelector('[data-invite-url]');
    const copy = root.querySelector('[data-invite-copy]');
    const list = root.querySelector('[data-invite-list]');

    function addPendingInvite(invite) {
        const empty = list.querySelector('[data-invite-empty]');
        if (empty) empty.remove();

        const row = document.createElement('tr');
        const emailCell = document.createElement('td');
        emailCell.textContent = invite.email || 'any email';
        if (!invite.email) emailCell.className = 'hint';

        const expiryCell = document.createElement('td');
        const expiry = document.createElement('time');
        expiry.className = 'timestamp';
        expiry.dateTime = invite.expires_at;
        expiry.textContent = new Date(invite.expires_at).toLocaleString();
        expiryCell.appendChild(expiry);

        const actionCell = document.createElement('td');
        const revoke = document.createElement('button');
        revoke.type = 'button';
        revoke.className = 'danger';
        revoke.dataset.inviteRevoke = '';
        revoke.dataset.inviteId = invite.id;
        revoke.textContent = 'revoke';
        actionCell.appendChild(revoke);

        row.append(emailCell, expiryCell, actionCell);
        list.prepend(row);
    }

    form.addEventListener('submit', async function (event) {
        event.preventDefault();
        const button = form.querySelector('button[type="submit"]');
        const email = new FormData(form).get('email').trim();
        button.disabled = true;
        status.textContent = '';
        status.classList.remove('is-error');
        secret.hidden = true;
        try {
            const response = await fetch('/api/v1/admin/invites', {
                method: 'POST',
                headers: {'Content-Type': 'application/json', 'Accept': 'application/json'},
                body: JSON.stringify({email: email || null})
            });
            if (!response.ok) throw new Error(await response.text() || 'Could not create invite');
            const invite = await response.json();
            urlInput.value = invite.url;
            secret.hidden = false;
            status.textContent = email && invite.delivered
                ? 'Invitation emailed.'
                : email
                    ? 'Invitation created, but email delivery failed. Copy the link below.'
                    : 'Invitation created. Copy the link below.';
            addPendingInvite(invite);
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
            await navigator.clipboard.writeText(urlInput.value);
            copy.textContent = 'copied';
            window.setTimeout(function () {
                copy.textContent = 'copy';
            }, 900);
        } catch (_) {
            urlInput.select();
            status.textContent = 'Could not copy the link. Copy it from the field instead.';
            status.classList.add('is-error');
        }
    });

    root.addEventListener('click', async function (event) {
        const button = event.target.closest('[data-invite-revoke]');
        if (!button || !window.confirm('revoke this invite?')) return;

        button.disabled = true;
        try {
            const response = await fetch('/api/v1/admin/invites/' + encodeURIComponent(button.dataset.inviteId), {
                method: 'DELETE'
            });
            if (!response.ok) throw new Error(await response.text() || 'Could not revoke invite');
            button.closest('tr').remove();
            if (!list.querySelector('tr')) {
                const empty = document.createElement('tr');
                empty.dataset.inviteEmpty = '';
                const cell = document.createElement('td');
                cell.colSpan = 3;
                cell.className = 'empty-value';
                cell.textContent = 'no pending invites';
                empty.appendChild(cell);
                list.appendChild(empty);
            }
            status.textContent = 'Invitation revoked.';
            status.classList.remove('is-error');
        } catch (error) {
            status.textContent = error.message;
            status.classList.add('is-error');
            button.disabled = false;
        }
    });
}());
