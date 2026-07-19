const browserApi = globalThis.browser || globalThis.chrome;
const ANY_DOMAIN = '__any__';
const connect = document.querySelector('[data-connect]');
const form = document.querySelector('[data-create]');
const status = document.querySelector('[data-status]');
const result = document.querySelector('[data-result]');
const address = document.querySelector('[data-address]');

function message(payload) {
  return browserApi.runtime.sendMessage(payload).then(response => {
    if (!response || !response.ok) throw new Error(response && response.error || 'Rampart request failed.');
    return response.value;
  });
}

function fillEmail(value) {
  function isEmailInput(input) {
    if (!(input instanceof HTMLInputElement) || input.disabled || input.readOnly) return false;
    const hints = [input.name, input.id, input.placeholder, input.getAttribute('aria-label')]
      .filter(Boolean)
      .join(' ');
    return input.type === 'email'
      || input.autocomplete === 'email'
      || /e-?mail|email address|your email/i.test(hints);
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
  function setValue(target) {
    target.focus();
    if (target.isContentEditable) {
      target.textContent = value;
    } else {
      const prototype = target instanceof HTMLTextAreaElement
        ? HTMLTextAreaElement.prototype
        : HTMLInputElement.prototype;
      const setter = Object.getOwnPropertyDescriptor(prototype, 'value').set;
      setter.call(target, value);
    }
    target.dispatchEvent(new InputEvent('input', {bubbles: true, inputType: 'insertText', data: value}));
    target.dispatchEvent(new Event('change', {bubbles: true}));
  }
  const active = document.activeElement;
  const activeInput = active instanceof HTMLInputElement
    && ['email', 'text', 'search', 'tel', 'url'].includes(active.type)
    && !active.disabled
    && !active.readOnly;
  const acceptsText = activeInput
    || active instanceof HTMLTextAreaElement && !active.disabled && !active.readOnly
    || active && active.isContentEditable;
  const candidates = Array.from(document.querySelectorAll(
    'input[type="email"], input[autocomplete="email"], input[name*="email" i]'
  ));
  const target = acceptsText ? active : candidates.find(element => {
    const bounds = element.getBoundingClientRect();
    return !element.disabled && !element.readOnly && bounds.width > 0 && bounds.height > 0;
  });
  if (!target) return false;
  const targets = new Set([target]);
  if (target instanceof HTMLInputElement) {
    const scope = target.form || target.closest('[role="form"]') || document;
    const emailInputs = [...scope.querySelectorAll('input')].filter(isEmailInput);
    emailInputs.filter(confirmationEmail).forEach(input => targets.add(input));
    if (confirmationEmail(target)) {
      const primary = emailInputs.find(input => !confirmationEmail(input));
      if (primary) targets.add(primary);
    }
    emailInputs.filter(input => targets.has(input)).forEach(setValue);
  } else {
    setValue(target);
  }
  target.focus();
  return true;
}

async function initialize() {
  try {
    const bootstrap = await message({type: 'bootstrap'});
    const domainSelect = form.elements.domain;
    const mailboxSelect = form.elements.mailbox_id;
    const domains = bootstrap.domains.filter(domain => domain.ready);
    const anyDomain = new Option('any ready domain', ANY_DOMAIN);
    anyDomain.dataset.defaultMailboxId = '';
    domainSelect.replaceChildren(
      anyDomain,
      ...domains.map(domain => {
        const option = new Option(domain.domain, domain.domain);
        option.dataset.defaultMailboxId = domain.default_mailbox_id || '';
        return option;
      })
    );
    mailboxSelect.replaceChildren(...bootstrap.mailboxes.map(mailbox =>
      new Option(mailbox.display_name ? `${mailbox.display_name} — ${mailbox.email}` : mailbox.email, mailbox.id)
    ));
    if (!domains.length || !bootstrap.mailboxes.length) {
      throw new Error('Rampart needs a ready domain and a verified mailbox.');
    }
    if (bootstrap.preferences.domain
      && [...domainSelect.options].some(option => option.value === bootstrap.preferences.domain)) {
      domainSelect.value = bootstrap.preferences.domain;
    }
    if (bootstrap.preferences.mailbox_id
      && [...mailboxSelect.options].some(option => Number(option.value) === bootstrap.preferences.mailbox_id)) {
      mailboxSelect.value = String(bootstrap.preferences.mailbox_id);
    }
    const selectDefaultMailbox = () => {
      const id = domainSelect.selectedOptions[0].dataset.defaultMailboxId;
      if (id && Array.from(mailboxSelect.options).some(option => option.value === id)) {
        mailboxSelect.value = id;
      }
    };
    domainSelect.addEventListener('change', async () => {
      selectDefaultMailbox();
      await browserApi.storage.local.set({
        preferredDomain: domainSelect.value,
        preferredMailboxId: Number(mailboxSelect.value)
      });
    });
    mailboxSelect.addEventListener('change', () => browserApi.storage.local.set({
      preferredMailboxId: Number(mailboxSelect.value)
    }));
    if (!bootstrap.preferences.mailbox_id) selectDefaultMailbox();
    form.hidden = false;
  } catch (error) {
    connect.hidden = false;
    status.textContent = error.message;
  }
}

form.addEventListener('submit', async event => {
  event.preventDefault();
  const button = form.querySelector('button[type="submit"]');
  button.disabled = true;
  status.textContent = 'Creating alias…';
  result.hidden = true;
  try {
    const [tab] = await browserApi.tabs.query({active: true, currentWindow: true});
    let hostname = null;
    try {
      hostname = new URL(tab.url).hostname;
    } catch (_error) {
      // Restricted browser pages have no usable site hostname.
    }
    const alias = await message({
      type: 'createAlias',
      idempotencyKey: crypto.randomUUID(),
      hostname,
      domain: form.elements.domain.value,
      mailboxId: Number(form.elements.mailbox_id.value),
      prefix: form.elements.prefix.value.trim(),
      note: form.elements.note.value.trim()
    });
    address.textContent = alias.address;
    result.hidden = false;
    const injected = await browserApi.scripting.executeScript({
      target: {tabId: tab.id},
      func: fillEmail,
      args: [alias.address]
    });
    status.textContent = injected[0] && injected[0].result
      ? 'Alias created and filled.'
      : 'Alias created. No email field was available, so copy it below.';
  } catch (error) {
    status.textContent = error.message;
  } finally {
    button.disabled = false;
  }
});

document.querySelectorAll('[data-open-options]').forEach(button => {
  button.addEventListener('click', () => browserApi.runtime.openOptionsPage());
});

document.querySelector('[data-copy]').addEventListener('click', async event => {
  await navigator.clipboard.writeText(address.textContent);
  event.currentTarget.textContent = 'copied';
});

initialize();
