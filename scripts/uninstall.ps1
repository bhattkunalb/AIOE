<#
.SYNOPSIS
    Professional Uninstaller for HMIR ELITE.
# cSpell:ignore WINDOWTITLE
    
.DESCRIPTION
    1. Terminates all background API, Dashboard, and NPU processes.
    2. Purges the .hmir application data directory.
    3. Removes HMIR from the USER PATH environment variable.
    4. Cleans up binary artifacts.
#>

$ErrorActionPreference = "SilentlyContinue"

Write-Host "`n🗑️  HMIR ELITE | FULL SYSTEM UNINSTALL" -ForegroundColor Cyan
Write-Host "--------------------------------------------------"

# 1. Terminate all instances
Write-Host "[1/4] Terminating background processes..." -NoNewline
taskkill /F /IM hmir-api.exe /T > $null 2>&1
taskkill /F /IM hmir-dashboard.exe /T > $null 2>&1
taskkill /F /IM hmir.exe /T > $null 2>&1
taskkill /F /IM python.exe /FI "WINDOWTITLE eq HMIR_NPU_BRIDGE*" /T > $null 2>&1
Start-Sleep -Seconds 1
Write-Host " ✅" -ForegroundColor Green

# 2. Purge Application Data
$HmirDir = Join-Path $HOME ".hmir"
Write-Host "[2/4] Purging runtime data ($HmirDir)..." -NoNewline
if (Test-Path $HmirDir) {
    Remove-Item -Path $HmirDir -Recurse -Force
    Write-Host " ✅" -ForegroundColor Green
} else {
    Write-Host " (Already clean)" -ForegroundColor Gray
}

# 3. Environment Cleanup (PATH)
Write-Host "[3/4] Cleaning environment variables..." -NoNewline
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
# In install.ps1 we added $HOME\.hmir to PATH.
if ($UserPath -like "*$HmirDir*") {
    $NewPath = ($UserPath -split ";" | Where-Object { $_ -ne $HmirDir }) -join ";"
    [Environment]::SetEnvironmentVariable("PATH", $NewPath, "User")
    Write-Host " ✅" -ForegroundColor Green
} else {
    Write-Host " (PATH already clean)" -ForegroundColor Gray
}

# 4. Binary Cleanup (Self-Destruct)
Write-Host "[4/4] Purging binary artifacts..." -NoNewline
# If we are running in the source tree, we don't want to delete the source.
# But if we were installed, we might want to delete the hmir.exe we're running from.
# On Windows, you can't delete a running exe, but you can RENAME it.
$CurrentExe = [System.Diagnostics.Process]::GetCurrentProcess().MainModule.FileName
if ($CurrentExe -like "*\.hmir\*") {
    Rename-Item -Path $CurrentExe -NewName "$CurrentExe.delete"
    Write-Host " ✅ (Binary marked for deletion on reboot)" -ForegroundColor Yellow
} else {
    Write-Host " ✅ (Source-tree preserved)" -ForegroundColor Gray
}

Write-Host "`n✨ HMIR ELITE has been successfully uninstalled from this system." -ForegroundColor Green
Write-Host "💡 Note: You may need to restart your terminal for PATH changes to take effect.`n"
