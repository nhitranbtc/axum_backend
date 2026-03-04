#!/bin/sh
# =============================================================================
# Scylla Manager entrypoint
#
# Rewrites the template config with actual env var values before starting the
# manager process. Scylla Manager's YAML parser does NOT expand ${VAR} on its
# own, so we do it here with sed.
# =============================================================================
set -e

CONFIG_SRC="/etc/scylla-manager/scylla-manager.yaml"
CONFIG_TMP="/tmp/scylla-manager-resolved.yaml"

# Substitute env vars into the config
sed \
    -e "s|\${SCYLLA_MANAGER_DATABASE_USER}|${SCYLLA_MANAGER_DATABASE_USER:-cassandra}|g" \
    -e "s|\${SCYLLA_MANAGER_DATABASE_PASSWORD}|${SCYLLA_MANAGER_DATABASE_PASSWORD:-cassandra}|g" \
    "$CONFIG_SRC" > "$CONFIG_TMP"

exec scylla-manager --config-file="$CONFIG_TMP"
