'use strict';

const browserApi = globalThis.browser || globalThis.chrome;
const ANY_DOMAIN = '__any__';
const PAGE_ORIGINS = ['https://*/*', 'http://*/*'];
const form = document.querySelector('[data-options]');
const choices = document.querySelector('[data-choices]');
const inlineAssistant = document.querySelector('[data-inline-assistant]');
const rememberSites = document.querySelector('[data-remember-sites]');
const status = document.querySelector('[data-status]');
const openSettings = document.querySelector('[data-open-settings]');
let currentContext;
let connected = false;

function normalizeOrigin(value) {
  const url = new URL(value);
  if (!['https:', 'http:'].includes(url.protocol)) throw new Error('Use an HTTPS Rampart URL.');
  if (url.protocol !== 'https:' && !['localhost', '127.0.0.1'].includes(url.hostname)) {
    throw new Error('HTTPS is required except for a local development server.');
  }
  if (url.username || url.password || url.pathname !== '/' || url.search || url.hash) {
    throw new Error('Enter only the Rampart origin, for example https://rampart.example.');
  }
  return url.origin;
}

function originPattern(origin) {
  return `${origin}/*`;
}

function message(payload) {
  return browserApi.runtime.sendMessage(payload).then(response => {
    if (!response || !response.ok) {
      throw new Error(response && response.error || 'Rampart request failed.');
    }
    return response.value;
  });
}

function option(value, text, selected) {
  const element = document.createElement('option');
  element.value = String(value);
  element.textContent = text;
  element.selected = selected;
  return element;
}

function selectDefaultMailbox(context) {
  const domain = context.domains.find(item => item.domain === form.elements.domain.value);
  const id = domain && domain.default_mailbox_id;
  if (id && [...form.elements.mailbox_id.options].some(item => Number(item.value) === id)) {
    form.elements.mailbox_id.value = String(id);
  }
}

async function renderContext(context) {
  currentContext = context;
  connected = true;
  const stored = await browserApi.storage.local.get(['preferredDomain', 'preferredMailboxId']);
  const domains = context.domains.filter(item => item.ready);
  if (!domains.length) throw new Error('No domain is ready for alias creation.');
  if (!context.mailboxes.length) throw new Error('No verified mailbox is available.');
  form.elements.domain.replaceChildren(
    option(
      ANY_DOMAIN,
      'any ready domain',
      !stored.preferredDomain || stored.preferredDomain === ANY_DOMAIN
    ),
    ...domains.map(item => option(
      item.domain,
      item.domain,
      item.domain === stored.preferredDomain
    ))
  );
  form.elements.mailbox_id.replaceChildren(...context.mailboxes.map(item => option(
    item.id,
    item.display_name ? `${item.display_name} — ${item.email}` : item.email,
    item.id === stored.preferredMailboxId
  )));
  if (!stored.preferredMailboxId) selectDefaultMailbox(context);
  choices.hidden = false;
  openSettings.href = `${normalizeOrigin(form.elements.instance.value)}/settings`;
  openSettings.hidden = false;
  return context;
}

async function initialize() {
  const stored = await browserApi.storage.local.get([
    'instanceOrigin',
    'inlineAssistant',
    'rememberSites'
  ]);
  if (stored.instanceOrigin) {
    form.elements.instance.value = stored.instanceOrigin;
    try {
      await renderContext(await message({ type: 'bootstrap' }));
      status.textContent = 'Connected.';
    } catch (error) {
      status.textContent = error.message;
    }
  }
  inlineAssistant.checked = stored.inlineAssistant === undefined
    ? true
    : stored.inlineAssistant === true;
  rememberSites.checked = stored.rememberSites === true;
}

form.addEventListener('submit', async event => {
  event.preventDefault();
  const button = form.querySelector('button[type="submit"]');
  button.disabled = true;
  status.textContent = 'Connecting…';
  let previous;
  let credentialsChanged = false;
  try {
    const instanceOrigin = normalizeOrigin(form.elements.instance.value.trim());
    const origins = inlineAssistant.checked
      ? [...PAGE_ORIGINS, originPattern(instanceOrigin)]
      : [originPattern(instanceOrigin)];
    const granted = await browserApi.permissions.request({ origins });
    if (!granted) throw new Error('Host permission is required to contact Rampart.');
    previous = await browserApi.storage.local.get(['instanceOrigin', 'apiToken']);
    const apiToken = form.elements.token.value.trim() || previous.apiToken;
    if (!apiToken) throw new Error('Enter an extension token.');
    await browserApi.storage.local.set({ instanceOrigin, apiToken });
    credentialsChanged = true;
    const context = await message({ type: 'bootstrap' });
    await renderContext(context);
    await browserApi.storage.local.set({ inlineAssistant: inlineAssistant.checked });
    const inlineEnabled = await message({ type: 'syncInlineAssistant' });
    if (inlineAssistant.checked && !inlineEnabled) {
      throw new Error('Firefox did not grant inline website access.');
    }
    form.elements.token.value = '';
    status.textContent = inlineAssistant.checked
      ? `Connected. ${context.alias_count}/${context.alias_limit} aliases used. Reload open pages once.`
      : `Connected. ${context.alias_count}/${context.alias_limit} aliases used.`;
  } catch (error) {
    if (credentialsChanged && previous.instanceOrigin && previous.apiToken) {
      await browserApi.storage.local.set(previous);
    } else if (credentialsChanged) {
      await browserApi.storage.local.remove(['instanceOrigin', 'apiToken']);
    }
    status.textContent = error.message;
  } finally {
    button.disabled = false;
  }
});

form.elements.domain.addEventListener('change', async () => {
  selectDefaultMailbox(currentContext);
  await browserApi.storage.local.set({
    preferredDomain: form.elements.domain.value,
    preferredMailboxId: Number(form.elements.mailbox_id.value)
  });
});

form.elements.mailbox_id.addEventListener('change', () => browserApi.storage.local.set({
  preferredMailboxId: Number(form.elements.mailbox_id.value)
}));

rememberSites.addEventListener('change', () => browserApi.storage.local.set({
  rememberSites: rememberSites.checked
}));

inlineAssistant.addEventListener('change', async () => {
  if (!connected) {
    status.textContent = inlineAssistant.checked
      ? 'Inline website access will be requested when you save.'
      : 'Toolbar-only mode selected.';
    return;
  }
  inlineAssistant.disabled = true;
  try {
    if (inlineAssistant.checked) {
      const granted = await browserApi.permissions.request({ origins: PAGE_ORIGINS });
      if (!granted) throw new Error('Website access was not granted.');
    }
    await browserApi.storage.local.set({ inlineAssistant: inlineAssistant.checked });
    await message({ type: 'syncInlineAssistant' });
    if (!inlineAssistant.checked) {
      await browserApi.permissions.remove({ origins: PAGE_ORIGINS });
    }
    status.textContent = inlineAssistant.checked
      ? 'Inline email helper enabled. Reload open pages once.'
      : 'Inline email helper disabled.';
  } catch (error) {
    inlineAssistant.checked = !inlineAssistant.checked;
    await browserApi.storage.local.set({ inlineAssistant: inlineAssistant.checked });
    status.textContent = error.message;
  } finally {
    inlineAssistant.disabled = false;
  }
});

async function disconnect() {
  const { instanceOrigin } = await browserApi.storage.local.get('instanceOrigin');
  await browserApi.storage.local.set({ inlineAssistant: false });
  await message({ type: 'syncInlineAssistant' });
  await browserApi.permissions.remove({ origins: PAGE_ORIGINS });
  if (instanceOrigin) {
    await browserApi.permissions.remove({ origins: [originPattern(instanceOrigin)] });
  }
  await browserApi.storage.local.remove([
    'instanceOrigin',
    'apiToken',
    'preferredDomain',
    'preferredMailboxId',
    'inlineAssistant',
    'rememberSites'
  ]);
  form.reset();
  connected = false;
  choices.hidden = true;
  openSettings.hidden = true;
}

document.querySelector('[data-disconnect]').addEventListener('click', async () => {
  await disconnect();
  status.textContent = 'Disconnected. The key remains valid until you revoke it in Rampart.';
});

document.querySelector('[data-revoke]').addEventListener('click', async () => {
  try {
    await message({ type: 'revokeSelf' });
    await disconnect();
    status.textContent = 'Key revoked and extension disconnected.';
  } catch (error) {
    status.textContent = error.message;
  }
});

initialize();
