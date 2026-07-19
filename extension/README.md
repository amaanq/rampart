# Rampart Email Masking

The Firefox/Chromium extension creates a fresh random Rampart alias and fills it into a signup form.

The toolbar popup works with one-page `activeTab` access. An optional inline helper can detect email fields, including fields added after page load, and place a Rampart button beside them. Enabling that helper explicitly requests access to HTTP(S) pages.

## Development install

Create a restricted browser-extension key from Rampart settings. Its expiry is configurable, including no expiration, and it can be revoked from Rampart or from the extension.

Chrome:

1. Run `./package.sh chrome`.
2. Open `chrome://extensions`, enable developer mode, and load `dist/chrome` unpacked.

Firefox:

1. Run `./package.sh firefox`.
2. Open `about:debugging#/runtime/this-firefox` and load `dist/firefox/manifest.json` as a temporary add-on.

Open the extension settings, enter the Rampart origin and the one-time token, then choose a default domain and mailbox. The token stays in background-only local extension storage; content scripts never receive it.
