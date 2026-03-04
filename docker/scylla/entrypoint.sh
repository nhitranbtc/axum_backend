#!/bin/bash
# =============================================================================
# Custom Scylla entrypoint
#
# Writes the Scylla Manager Agent auth token to its config file before
# starting the main Scylla process. The token is injected via the
# SCYLLA_MANAGER_AGENT_AUTH_TOKEN environment variable set in docker-compose.
# =============================================================================
set -e

if [ -n "$SCYLLA_MANAGER_AGENT_AUTH_TOKEN" ]; then
    mkdir -p /etc/scylla-manager-agent
    echo "auth_token: $SCYLLA_MANAGER_AGENT_AUTH_TOKEN" \
        > /etc/scylla-manager-agent/scylla-manager-agent.yaml
fi

exec /docker-entrypoint.py "$@"
