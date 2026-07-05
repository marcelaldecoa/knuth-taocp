# Setup (macOS & Windows)

Activating the Knuth expert agents, and **where each piece of information comes
from**. The files already live in the repo (see [`README.md`](README.md)); this
is the wiring. Steps are given for both **macOS** (zsh / Homebrew) and
**Windows** (PowerShell).

## Where each value comes from

| You need | Where to get it | Secret? |
|---|---|---|
| **Claude Code** | `npm i -g @anthropic-ai/claude-code` (verify name in docs) | no |
| **Drive folder ID** | The folder's URL in your browser (step 2) | no (just a handle) |
| **Drive authorization** | Managed connector click-through, or Google Cloud OAuth (step 3) | **yes** (the token) |
| **File IDs** | Never needed by hand — the navigator resolves them at runtime | no |
| **Section numbers / pages** | Already baked into `map.json` / `INDEX.md` | no |

The only genuine secret is the **Drive authorization token**. Everything else is
public knowledge or a non-sensitive handle.

## 0. Prerequisites

If you don't already run Claude Code:

**macOS** (zsh):
```zsh
# install Homebrew first if needed: https://brew.sh
brew install node && node -v
npm install -g @anthropic-ai/claude-code   # verify pkg name in docs
```
**Windows** (PowerShell):
```powershell
winget install OpenJS.NodeJS   # or install LTS from https://nodejs.org
# reopen PowerShell so PATH refreshes
node -v
npm install -g @anthropic-ai/claude-code   # verify pkg name in docs
```

## 1. Point the navigator at your Drive folder

```zsh
cp config.local.example.json config.local.json
```
Open `config.local.json` and set `KNUTH_DRIVE_FOLDER` to your folder ID.

## 2. Get the folder ID

Open the Drive folder that holds your Knuth PDFs in a browser. The URL is
`https://drive.google.com/drive/folders/<THIS-IS-THE-ID>`. Copy that id.
It is a handle, not a secret — but it still stays in the gitignored
`config.local.json`, never in a tracked file (the pre-commit hook enforces this).

## 3. Connect Google Drive (pick ONE)

### Option A — managed connector (recommended)
The same Google Drive connector Claude uses elsewhere; it handles OAuth for you,
so **no credentials file lives on disk to guard**.

1. In Claude Code / the Claude app, open connector settings.
2. Add / enable **Google Drive** and complete Google sign-in.
3. Grant read access to the account that owns the Knuth folder.

With Option A you do **not** need `.mcp.json` — skip it.

### Option B — self-hosted MCP server (more control)
Only if you want the Drive server running locally under your own Google Cloud
OAuth app.

1. **Google Cloud Console** → create/reuse a project → enable the **Google Drive API**.
2. **OAuth consent screen** → configure, add your account as a test user.
3. **Credentials → OAuth client ID → Desktop app** → download the JSON.
4. Store it **outside the repo** and lock it down:
   - macOS: `mkdir -p ~/.config/knuth-taocp && mv ~/Downloads/client_secret_*.json ~/.config/knuth-taocp/gdrive-creds.json && chmod 600 ~/.config/knuth-taocp/gdrive-creds.json`
   - Windows: put it at `%APPDATA%\knuth-taocp\gdrive-creds.json` and restrict ACLs to your user.
5. `cp .mcp.json.example .mcp.json` and point `GDRIVE_CREDENTIALS_PATH` at that file.
   (`.mcp.json` is gitignored.)

## 4. Turn on the safety hook

```
git config core.hooksPath githooks
```
`githooks/pre-commit` (and `pre-commit.ps1`) refuse to commit a real folder ID,
a Drive file ID, credentials, or the local cache.

## 5. Use it

Ask about any Knuth section — "walk me through §5.2.2, quicksort's analysis",
"which volume has ZDDs", "prove Theorem 4.5.3F (Lamé)". The `knuth-navigator`
skill resolves the volume to a live file ID, fetches just that section, and for
deep dives hands off to the matching expert subagent. On a Drive auth error,
re-authorize the connector — the agents will not guess book content.

> **Config locations differ by OS.** macOS/Linux use `~/.config/knuth-taocp/`;
> Windows uses `%APPDATA%\knuth-taocp\`.
