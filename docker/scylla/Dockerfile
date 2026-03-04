FROM scylladb/scylla:6.2

# =============================================================================
# Install Scylla Manager Agent
#
# Adds the ScyllaDB Manager APT repository and installs the agent package,
# which runs as a supervisord sidecar alongside the Scylla server process.
# =============================================================================
RUN apt-get update && \
    apt-get install -y --no-install-recommends curl gnupg && \
    mkdir -p /etc/apt/keyrings && \
    curl -fsSL "https://keyserver.ubuntu.com/pks/lookup?op=get&search=0xa43e06657bac99e3" \
    | gpg --dearmor -o /etc/apt/keyrings/scylladb.gpg && \
    curl -fsSL -o /etc/apt/sources.list.d/scylla-manager.list \
    https://downloads.scylladb.com/deb/debian/scylladb-manager-3.8.list && \
    apt-get update && \
    apt-get install -y --no-install-recommends scylla-manager-agent && \
    scyllamgr_agent_setup -y || true && \
    rm -rf /var/lib/apt/lists/*

# Register the agent with supervisord so it starts alongside scylla-server
COPY docker/scylla/scylla-manager-agent.conf /etc/supervisord.conf.d/scylla-manager-agent.conf

# Custom entrypoint: writes the auth token before handing off to Scylla
COPY docker/scylla/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
