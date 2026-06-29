#!/usr/bin/env bash
# Appends multi-route window flags to the ARGS bash array.
# Set PAKE_WINDOW_SPECS (comma-separated, e.g. live=/live), PAKE_MULTI_WINDOW,
# and PAKE_SHOW_SYSTEM_TRAY to "true" before calling append_pake_window_args.

append_pake_window_args() {
  local specs="${PAKE_WINDOW_SPECS:-}"
  if [ -n "$specs" ]; then
    local IFS=','
    read -ra spec_list <<< "$specs"
    for spec in "${spec_list[@]}"; do
      spec="${spec#"${spec%%[![:space:]]*}"}"
      spec="${spec%"${spec##*[![:space:]]}"}"
      if [ -n "$spec" ]; then
        ARGS+=("--window" "$spec")
      fi
    done
  fi

  if [ "${PAKE_MULTI_WINDOW}" = "true" ]; then
    ARGS+=("--multi-window")
  fi

  if [ "${PAKE_SHOW_SYSTEM_TRAY}" = "true" ]; then
    ARGS+=("--show-system-tray")
  fi
}
