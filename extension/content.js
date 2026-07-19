'use strict';

(function () {
  const browserApi = globalThis.browser || globalThis.chrome;
  const ANY_DOMAIN = '__any__';
  const seen = new WeakSet();
  const watched = new WeakSet();
  const EMAIL_HINT = /e-?mail|email address|your email/i;

  function message(payload) {
    return browserApi.runtime.sendMessage(payload).then(response => {
      if (!response || !response.ok) {
        throw new Error(response && response.error || 'Rampart request failed.');
      }
      return response.value;
    });
  }

  function isVisible(input) {
    const box = input.getBoundingClientRect();
    const style = getComputedStyle(input);
    return box.width > 30
      && box.height > 12
      && style.visibility !== 'hidden'
      && style.display !== 'none';
  }

  function isEmailInput(input) {
    if (!(input instanceof HTMLInputElement) || input.disabled || input.readOnly) return false;
    const hints = [
      input.name,
      input.id,
      input.placeholder,
      input.getAttribute('aria-label')
    ].filter(Boolean).join(' ');
    return input.type === 'email' || input.autocomplete === 'email' || EMAIL_HINT.test(hints);
  }

  function confirmationEmail(input) {
    const label = [...(input.labels || [])].map(item => item.textContent).join(' ');
    const hints = [
      input.name,
      input.id,
      input.placeholder,
      input.getAttribute('aria-label'),
      label
    ].filter(Boolean).join(' ');
    return /(?:confirm|verify|re[ -]?enter|retype|repeat).{0,30}e-?mail|e-?mail.{0,30}(?:confirm|verify|again|repeat)|e-?mail[_-]?2/i.test(hints);
  }

  function fill(input, address) {
    const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set;
    const scope = input.form || input.closest('[role="form"]') || document;
    const emailInputs = [...scope.querySelectorAll('input')]
      .filter(candidate => isEmailInput(candidate) && isVisible(candidate));
    const targets = new Set([input]);
    emailInputs.filter(confirmationEmail).forEach(candidate => targets.add(candidate));
    if (confirmationEmail(input)) {
      const primary = emailInputs.find(candidate => !confirmationEmail(candidate));
      if (primary) targets.add(primary);
    }
    for (const target of emailInputs.filter(candidate => targets.has(candidate))) {
      setter.call(target, address);
      target.dispatchEvent(new InputEvent('input', {
        bubbles: true,
        inputType: 'insertText',
        data: address
      }));
      target.dispatchEvent(new Event('change', { bubbles: true }));
    }
    input.focus();
  }

  function option(value, text, selected) {
    const element = document.createElement('option');
    element.value = String(value);
    element.textContent = text;
    element.selected = selected;
    return element;
  }

  function attach(input) {
    if (seen.has(input) || !isVisible(input)) return;
    seen.add(input);

    const host = document.createElement('span');
    host.style.cssText = 'all:initial;position:static;display:inline;z-index:2147483647';
    input.insertAdjacentElement('afterend', host);
    const root = host.attachShadow({ mode: 'closed' });
    root.innerHTML = `<style>
      :host{all:initial}.launch{position:fixed;z-index:2147483646;width:28px;height:28px;padding:0;border:0;border-radius:5px;background:#3b2767;color:#fff;font:700 17px system-ui;line-height:28px;text-align:center;cursor:pointer;box-shadow:0 1px 5px #0005}.launch:hover{background:#51368b}.launch:focus-visible{outline:2px solid #8f70c1;outline-offset:1px}
      .menu{position:fixed;z-index:2147483647;width:290px;box-sizing:border-box;padding:14px;border:1px solid #75658a;border-radius:10px;background:#fff;color:#211b2b;box-shadow:0 8px 28px #0006;font:14px system-ui}.menu[hidden]{display:none}.title{font-weight:750;margin:0 0 10px}.close{float:right;border:0;background:transparent;color:#54475f;font-size:20px;cursor:pointer;margin:-6px}.menu label{display:block;font-weight:600;margin:9px 0}.menu select,.menu input{display:block;box-sizing:border-box;width:100%;margin-top:4px;padding:7px;border:1px solid #93879d;border-radius:5px;background:#fff;color:#211b2b}.create{width:100%;margin-top:12px;border:0;border-radius:6px;padding:8px;background:#3b2767;color:#fff;font-weight:700;cursor:pointer}.create:disabled{opacity:.6}.status{display:block;color:#9b1c1c;margin-top:7px;font-size:12px}.hint{color:#675d70;font-size:12px;margin:5px 0}
      @media(prefers-color-scheme:dark){.menu{background:#241e2d;color:#eee}.menu select,.menu input{background:#17131d;color:#eee}}
    </style><button class="launch" type="button" title="Mask with Rampart" aria-label="Mask email with Rampart">R</button>
    <div class="menu" hidden role="dialog" aria-label="Create Rampart alias"><button class="close" type="button" aria-label="Close">×</button><p class="title">Mask with Rampart</p><label>Domain<select class="domain"></select></label><label>Forward to<select class="mailbox"></select></label><label>Prefix<input class="prefix" type="text" maxlength="64" autocomplete="off" spellcheck="false" placeholder="Blank generates randomly"></label><p class="hint">Creates an address and fills this field.</p><button class="create" type="button">Create and fill</button><span class="status" role="status"></span></div>`;

    const launch = root.querySelector('.launch');
    const menu = root.querySelector('.menu');
    const domain = root.querySelector('.domain');
    const mailbox = root.querySelector('.mailbox');
    const prefix = root.querySelector('.prefix');
    const create = root.querySelector('.create');
    const status = root.querySelector('.status');
    let context;

    function selectDefaultMailbox() {
      const selectedDomain = context.domains.find(item => item.domain === domain.value);
      const preferred = context.preferences.mailbox_id;
      const mailboxId = selectedDomain?.default_mailbox_id || preferred;
      if (mailboxId && [...mailbox.options].some(item => Number(item.value) === mailboxId)) {
        mailbox.value = String(mailboxId);
      }
    }

    function positionIcon() {
      const box = input.getBoundingClientRect();
      const size = 28;
      launch.style.left = `${Math.max(0, box.right - size - 6)}px`;
      launch.style.top = `${Math.max(0, box.top + (box.height - size) / 2)}px`;
      launch.hidden = box.bottom < 0
        || box.top > innerHeight
        || box.right < 0
        || box.left > innerWidth;
    }

    function positionMenu() {
      positionIcon();
      const box = input.getBoundingClientRect();
      const menuHeight = 340;
      const top = innerHeight - box.bottom > menuHeight
        ? box.bottom + 6
        : Math.max(8, box.top - menuHeight - 6);
      const left = Math.max(8, Math.min(box.right - 290, innerWidth - 298));
      menu.style.top = `${top}px`;
      menu.style.left = `${left}px`;
    }

    root.querySelector('.close').addEventListener('click', () => {
      menu.hidden = true;
    });
    domain.addEventListener('change', selectDefaultMailbox);
    launch.addEventListener('click', async () => {
      positionMenu();
      menu.hidden = false;
      status.textContent = 'Loading domains…';
      try {
        context = await message({ type: 'bootstrap' });
        const domains = context.domains.filter(item => item.ready);
        if (!domains.length || !context.mailboxes.length) {
          throw new Error('Rampart needs a ready domain and verified mailbox.');
        }
        domain.replaceChildren(
          option(
            ANY_DOMAIN,
            'any ready domain',
            !context.preferences.domain || context.preferences.domain === ANY_DOMAIN
          ),
          ...domains.map(item => option(
            item.domain,
            item.domain,
            item.domain === context.preferences.domain
          ))
        );
        mailbox.replaceChildren(...context.mailboxes.map(item => option(
          item.id,
          item.display_name ? `${item.display_name} — ${item.email}` : item.email,
          item.id === context.preferences.mailbox_id
        )));
        selectDefaultMailbox();
        status.textContent = '';
        create.focus();
      } catch (error) {
        status.textContent = error.message;
      }
    });
    create.addEventListener('click', async () => {
      create.disabled = true;
      status.textContent = 'Creating alias…';
      try {
        const alias = await message({
          type: 'createAlias',
          idempotencyKey: crypto.randomUUID(),
          hostname: location.hostname,
          domain: domain.value,
          mailboxId: Number(mailbox.value),
          prefix: prefix.value.trim()
        });
        fill(input, alias.address);
        menu.hidden = true;
      } catch (error) {
        status.textContent = error.message;
      } finally {
        create.disabled = false;
      }
    });

    positionIcon();
    const reposition = () => {
      positionIcon();
      if (!menu.hidden) positionMenu();
    };
    addEventListener('resize', reposition, { passive: true });
    addEventListener('scroll', reposition, { passive: true, capture: true });
    new ResizeObserver(reposition).observe(input);
  }

  function scan(root = document) {
    const inputs = root.matches?.('input') ? [root] : root.querySelectorAll('input');
    inputs.forEach(input => {
      if (!isEmailInput(input) || confirmationEmail(input) || watched.has(input)) return;
      watched.add(input);
      input.addEventListener('focus', () => attach(input));
      attach(input);
    });
  }

  scan();
  new MutationObserver(records => records.forEach(record => record.addedNodes.forEach(node => {
    if (node.nodeType === Node.ELEMENT_NODE) scan(node);
  }))).observe(document.documentElement, { childList: true, subtree: true });
}());
