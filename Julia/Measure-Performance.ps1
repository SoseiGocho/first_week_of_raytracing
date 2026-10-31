# /usr/bin/env pwsh

param(
    [Int16]$NumOfThreads,
    [Int16]$SPP,
    [Int16]$Depth,
    [String]$OutputPath
)

$date = Get-Date -Format "yyyyMMddHHmmss";

Measure-Command { julia --project=. -t ${NumOfThreads} .\AFinalRender.jl `
    (${OutputPath} `
    + "\" `
    +  ${date}`
    + "AFinalRenderS" + ${SPP} + "D" + ${Depth} + "Corrected.png") } `
    | Out-File -LiteralPath ( `
        ${OutputPath} `
        + "\" `
        + ${date} `
        + "a_final_render_s" + ${SPP} `
        + "_d" + ${Depth} `
        + "_t" + ${NumOfThreads} `
        + "_corrected_time.log")