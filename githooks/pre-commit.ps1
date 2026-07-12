# Pre-commit guard (PowerShell equivalent of githooks/pre-commit).
# Blocks Google Drive handles / credentials from reaching git history.
# Activate on Windows with a hook that calls this, or:  git config core.hooksPath githooks
#
# Content checks read the STAGED blob (git show :<file>), not the working
# tree — what gets committed is exactly what gets scanned. All findings are
# collected and reported together before blocking.
$staged = git diff --cached --name-only --diff-filter=ACM
$failures = @()

foreach ($f in $staged) {
  # 1) Sensitive local files must never be committed, at any depth.
  $base = [System.IO.Path]::GetFileName($f)
  if ($base -match '^(config\.local\.json|\.mcp\.json|gdrive-creds\.json|client_secret_.*\.json)$') {
    $failures += "refusing to commit sensitive Drive file: $f"; continue
  }
  if ($f -like '.knuth-cache/*' -or $f -like '*/.knuth-cache/*') {
    $failures += "refusing to commit the resolved-ID / volume-text cache: $f"; continue
  }

  $lines = @(git show ":$f" 2>$null)
  if ($LASTEXITCODE -ne 0) { continue }

  # 2) A real folder ID must stay in config.local.json. The placeholder is
  #    exempt per LINE, not per file.
  if ($lines | Where-Object { $_ -match '"KNUTH_DRIVE_FOLDER"\s*:\s*"[A-Za-z0-9_-]{20,}"' -and $_ -notmatch 'PUT-YOUR-FOLDER-ID-HERE' }) {
    $failures += "a real KNUTH_DRIVE_FOLDER id is staged in $f - keep it in config.local.json (gitignored)."
  }

  # 3) A Drive URL is a handle too - block it repo-wide.
  if ($lines | Where-Object { $_ -match 'drive\.google\.com/(file/d|drive/folders)/[A-Za-z0-9_-]{20,}' }) {
    $failures += "a Google Drive URL with a file/folder id is staged in $f - resolve IDs at runtime, never commit them."
  }

  # 4) Drive file IDs must never be hardcoded into the map/skill/agents/docs.
  if (($f -like '.claude/skills/knuth-navigator/*' -or $f -like '.claude/agents/knuth-*' -or $f -like 'docs/knuth-agents/*') -and
      ($lines | Where-Object { $_ -match '"(id|fileId)"\s*:\s*"[A-Za-z0-9_-]{25,}"' })) {
    $failures += "a Drive file ID looks hardcoded in $f - resolve IDs at runtime, never commit them."
  }
}

if ($failures.Count -gt 0) {
  foreach ($m in $failures) { Write-Host "pre-commit: $m" }
  Write-Host 'pre-commit: blocked. See docs/knuth-agents/README.md.'
  exit 1
}
exit 0
