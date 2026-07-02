@echo off
rem Grade wrapper (cmd.exe) — builds and runs the course grader.
rem   grade            progress overview
rem   grade 1          grade module 01
rem   grade verify     course self-check
rem
rem Lets `grade ...` work from cmd.exe / double-click on Windows. All arguments
rem pass straight through to the grader.
cd /d "%~dp0"
cargo run -q -p grader -- %*
