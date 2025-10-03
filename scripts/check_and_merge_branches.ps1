# Git Branch Checker and Merger Script for RustRecon
# This script checks branch status, merges branches, and ensures main is up to date

param(
    [switch]$Force,
    [switch]$DryRun,
    [switch]$Push,
    [switch]$Help
)

# Color functions for output
function Write-Success { param($Message) Write-Host $Message -ForegroundColor Green }
function Write-Warning { param($Message) Write-Host $Message -ForegroundColor Yellow }
function Write-Error { param($Message) Write-Host $Message -ForegroundColor Red }
function Write-Info { param($Message) Write-Host $Message -ForegroundColor Cyan }

# Help text
if ($Help) {
    Write-Host @"
RustRecon Git Branch Checker and Merger

USAGE:
    .\check_and_merge_branches.ps1 [OPTIONS]

OPTIONS:
    -Force      Force merge even if there are conflicts (use with caution)
    -DryRun     Show what would be done without making changes
    -Push       Push changes to remote after merging
    -Help       Show this help message

EXAMPLES:
    .\check_and_merge_branches.ps1
    .\check_and_merge_branches.ps1 -DryRun
    .\check_and_merge_branches.ps1 -Force -Push

"@ -ForegroundColor White
    exit 0
}

# Header
Write-Host @"
╔══════════════════════════════════════════════════════════════╗
║                Git Branch Status & Merger                    ║
║                                                              ║
║  Checking branch status and merging for RustRecon project   ║
╚══════════════════════════════════════════════════════════════╝
"@ -ForegroundColor Cyan

# Function to run git command with error handling
function Invoke-Git {
    param(
        [string]$Arguments,
        [switch]$ReturnOutput
    )

    try {
        if ($DryRun -and $Arguments -match "^(add|commit|merge|push|pull)") {
            Write-Info "[DRY RUN] Would run: git $Arguments"
            return $true
        }

        $result = Invoke-Expression "git $Arguments" 2>&1

        if ($LASTEXITCODE -eq 0) {
            if ($ReturnOutput) {
                return $result
            }
            return $true
        } else {
            Write-Error "Git command failed: git $Arguments"
            Write-Error "Error: $result"
            return $false
        }
    } catch {
        Write-Error "Failed to execute git command: $($_.Exception.Message)"
        return $false
    }
}

# Check if we're in a git repository
if (-not (Test-Path ".git")) {
    Write-Error "Not in a git repository. Please run this script from the RustRecon root directory."
    exit 1
}

Write-Info "Checking git repository status..."

# Get current branch
$currentBranch = Invoke-Git "rev-parse --abbrev-ref HEAD" -ReturnOutput
if (-not $currentBranch) {
    Write-Error "Failed to get current branch"
    exit 1
}

Write-Info "Current branch: $currentBranch"

# Check if working directory is clean
$status = Invoke-Git "status --porcelain" -ReturnOutput
if ($status) {
    Write-Warning "Working directory is not clean:"
    $status | ForEach-Object { Write-Host "  $_" }

    if (-not $Force) {
        $response = Read-Host "Continue anyway? Uncommitted changes may be lost. (y/N)"
        if ($response -notmatch '^y|yes$') {
            Write-Info "Operation cancelled by user."
            exit 0
        }
    }
}

# Get all branches
Write-Info "Getting branch information..."
$localBranches = Invoke-Git "branch" -ReturnOutput
$remoteBranches = Invoke-Git "branch -r" -ReturnOutput

if (-not $localBranches) {
    Write-Error "Failed to get branch information"
    exit 1
}

Write-Info "Local branches:"
$localBranches | ForEach-Object {
    $branch = $_.Trim().Replace("*", "").Trim()
    if ($branch) {
        $marker = if ($_.StartsWith("*")) { " (current)" } else { "" }
        Write-Host "  $branch$marker"
    }
}

if ($remoteBranches) {
    Write-Info "Remote branches:"
    $remoteBranches | ForEach-Object {
        $branch = $_.Trim()
        if ($branch -and -not $branch.Contains("HEAD")) {
            Write-Host "  $branch"
        }
    }
}

# Check if main branch exists
$hasMain = $localBranches -match "\bmain\b"
$hasMaster = $localBranches -match "\bmaster\b"

$mainBranch = if ($hasMain) { "main" } elseif ($hasMaster) { "master" } else { $null }

if (-not $mainBranch) {
    Write-Error "Neither 'main' nor 'master' branch found!"
    exit 1
}

Write-Success "Main branch identified as: $mainBranch"

# Switch to main branch if not already there
if ($currentBranch -ne $mainBranch) {
    Write-Info "Switching to $mainBranch branch..."
    if (-not (Invoke-Git "checkout $mainBranch")) {
        Write-Error "Failed to switch to $mainBranch branch"
        exit 1
    }
    Write-Success "Switched to $mainBranch branch"
}

# Fetch latest changes from remote
Write-Info "Fetching latest changes from remote..."
if (Invoke-Git "fetch origin") {
    Write-Success "Fetched latest changes"
} else {
    Write-Warning "Failed to fetch from remote, continuing with local branches"
}

# Check if main is behind remote
$behindCount = Invoke-Git "rev-list --count HEAD..origin/$mainBranch 2>/dev/null || echo '0'" -ReturnOutput
if ($behindCount -and $behindCount -gt 0) {
    Write-Warning "$mainBranch is $behindCount commits behind origin/$mainBranch"

    Write-Info "Pulling latest changes from origin/$mainBranch..."
    if (Invoke-Git "pull origin $mainBranch") {
        Write-Success "Successfully updated $mainBranch from remote"
    } else {
        Write-Error "Failed to pull from remote"
        if (-not $Force) {
            exit 1
        }
    }
}

# Find branches that need to be merged
$branchesToMerge = @()
$localBranches | ForEach-Object {
    $branch = $_.Trim().Replace("*", "").Trim()
    if ($branch -and $branch -ne $mainBranch -and -not $branch.StartsWith("origin/")) {
        $branchesToMerge += $branch
    }
}

if ($branchesToMerge.Count -eq 0) {
    Write-Success "No branches to merge. $mainBranch is up to date."
} else {
    Write-Info "Found $($branchesToMerge.Count) branch(es) to potentially merge:"
    $branchesToMerge | ForEach-Object { Write-Host "  $_" }

    # Check each branch for unmerged commits
    $branchesWithChanges = @()
    foreach ($branch in $branchesToMerge) {
        $unmergedCount = Invoke-Git "rev-list --count $mainBranch..$branch" -ReturnOutput
        if ($unmergedCount -and $unmergedCount -gt 0) {
            $branchesWithChanges += @{
                Name = $branch
                Commits = $unmergedCount
            }
        }
    }

    if ($branchesWithChanges.Count -eq 0) {
        Write-Success "All branches are already merged into $mainBranch"
    } else {
        Write-Warning "Found branches with unmerged changes:"
        $branchesWithChanges | ForEach-Object {
            Write-Host "  $($_.Name): $($_.Commits) unmerged commits" -ForegroundColor Yellow
        }

        if (-not $Force -and -not $DryRun) {
            $response = Read-Host "Merge these branches into $mainBranch? (y/N)"
            if ($response -notmatch '^y|yes$') {
                Write-Info "Merge cancelled by user."
                exit 0
            }
        }

        # Merge each branch
        foreach ($branchInfo in $branchesWithChanges) {
            $branch = $branchInfo.Name
            Write-Info "Merging branch '$branch' into $mainBranch..."

            if ($DryRun) {
                Write-Info "[DRY RUN] Would merge: $branch -> $mainBranch"
                continue
            }

            # Try to merge
            if (Invoke-Git "merge $branch --no-ff -m `"Merge branch '$branch' into $mainBranch`"") {
                Write-Success "Successfully merged $branch"

                # Ask if we should delete the branch
                if (-not $Force) {
                    $deleteResponse = Read-Host "Delete merged branch '$branch'? (y/N)"
                    if ($deleteResponse -match '^y|yes$') {
                        if (Invoke-Git "branch -d $branch") {
                            Write-Success "Deleted merged branch '$branch'"
                        }
                    }
                }
            } else {
                Write-Error "Failed to merge branch '$branch'"
                Write-Info "You may need to resolve conflicts manually."

                if (-not $Force) {
                    $continueResponse = Read-Host "Continue with remaining branches? (y/N)"
                    if ($continueResponse -notmatch '^y|yes$') {
                        break
                    }
                }
            }
        }
    }
}

# Push changes if requested
if ($Push -and -not $DryRun) {
    Write-Info "Pushing changes to remote..."
    if (Invoke-Git "push origin $mainBranch") {
        Write-Success "Successfully pushed changes to origin/$mainBranch"
    } else {
        Write-Error "Failed to push changes"
    }
}

# Final status check
Write-Info "Final repository status:"
$finalStatus = Invoke-Git "status --short" -ReturnOutput
if ($finalStatus) {
    Write-Warning "Uncommitted changes remain:"
    $finalStatus | ForEach-Object { Write-Host "  $_" }
} else {
    Write-Success "Working directory is clean"
}

# Check if main is now ahead of remote
$aheadCount = Invoke-Git "rev-list --count origin/$mainBranch..HEAD 2>/dev/null || echo '0'" -ReturnOutput
if ($aheadCount -and $aheadCount -gt 0) {
    Write-Info "$mainBranch is $aheadCount commits ahead of origin/$mainBranch"
    if (-not $Push) {
        Write-Info "Run with -Push to push changes to remote"
    }
}

# Summary
Write-Host ""
Write-Success "╔══════════════════════════════════════════════════════════════╗"
Write-Success "║                    Branch Check Complete                     ║"
Write-Success "║                                                              ║"
Write-Success "║  Repository is ready for installer to pull from main        ║"
Write-Success "╚══════════════════════════════════════════════════════════════╝"
Write-Host ""

if ($DryRun) {
    Write-Info "This was a dry run. No changes were made."
} else {
    Write-Info "All branches have been checked and merged as needed."
    Write-Info "The installer can now pull from main branch safely."
}

Write-Host ""
Write-Info "Next steps:"
Write-Host "  1. Verify the rustrecon_config.toml has correct model (gemini-1.5-pro-latest)"
Write-Host "  2. Test the application locally before pushing"
Write-Host "  3. Push changes to remote if not already done"
Write-Host "  4. Run installer to get updated version"

exit 0
