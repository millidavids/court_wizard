#!/usr/bin/env python3
"""Post a release announcement to Bluesky.

Bluesky's API is open and free — an app password plus two HTTP calls, no
approval or keys. `steam-promote.yml` runs this from the "live but not
announced" arm, so a post only goes out once the build is actually live on
Steam's default branch, and the `announced/v<version>` tag stops it repeating.

The post body comes from the `### Description` section of the version's
changelog block. That section is the release's one-line public hook, written by
hand during `/game-release`, and it also appears in the Discord announcement.

Two things about the AT Protocol that are easy to get wrong, and are the reason
this is Python rather than another `curl` step:

  * A URL in the text is NOT a link. Clickable links require an explicit
    `facet`. Because a facet can attach a URL to *any* span, the three links
    ride on short anchor words ("Steam", "Website", "Studio") instead of raw
    URLs — ~24 characters instead of ~130, which matters a lot against a
    300-character limit.
  * Facet offsets are *byte* offsets, not character offsets. Our changelog is
    full of em-dashes (3 bytes each), so naive character indexing silently
    points a link at the wrong span. Offsets here are recorded while building
    the string rather than searched for afterwards, so an anchor word that also
    occurs in the description text cannot be matched by mistake.

Usage:
    post_to_bluesky.py --version 1.0.38 [--changelog docs/CHANGELOG.md]
                       [--app-id 4550880] [--dry-run]

Environment:
    BLUESKY_USERNAME      handle, e.g. courtwizard.bsky.social
    BLUESKY_APP_PASSWORD  app password from Bluesky settings (not the account password)
"""

import argparse
import json
import os
import re
import sys
import urllib.error
import urllib.request
from datetime import datetime, timezone

PDS = "https://bsky.social"
# Bluesky counts graphemes, not bytes. 300 is the hard limit.
MAX_GRAPHEMES = 300

GAME_SITE = "https://courtwizard.blackhearthgames.com"
STUDIO_SITE = "https://blackhearthgames.com"
LINK_SEPARATOR = " · "


def fail(message: str):
    print(f"::error::{message}", file=sys.stderr)
    sys.exit(1)


def version_block(path: str, version: str) -> list[str]:
    """Returns the lines of the `## [v<version>]` block.

    Selects by version rather than taking the top block: a manual dispatch can
    target an older release while the changelog on main has already moved on,
    and announcing the top block there would ship the wrong notes.
    """
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except OSError as err:
        fail(f"cannot read {path}: {err}")

    header = f"## [v{version}]"
    block: list[str] = []
    inside = False
    for line in lines:
        if line.startswith(header):
            inside = True
            continue
        if inside and line.startswith("## ["):
            break
        if inside:
            block.append(line)
    if not block:
        fail(f"no changelog block found for '{header}' in {path}")
    return block


def description(block: list[str]) -> str:
    """Pulls the prose under `### Description`.

    Falls back to the first bullet's bolded lead-in so an older release, or one
    where the section was forgotten, still produces a sensible post rather than
    failing the promotion.
    """
    text: list[str] = []
    inside = False
    for line in block:
        if line.strip().startswith("### "):
            inside = line.strip().lower() == "### description"
            continue
        if inside and line.strip():
            text.append(line.strip())
    if text:
        return " ".join(text)

    for line in block:
        if line.startswith("- "):
            bullet = line[2:].strip()
            match = re.match(r"\*\*(.+?)\*\*\s*(?:—|-|–)?\s*(.*)", bullet, re.DOTALL)
            if match:
                lead, rest = match.group(1).strip(), match.group(2).strip()
                return f"{lead} — {rest}" if rest else lead
            return bullet.replace("**", "").strip()

    fail("changelog block has no ### Description and no bullets to fall back on")


def compose(version: str, body: str, links: list[tuple[str, str]]) -> tuple[str, list[dict]]:
    """Builds the post text and its facets, recording byte offsets as it goes."""
    title = f"Court Wizard v{version}"
    footer_len = len(LINK_SEPARATOR.join(label for label, _ in links))
    budget = MAX_GRAPHEMES - len(title) - footer_len - 4  # 4 = two blank-line joins

    if len(body) > budget:
        body = body[: max(budget - 1, 0)].rsplit(" ", 1)[0].rstrip(" ,;:—–-") + "…"

    text = f"{title}\n\n{body}\n\n"
    facets: list[dict] = []
    for index, (label, url) in enumerate(links):
        if index:
            text += LINK_SEPARATOR
        start = len(text.encode("utf-8"))
        text += label
        facets.append(
            {
                "index": {"byteStart": start, "byteEnd": len(text.encode("utf-8"))},
                "features": [{"$type": "app.bsky.richtext.facet#link", "uri": url}],
            }
        )
    return text, facets


def request(url: str, payload: dict, token: str | None = None) -> dict:
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    req = urllib.request.Request(
        url, data=json.dumps(payload).encode("utf-8"), headers=headers, method="POST"
    )
    try:
        with urllib.request.urlopen(req, timeout=30) as response:
            return json.loads(response.read().decode("utf-8"))
    except urllib.error.HTTPError as err:
        fail(f"{url} returned {err.code}: {err.read().decode('utf-8', 'replace')}")
    except urllib.error.URLError as err:
        fail(f"{url} unreachable: {err.reason}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--version", required=True)
    parser.add_argument("--changelog", default="docs/CHANGELOG.md")
    parser.add_argument("--app-id", default="4550880")
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="print the post and exit without contacting Bluesky",
    )
    args = parser.parse_args()

    version = args.version.lstrip("v")
    links = [
        ("Steam", f"https://store.steampowered.com/app/{args.app_id}/"),
        ("Website", GAME_SITE),
        ("Studio", STUDIO_SITE),
    ]
    body = description(version_block(args.changelog, version))
    text, facets = compose(version, body, links)

    if len(text) > MAX_GRAPHEMES:
        fail(f"post is {len(text)} graphemes, over the {MAX_GRAPHEMES} limit")

    print(f"--- post ({len(text)}/{MAX_GRAPHEMES} graphemes) ---\n{text}\n---")
    for facet, (label, url) in zip(facets, links):
        i = facet["index"]
        check = text.encode("utf-8")[i["byteStart"] : i["byteEnd"]].decode("utf-8")
        print(f"  facet {check!r} -> {url}" + ("" if check == label else "  MISMATCH"))

    if args.dry_run:
        return

    handle = os.environ.get("BLUESKY_USERNAME", "").strip()
    password = os.environ.get("BLUESKY_APP_PASSWORD", "").strip()
    if not handle or not password:
        fail("BLUESKY_USERNAME and BLUESKY_APP_PASSWORD must both be set")

    session = request(
        f"{PDS}/xrpc/com.atproto.server.createSession",
        {"identifier": handle, "password": password},
    )
    created_at = (
        datetime.now(timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z")
    )
    result = request(
        f"{PDS}/xrpc/com.atproto.repo.createRecord",
        {
            "repo": session["did"],
            "collection": "app.bsky.feed.post",
            "record": {
                "$type": "app.bsky.feed.post",
                "text": text,
                "createdAt": created_at,
                "facets": facets,
            },
        },
        token=session["accessJwt"],
    )
    print(f"posted: {result.get('uri', '(no uri returned)')}")


if __name__ == "__main__":
    main()
