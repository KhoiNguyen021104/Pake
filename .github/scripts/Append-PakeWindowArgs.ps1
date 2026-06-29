function Append-PakeWindowArgs {
    param(
        [Parameter(Mandatory)]
        [ref]$CliArgs,
        [string]$WindowSpecs,
        [string]$MultiWindow,
        [string]$ShowSystemTray
    )

    if ($WindowSpecs) {
        foreach ($spec in ($WindowSpecs -split ',')) {
            $trimmed = $spec.Trim()
            if ($trimmed) {
                $CliArgs.Value += '--window', $trimmed
            }
        }
    }

    if ($MultiWindow -eq 'true') {
        $CliArgs.Value += '--multi-window'
    }

    if ($ShowSystemTray -eq 'true') {
        $CliArgs.Value += '--show-system-tray'
    }
}
