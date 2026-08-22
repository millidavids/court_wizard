#!/usr/bin/env node
/**
 * Mint the Steam refresh token that steam-promote.yml uses to post the
 * patch-notes event. Run this by hand, on your own machine, and paste the
 * result into the STEAM_WEB_REFRESH_TOKEN repo secret.
 *
 *   npm install --no-save steam-session qrcode-terminal
 *   node scripts/steam_refresh_token.mjs
 *
 * Scan the QR with the Steam Mobile app and approve. Nothing is written to
 * disk — the token is printed once and that is the only copy.
 *
 * WHY MobileApp, and not the platform you would expect
 * ----------------------------------------------------
 * The posting script exchanges this refresh token for a web access token via
 * IAuthenticationService/GenerateAccessTokenForApp, and that RPC only accepts
 * MobileApp-platform tokens. A WebBrowser token — the intuitive choice for a
 * web POST — is refused, and SteamClient tokens have been refused since
 * 2025-04-30. The refusal is not an HTTP error (see below), so a token minted
 * with the wrong platform fails silently at post time rather than here.
 *
 * WHAT THIS TOKEN IS
 * ------------------
 * A full web session for the account, valid for months, stored in a public
 * repository's secrets. It is the most dangerous credential in this repo. Mint
 * it from the account with the narrowest Steamworks rights that can still post
 * hub announcements. Revoke it from Steam → Account details → authorised
 * devices if it ever leaks.
 */

import { LoginSession, EAuthTokenPlatformType } from 'steam-session';
import qrcode from 'qrcode-terminal';

const TOKEN_URL =
  'https://api.steampowered.com/IAuthenticationService/GenerateAccessTokenForApp/v1/';

function die(message) {
  console.error(`\nerror: ${message}`);
  process.exit(1);
}

/** Decodes a JWT payload without verifying it — we only want `exp`. */
function jwtPayload(token) {
  const parts = token.split('.');
  if (parts.length !== 3) return null;
  try {
    return JSON.parse(Buffer.from(parts[1], 'base64url').toString('utf8'));
  } catch {
    return null;
  }
}

/**
 * Proves the token can actually do the one thing CI needs, before you paste it
 * anywhere. Worth the extra call: this endpoint reports failure as HTTP 200
 * with an `x-eresult` header and an empty body, so the only way to know a token
 * is usable is to use it and read that header.
 */
async function verify(refreshToken, steamId) {
  const response = await fetch(TOKEN_URL, {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body: new URLSearchParams({ refresh_token: refreshToken, steamid: steamId }),
  });

  const eresult = response.headers.get('x-eresult');
  const body = await response.json().catch(() => ({}));
  const accessToken = body?.response?.access_token;

  if (eresult !== '1' || !accessToken) {
    die(
      `the token was issued but cannot mint an access token ` +
        `(HTTP ${response.status}, x-eresult ${eresult ?? 'absent'}).\n` +
        `       x-eresult 15 is AccessDenied, which here almost always means the\n` +
        `       platform type was wrong. This script asks for MobileApp; if you\n` +
        `       edited that, put it back.`,
    );
  }

  console.log(`  verified: it mints an access token (x-eresult 1).`);
}

const session = new LoginSession(EAuthTokenPlatformType.MobileApp);
// The default window is short for "walk over to your phone and unlock it".
session.loginTimeout = 120_000;

session.on('remoteInteraction', () => {
  console.log('QR scanned — now approve it in the app.');
});

session.on('authenticated', async () => {
  const steamId = session.steamID.getSteamID64();
  const token = session.refreshToken;

  console.log(`\nSigned in as ${steamId}.`);
  await verify(token, steamId);

  const payload = jwtPayload(token);
  if (payload?.exp) {
    const expires = new Date(payload.exp * 1000);
    const days = Math.round((expires - Date.now()) / 86_400_000);
    console.log(`  expires:  ${expires.toISOString().slice(0, 10)} (~${days} days)`);
  }

  console.log(`
Set these two repository secrets:

  STEAM_WEB_REFRESH_TOKEN
${token}

  STEAM_PUBLISH_STEAMID
${steamId}

STEAM_PUBLISH_STEAMID is probably already set — it is the same account that
receives the promotion prompt. Re-run this script when the token expires; the
posting step warns for 30 days beforehand.
`);
  process.exit(0);
});

session.on('timeout', () => die('the login request timed out — run it again.'));
session.on('error', (err) => die(`login failed: ${err.message}`));

const started = await session.startWithQR();

console.log('Scan this with the Steam Mobile app:\n');
qrcode.generate(started.qrChallengeUrl, { small: true });
console.log('\nWaiting for approval…');
