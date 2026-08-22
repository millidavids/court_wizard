#!/usr/bin/env python3
"""Post a release's patch notes to the Steam community hub as a partner event.

`steam-promote.yml` runs this from the "live but not announced" arm, so the post
only goes out once the build is genuinely live on Steam's default branch, and the
`steam-announced/v<version>` tag stops it repeating.

THERE IS NO SUPPORTED API FOR THIS
----------------------------------
Valve's event tools are UI-only: `ISteamNews` is read-only, `app_build.vdf` has
no patch-notes key, and the documented automation story for events points the
other way (Steam emits an RSS feed of your events, it does not ingest one). So
this drives the same undocumented endpoint the Steamworks event editor itself
calls, with a logged-in web session. Expect it to break when Valve changes the
editor; that is why the caller treats a failure as non-fatal and why the BBCode
is still published as a release artifact for pasting by hand.

The request contract was captured from the live editor on 2026-08-22 and is
documented in steam/README.md. Three things about it are easy to get wrong:

  * Publishing is a SECOND request. The create call saves a hidden, unpublished
    event; `bPublish` carries no content and only flips it live. Replaying just
    the create leaves an invisible draft that no feed will ever show, so the
    idempotency check would never see it and every run would make another one.
  * The endpoint is the `/gid/<clanid64>/` form, and redirects must not be
    followed — a redirect turns POST into GET and silently drops the body.
  * `GenerateAccessTokenForApp` reports failure as HTTP 200 with an empty body
    and an `x-eresult` header. Checking the status code detects nothing; see
    steam_access_token().

Usage:
    steam_post_announcement.py --version 1.0.38 [--changelog docs/CHANGELOG.md]
                               [--app-id 4550880] [--draft] [--dry-run]

Environment:
    STEAM_WEB_REFRESH_TOKEN  MobileApp-platform refresh token; scripts/steam_refresh_token.mjs mints it
    STEAM_PUBLISH_STEAMID    SteamID64 that token belongs to
"""

import argparse
import base64
import json
import os
import re
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request

# The Court Wizard community hub's clan id. Find another app's with:
#   curl -sSL "https://steamcommunity.com/games/<appid>" | grep -o '/gid/[0-9]*'
CLAN_ID = "103582791475642999"

TOKEN_URL = "https://api.steampowered.com/IAuthenticationService/GenerateAccessTokenForApp/v1/"
EVENTS_URL = "https://store.steampowered.com/events/ajaxgetpartnereventspageable/"

# Small Update / Patch Notes. 13 is Regular Update, 14 is Major Update.
EVENT_TYPE_PATCH_NOTES = 12

# Editor limits. The body has no stated cap; 6,977 characters posted fine.
MAX_TITLE = 80
MAX_SUBTITLE = 120
MAX_SUMMARY = 180

# Steam's localisation arrays are 32 slots wide; index 0 is English.
LANG_SLOTS = 32

RENDER = os.path.join(os.path.dirname(os.path.abspath(__file__)), "changelog_to_bbcode.sh")


def fail(message: str):
    print(f"::error::{message}", file=sys.stderr)
    sys.exit(1)


def render(changelog: str, version: str, mode: str) -> str:
    """Shells out to changelog_to_bbcode.sh so BBCode has exactly one owner.

    That script is also what release.yml publishes as the paste-by-hand
    artifact, so the automatic post and the manual fallback cannot drift.
    """
    result = subprocess.run(
        [RENDER, "--version", version, mode, changelog],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        fail(f"changelog_to_bbcode.sh {mode} failed: {result.stderr.strip()}")
    return result.stdout.strip("\n")


def fit(text: str, limit: int) -> str:
    """Trims to `limit`, preferring to end on a sentence boundary.

    A hard character trim on a changelog description reads badly — it lands
    mid-clause. Packing whole sentences keeps the field readable and only falls
    back to a word-boundary cut when even the first sentence is too long.
    """
    if len(text) <= limit:
        return text
    out = ""
    for sentence in re.findall(r"[^.!?]+[.!?]*\s*", text):
        if len(out) + len(sentence.rstrip()) > limit:
            break
        out += sentence
    out = out.strip()
    if out:
        return out
    return text[: limit - 1].rsplit(" ", 1)[0].rstrip(" ,;:—–-") + "…"


def steam_access_token(refresh_token: str, steamid: str) -> str:
    """Exchanges a refresh token for a web access token.

    This call NEVER returns a non-2xx status. A dead token, or one minted with
    the wrong platform type, comes back as HTTP 200 with the body `{"response":{}}`
    and the real outcome in the `x-eresult` header. `--fail`-style status checks
    sail straight past it and the caller ends up building
    `steamLoginSecure=<steamid>||` — a syntactically valid, entirely anonymous
    cookie that posts nothing. So assert on the header, on the value, and on the
    shape before this token is allowed anywhere near a request.
    """
    request = urllib.request.Request(
        TOKEN_URL,
        data=urllib.parse.urlencode(
            {"refresh_token": refresh_token, "steamid": steamid}
        ).encode(),
        headers={"Content-Type": "application/x-www-form-urlencoded"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            eresult = response.headers.get("x-eresult")
            payload = json.loads(response.read().decode("utf-8") or "{}")
    except urllib.error.URLError as err:
        fail(f"{TOKEN_URL} unreachable: {err}")

    if eresult != "1":
        hint = ""
        if eresult == "15":
            hint = (
                " (AccessDenied — the refresh token is expired, revoked, or was"
                " minted with the wrong platform type. It must be MobileApp;"
                " re-run scripts/steam_refresh_token.mjs.)"
            )
        fail(f"GenerateAccessTokenForApp returned x-eresult {eresult}{hint}")

    token = (payload.get("response") or {}).get("access_token") or ""
    if not token:
        fail("GenerateAccessTokenForApp returned x-eresult 1 but no access_token")
    if token.count(".") != 2:
        fail("access_token is not a JWT — refusing to build a session cookie from it")

    print(f"::add-mask::{token}")
    warn_if_expiring(refresh_token)
    return token


def warn_if_expiring(refresh_token: str, days: int = 30):
    """Reads `exp` out of the refresh token rather than assuming a lifetime.

    Refresh-token lifetimes are not a fixed constant across platforms, so the
    token itself is the only honest source.
    """
    try:
        part = refresh_token.split(".")[1]
        part += "=" * (-len(part) % 4)
        exp = json.loads(base64.urlsafe_b64decode(part)).get("exp")
    except Exception:
        return
    if not exp:
        return
    left = int((exp - time.time()) // 86400)
    if left <= days:
        print(
            f"::warning title=Steam token expiring::STEAM_WEB_REFRESH_TOKEN expires in "
            f"{left} day(s). Re-run scripts/steam_refresh_token.mjs and update the secret."
        )


def published_events(app_id: str) -> list:
    """Public, keyless, and — unlike ISteamNews/GetNewsForApp — not edge-cached.

    GetNewsForApp sits behind a one-hour Akamai TTL and mixes third-party press
    into the same feed, which makes it useless both for "did I already post
    this?" and for "did the post land?".
    """
    url = f"{EVENTS_URL}?" + urllib.parse.urlencode(
        {"clan_accountid": 0, "appid": app_id, "count": 20}
    )
    try:
        with urllib.request.urlopen(url, timeout=30) as response:
            return json.loads(response.read().decode("utf-8")).get("events") or []
    except (urllib.error.URLError, json.JSONDecodeError) as err:
        fail(f"could not read the public events listing: {err}")


def find_event(app_id: str, headline: str):
    for event in published_events(app_id):
        if (event.get("event_name") or "").strip() == headline:
            return event
    return None


def find_gids(payload):
    """Digs `gid` / `announcement_gid` out of the create response.

    Deliberately shape-agnostic: the response was not captured during the
    editor session, so rather than guess at a nesting we walk for the keys.
    """
    found = {}

    def walk(node):
        if isinstance(node, dict):
            for key, value in node.items():
                if key in ("gid", "announcement_gid") and isinstance(value, (str, int)):
                    found.setdefault(key, str(value))
                walk(value)
        elif isinstance(node, list):
            for item in node:
                walk(item)

    walk(payload)
    return found.get("gid"), found.get("announcement_gid")


def post(session: dict, fields: dict) -> dict:
    """One form POST to the partner-events endpoint.

    urllib does not follow redirects for POST by default, which is exactly what
    we want: a followed redirect would become a GET, drop the body, return a
    cheerful 200, and create nothing.
    """
    url = f"https://steamcommunity.com/gid/{CLAN_ID}/ajaxcreateupdatedeletepartnerevents/"
    body = urllib.parse.urlencode({**fields, "sessionid": session["sessionid"]})
    request = urllib.request.Request(
        url,
        data=body.encode("utf-8"),
        headers={
            "Content-Type": "application/x-www-form-urlencoded; charset=UTF-8",
            "Cookie": (
                f"sessionid={session['sessionid']}; "
                f"steamLoginSecure={session['steamid']}%7C%7C{session['token']}"
            ),
            "Referer": f"https://steamcommunity.com/games/{session['app_id']}/partnerevents/edit/",
            "Origin": "https://steamcommunity.com",
            "User-Agent": "Mozilla/5.0",
        },
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=60) as response:
            raw = response.read().decode("utf-8", "replace")
            if response.status in (301, 302, 303, 307, 308):
                fail("the endpoint redirected the POST — refusing to retry as a GET")
    except urllib.error.HTTPError as err:
        detail = err.read().decode("utf-8", "replace")[:400]
        if err.code in (401, 403):
            fail(
                f"Steam rejected the session ({err.code}). The refresh token is probably "
                f"no longer valid from this IP. {detail}"
            )
        fail(f"partner-events endpoint returned {err.code}: {detail}")
    except urllib.error.URLError as err:
        fail(f"partner-events endpoint unreachable: {err}")

    try:
        return json.loads(raw or "{}")
    except json.JSONDecodeError:
        fail(f"partner-events endpoint returned non-JSON: {raw[:400]}")


def jsondata(subtitle: str, summary: str) -> str:
    """The editor's side-channel blob. Only two of its keys matter to us.

    The sale_* keys the editor also sends belong to discount events and are
    omitted; the image arrays are sent empty so Steam falls back to the game
    capsule, which is what every previous Court Wizard announcement does.
    """
    slots = lambda value: [value] + [None] * (LANG_SLOTS - 1)  # noqa: E731
    return json.dumps(
        {
            "localized_subtitle": slots(subtitle),
            "localized_summary": slots(summary),
            "localized_title_image": slots(None),
            "localized_capsule_image": slots(None),
        }
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--version", required=True)
    parser.add_argument("--changelog", default="docs/CHANGELOG.md")
    parser.add_argument("--app-id", default="4550880")
    parser.add_argument(
        "--draft",
        action="store_true",
        help="create the event but do not publish it (leaves a Steamworks draft)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="render and validate the post, then exit without contacting Steam",
    )
    args = parser.parse_args()
    version = args.version.lstrip("v")

    headline = render(args.changelog, version, "--headline")
    description = render(args.changelog, version, "--description")
    body = render(args.changelog, version, "--body")

    # A checkout on the wrong ref renders the wrong block and, before this
    # guard existed, would have posted an announcement titled "pending" that no
    # version match could ever recognise — so every run would post another.
    if not headline.startswith(f"v{version}"):
        fail(f"rendered headline {headline!r} is not for v{version} — wrong changelog or ref")
    if not body.strip():
        fail(f"rendered body for v{version} is empty")
    if len(headline) > MAX_TITLE:
        fail(f"headline is {len(headline)} chars, over Steam's {MAX_TITLE} limit")

    # The subtitle stays empty and the description prose leads the body, which
    # is where changelog_to_bbcode.sh --body already puts it. A changelog
    # Description runs ~200 characters and the subtitle field caps at 120, so it
    # never fits; putting a truncated copy there as well would only duplicate
    # the opening line of the body.
    #
    # The summary does get a sentence-bounded prefix: it is what shows in event
    # lists, and leaving it blank makes Steam auto-generate one from the body,
    # which would open with a raw "[h3]Added[/h3]".
    subtitle = ""
    summary = fit(description, MAX_SUMMARY) if description else ""

    print(f"--- {headline} ---")
    print(f"  subtitle : empty (max {MAX_SUBTITLE}; prose leads the body instead)")
    print(f"  summary  : {len(summary)}/{MAX_SUMMARY}  {summary}")
    print(f"  body     : {len(body)} chars")

    existing = find_event(args.app_id, headline)
    if existing:
        print(f"'{headline}' is already on the hub as event {existing.get('gid')} — nothing to do.")
        emit_output("event-gid", str(existing.get("gid") or ""))
        return

    if args.dry_run:
        print("--- dry run: not contacting Steam ---")
        return

    refresh_token = os.environ.get("STEAM_WEB_REFRESH_TOKEN", "").strip()
    steamid = os.environ.get("STEAM_PUBLISH_STEAMID", "").strip()
    if not refresh_token or not steamid:
        fail("STEAM_WEB_REFRESH_TOKEN and STEAM_PUBLISH_STEAMID must both be set")

    session = {
        "sessionid": os.urandom(12).hex(),
        "steamid": steamid,
        "token": steam_access_token(refresh_token, steamid),
        "app_id": args.app_id,
    }

    now = int(time.time())
    created = post(
        session,
        {
            "bCreate": 1,
            "appid": args.app_id,
            "event_name": headline,
            "event_type": EVENT_TYPE_PATCH_NOTES,
            "body": body,
            "tags": json.dumps(["patchnotes"]),
            "jsondata": jsondata(subtitle, summary),
            "english_headline": headline,
            "english_body": body,
            "hidden": "true",
            "published": "false",
            "start_time_is_now": 1,
            "rtime32_start_time": now,
            "rtime32_end_time": now + 3600,
            "rtime32_visibility_start": 0,
            # Literal strings: this is what the editor sends for "unset".
            "rtime32_visibility_end": "undefined",
            "build_id": "undefined",
            "build_branch": "undefined",
        },
    )

    gid, announcement_gid = find_gids(created)
    if not gid or not announcement_gid:
        fail(
            "the create call did not return both gid and announcement_gid, so the event "
            f"cannot be published. Check Steamworks for a stray draft. Response: {created}"
        )
    # Printed before publishing on purpose: if the publish call dies, these are
    # the only handle on the draft that now exists.
    print(f"created draft event gid={gid} announcement_gid={announcement_gid}")
    emit_output("event-gid", gid)

    if args.draft:
        print("--draft: leaving the event unpublished.")
        return

    post(
        session,
        {
            "bPublish": 1,
            "gid": gid,
            "announcement_gid": announcement_gid,
            "unlistedmode": 0,
            "start_time_is_now": 1,
            "rtime32_visibility_start": 0,
            "rtime32_visibility_end": "undefined",
        },
    )

    # The POST's status is not proof: only the public listing is. Never retry the
    # create on a failure here — a second run would make a duplicate event while
    # the first sits published. The next scheduled pass sees it and exits.
    for attempt in range(6):
        event = find_event(args.app_id, headline)
        if event and event.get("published"):
            print(f"published: https://store.steampowered.com/news/app/{args.app_id}/view/{gid}")
            summarise(headline, gid, args.app_id, event)
            return
        time.sleep(10 * (attempt + 1))

    fail(
        f"published event {gid} but it has not appeared in the public listing. It may "
        "still be in Steam's moderation queue — check Steamworks before re-running, "
        "and do not let this run create a second event."
    )


def emit_output(name: str, value: str):
    path = os.environ.get("GITHUB_OUTPUT")
    if path:
        with open(path, "a", encoding="utf-8") as handle:
            handle.write(f"{name}={value}\n")


def summarise(headline: str, gid: str, app_id: str, event: dict):
    path = os.environ.get("GITHUB_STEP_SUMMARY")
    if not path:
        return
    url = f"https://store.steampowered.com/news/app/{app_id}/view/{gid}"
    with open(path, "a", encoding="utf-8") as handle:
        handle.write(
            f"### Steam patch notes posted\n\n"
            f"[{headline}]({url}) — event type `{event.get('event_type')}`, "
            f"tags `{(event.get('announcement_body') or {}).get('tags')}`.\n\n"
            f"Steam moderates new posts, so it can take an hour or more to show in the "
            f"Steam library.\n"
        )


if __name__ == "__main__":
    main()
