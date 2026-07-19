const browserApi = globalThis.browser || globalThis.chrome;
const INLINE_SCRIPT_ID = 'rampart-inline-assistant';
const ANY_DOMAIN = '__any__';
const PAGE_ORIGINS = ['https://*/*', 'http://*/*'];

function normalizeOrigin(value) {
  const url = new URL(value);
  if (!['https:', 'http:'].includes(url.protocol)) {
    throw new Error('Rampart must use HTTPS.');
  }
  if (url.protocol !== 'https:' && !['localhost', '127.0.0.1'].includes(url.hostname)) {
    throw new Error('HTTPS is required except for a local development server.');
  }
  if (url.username || url.password || url.pathname !== '/' || url.search || url.hash) {
    throw new Error('Enter only the Rampart origin, for example https://rampart.example.');
  }
  return url.origin;
}

async function restrictCredentialStorage() {
  if (!browserApi.storage.local.setAccessLevel) return;
  try {
    await browserApi.storage.local.setAccessLevel({accessLevel: 'TRUSTED_CONTEXTS'});
  } catch (_error) {
    // Firefox exposes the same API with the access level as a bare string.
    await browserApi.storage.local.setAccessLevel('TRUSTED_CONTEXTS');
  }
}

function maintainBackgroundState() {
  restrictCredentialStorage().catch(console.error);
  syncInlineAssistant().catch(console.error);
}

async function configuration() {
  const stored = await browserApi.storage.local.get(['instanceOrigin', 'apiToken']);
  if (!stored.instanceOrigin || !stored.apiToken) {
    throw new Error('Connect this extension to Rampart first.');
  }
  return {...stored, instanceOrigin: normalizeOrigin(stored.instanceOrigin)};
}

async function rampartRequest(path, init = {}) {
  const {instanceOrigin, apiToken} = await configuration();
  const url = new URL(path, instanceOrigin);
  if (url.origin !== instanceOrigin) throw new Error('Invalid Rampart API URL.');
  const headers = new Headers(init.headers || {});
  headers.set('Accept', 'application/json');
  headers.set('Authorization', `Bearer ${apiToken}`);
  const response = await fetch(url, {...init, headers, credentials: 'omit'});
  if (response.ok) {
    return response.status === 204 ? null : response.json();
  }
  const contentType = response.headers.get('content-type') || '';
  const body = contentType.includes('application/json')
    ? await response.json()
    : await response.text();
  const message = response.status === 401
    ? 'Rampart token is invalid, expired, or revoked.'
    : typeof body === 'string'
      ? body
      : body.message || body.error || `Rampart returned ${response.status}`;
  const error = new Error(message);
  error.status = response.status;
  throw error;
}

async function bootstrap() {
  const [context, preferences] = await Promise.all([
    rampartRequest('/api/v1/extension/bootstrap'),
    browserApi.storage.local.get(['preferredDomain', 'preferredMailboxId', 'rememberSites'])
  ]);
  return {
    ...context,
    preferences: {
      domain: preferences.preferredDomain || null,
      mailbox_id: preferences.preferredMailboxId || null,
      remember_sites: preferences.rememberSites === true
    }
  };
}

async function resolveAliasDomain(requestedDomain) {
  if (requestedDomain && requestedDomain !== ANY_DOMAIN) return requestedDomain;
  const context = await rampartRequest('/api/v1/extension/bootstrap');
  const ready = context.domains.filter(domain => domain.ready);
  if (!ready.length) throw new Error('Rampart has no ready alias domain.');
  return ready[Math.floor(Math.random() * ready.length)].domain;
}

async function unregisterInlineAssistant() {
  const registered = await browserApi.scripting.getRegisteredContentScripts({ids: [INLINE_SCRIPT_ID]});
  if (registered.length) {
    await browserApi.scripting.unregisterContentScripts({ids: [INLINE_SCRIPT_ID]});
  }
}

async function syncInlineAssistant() {
  const {inlineAssistant} = await browserApi.storage.local.get('inlineAssistant');
  const allowed = await browserApi.permissions.contains({origins: PAGE_ORIGINS});
  await unregisterInlineAssistant();
  if (!inlineAssistant || !allowed) return false;
  await browserApi.scripting.registerContentScripts([{
    id: INLINE_SCRIPT_ID,
    matches: PAGE_ORIGINS,
    js: ['content.js'],
    allFrames: true,
    runAt: 'document_idle'
  }]);
  return true;
}

async function handleMessage(message, sender) {
  if (!message || sender.id !== browserApi.runtime.id) {
    throw new Error('Invalid extension request.');
  }
  switch (message.type) {
    case 'bootstrap':
      return bootstrap();
    case 'createAlias': {
      const preferences = await browserApi.storage.local.get([
        'preferredDomain',
        'preferredMailboxId',
        'rememberSites'
      ]);
      const siteNote = preferences.rememberSites && message.hostname
        ? `Created for ${String(message.hostname).slice(0, 190)}`
        : null;
      const requestedDomain = Object.hasOwn(message, 'domain')
        ? message.domain
        : preferences.preferredDomain;
      const domain = await resolveAliasDomain(requestedDomain);
      const prefix = String(message.prefix || '').trim();
      const path = prefix ? '/api/v1/alias/prefix' : '/api/v1/alias/random';
      return rampartRequest(path, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Idempotency-Key': message.idempotencyKey || crypto.randomUUID()
        },
        body: JSON.stringify({
          domain,
          mailbox_id: message.mailboxId || preferences.preferredMailboxId || null,
          note: message.note || siteNote,
          prefix: prefix || null
        })
      });
    }
    case 'syncInlineAssistant':
      return syncInlineAssistant();
    case 'revokeSelf':
      await rampartRequest('/api/v1/api-key/self', {method: 'DELETE'});
      await browserApi.storage.local.remove('apiToken');
      return null;
    default:
      throw new Error('Unsupported extension request.');
  }
}

browserApi.runtime.onMessage.addListener((message, sender, sendResponse) => {
  handleMessage(message, sender).then(
    value => sendResponse({ok: true, value}),
    error => sendResponse({ok: false, error: error.message, status: error.status || 0})
  );
  return true;
});

browserApi.runtime.onInstalled.addListener(maintainBackgroundState);
browserApi.runtime.onStartup.addListener(maintainBackgroundState);
maintainBackgroundState();
