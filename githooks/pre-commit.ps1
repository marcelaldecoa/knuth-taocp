# Pre-commit guard (PowerShell equivalent of githooks/pre-commit).
# Blocks Google Drive handles / credentials from reaching git history.
# Activate on Windows with a hook that calls this, or:  git config core.hooksPath githooks
$ErrorActionPreference = 'Stop'
$staged = git diff --cached --name-only --diff-filter=ACM
$fail = $false

foreach ($f in $staged) {
  if ($f -match '^(config\.local\.json|\.mcp\.json|gdrive-creds\.json|client_secret_.*\.json)$' -or
      $f -match 'gdrive-creds\.json$' -or $f -like '.knuth-cache/*') {
    Write-Error "pre-commit: refusing to commit sensitive Drive file: $f"; $fail = $true
  }
}

foreach ($f in $staged) {
  if (-not (Test-Path $f)) { continue }
  $c = Get-Content -Raw $f
  if ($c -match '"KNUTH_DRIVE_FOLDER"\s*:\s*"[A-Za-z0-9_-]{20,}"' -and $c -notmatch 'PUT-YOUR-FOLDER-ID-HERE') {
    Write-Error "pre-commit: a real KNUTH_DRIVE_FOLDER id is staged in $f — keep it in config.local.json (gitignored)."; $fail = $true
  }
  if (($f -like '.claude/skills/knuth-navigator/*' -or $f -like '.claude/agents/knuth-*' -or $f -like 'docs/knuth-agents/*') -and
      $c -match '"(id|fileId)"\s*:\s*"[A-Za-z0-9_-]{25,}"') {
    Write-Error "pre-commit: a Drive file ID looks hardcoded in $f — resolve IDs at runtime, never commit them."; $fail = $true
  }
}

if ($fail) { Write-Error 'pre-commit: blocked. See docs/knuth-agents/README.md.'; exit 1 }
