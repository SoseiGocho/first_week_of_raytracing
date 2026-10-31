#! /usr/bin/env pwsh

param([String]$ProjectName)

Measure-Command { &( ".\" + $ProjectName + "\target\release\" + $ProjectName + ".exe") } `
    | Out-File -LiteralPath (
        ".\result\" `
            + (Get-Date -Format "yyyyMMddHHmmss") `
            + "_" `
            + $ProjectName `
            + "_performance.txt")
