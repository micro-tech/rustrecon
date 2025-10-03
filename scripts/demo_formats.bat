@echo off
echo ========================================
echo RustRecon Report Format Demo
echo ========================================
echo.

echo This demo shows the different report formats available in RustRecon:
echo - summary: One-line status overview
echo - condensed: Key findings only
echo - markdown: Full detailed report
echo - json: Machine-readable format
echo.

echo ========================================
echo 1. SUMMARY FORMAT (One-line overview)
echo ========================================
echo.
cargo run -- scan . --format summary --skip-dependencies
echo.
echo.

echo ========================================
echo 2. CONDENSED FORMAT (Key findings only)
echo ========================================
echo.
cargo run -- scan . --format condensed -o demo_condensed.md --skip-dependencies
echo Report saved to: demo_condensed.md
type demo_condensed.md
echo.
echo.

echo ========================================
echo 3. MARKDOWN FORMAT (Full details)
echo ========================================
echo.
cargo run -- scan . --format markdown -o demo_full.md --skip-dependencies
echo Report saved to: demo_full.md
echo First 50 lines of full report:
powershell "Get-Content demo_full.md | Select-Object -First 50"
echo ... (full report truncated for demo)
echo.
echo.

echo ========================================
echo 4. JSON FORMAT (Machine readable)
echo ========================================
echo.
cargo run -- scan . --format json -o demo_results.json --skip-dependencies
echo Report saved to: demo_results.json
echo JSON structure (first 20 lines):
powershell "Get-Content demo_results.json | Select-Object -First 20"
echo ... (JSON truncated for demo)
echo.
echo.

echo ========================================
echo FORMAT COMPARISON SUMMARY
echo ========================================
echo.
echo Summary format:    Perfect for dashboards and quick status checks
echo Condensed format:  Ideal for CI/CD and regular security reviews
echo Markdown format:   Best for thorough analysis and documentation
echo JSON format:       Excellent for tool integration and automation
echo.
echo Files generated:
echo - demo_condensed.md  (condensed report)
echo - demo_full.md       (full markdown report)
echo - demo_results.json  (JSON data)
echo.
echo Demo complete! Check the generated files for detailed examples.
pause
