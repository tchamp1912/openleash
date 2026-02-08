#!/bin/bash
# Leash AI - Sandboxed Execution Helper

# Try to find the generated profile from 'leash init' first
HOME_SB="$HOME/.leash/agent.sb"
LOCAL_SB="sandbox/agent.sb"

if [ -f "$HOME_SB" ]; then
    SB_PROFILE="$HOME_SB"
elif [ -f "$LOCAL_SB" ]; then
    SB_PROFILE="$LOCAL_SB"
else
    echo "Error: No sandbox profile found. Run 'leash init' first."
    exit 1
fi

TASKS_DIR="/tmp/leash-tasks"
mkdir -p "$TASKS_DIR"

echo "--- Entering Sandbox ($SB_PROFILE) ---"
sandbox-exec -f "$SB_PROFILE" "$@"