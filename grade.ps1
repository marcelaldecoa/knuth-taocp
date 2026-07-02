# Grade wrapper (PowerShell) — builds and runs the course grader.
#   .\grade.ps1            progress overview
#   .\grade.ps1 1          grade module 01
#   .\grade.ps1 verify     course self-check
#
# The Windows-native twin of the bash `grade` script, so `./grade` parity works
# from PowerShell without Git Bash. All arguments pass straight through.
$ErrorActionPreference = 'Stop'
Set-Location -LiteralPath $PSScriptRoot
& cargo run -q -p grader -- @args
exit $LASTEXITCODE
