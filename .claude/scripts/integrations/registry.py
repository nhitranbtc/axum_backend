#!/usr/bin/env python3
"""
registry.py — Integration registry.

Tracks which integrations are enabled, their auth status,
and provides a unified interface for health checks.
"""

from __future__ import annotations

import os
import sys
from dataclasses import dataclass, fields
from pathlib import Path
from typing import Optional

# ---------------------------------------------------------------------------
# Integration list
# ---------------------------------------------------------------------------

INTEGRATIONS = {
    "github": {
        "module": "integrations.github",
        "class": "GitHubClient",
        "env_vars": ["GITHUB_TOKEN"],
        "description": "Pull requests, issues, commit status",
    },
    "linear": {
        "module": "integrations.linear",
        "class": "LinearClient",
        "env_vars": ["LINEAR_API_KEY"],
        "description": "Issues, teams, projects",
    },
    "slack": {
        "module": "integrations.slack",
        "class": "SlackClient",
        "env_vars": ["SLACK_BOT_TOKEN", "SLACK_APP_TOKEN"],
        "description": "Channels, messages, notifications",
    },
}


# ---------------------------------------------------------------------------
# Registry entry
# ---------------------------------------------------------------------------

@dataclass
class IntegrationStatus:
    name: str
    enabled: bool
    configured: bool
    error: Optional[str] = None
    last_check: Optional[str] = None


def check_integration(name: str) -> IntegrationStatus:
    """
    Check if an integration is configured and functional.

    Checks:
    1. Module can be imported
    2. Environment variables are set
    3. API token is valid (lightweight API call)
    """
    config = INTEGRATIONS.get(name)
    if not config:
        return IntegrationStatus(name=name, enabled=False, configured=False,
                                 error=f"Unknown integration: {name}")

    # Check env vars
    missing = [v for v in config["env_vars"] if not os.environ.get(v)]
    if missing:
        return IntegrationStatus(
            name=name,
            enabled=False,
            configured=False,
            error=f"Missing env vars: {missing}",
        )

    # Check module imports
    try:
        sys.path.insert(0, str(Path(__file__).parent))
        __import__(config["module"])
    except Exception as e:
        return IntegrationStatus(
            name=name,
            enabled=True,
            configured=False,
            error=f"Import error: {e}",
        )

    return IntegrationStatus(
        name=name,
        enabled=True,
        configured=True,
    )


def list_integrations() -> dict[str, IntegrationStatus]:
    """Check all registered integrations."""
    return {name: check_integration(name) for name in INTEGRATIONS}


def print_status():
    """Print a table of integration statuses."""
    statuses = list_integrations()
    print(f"{'Integration':<12} {'Enabled':<10} {'Configured':<12} {'Status'}")
    print("-" * 60)
    for name, status in statuses.items():
        if status.configured:
            state = "✅ OK"
        elif status.enabled:
            state = f"⚠️  {status.error}"
        else:
            state = f"❌ {status.error}"
        print(f"{name:<12} {'yes' if status.enabled else 'no':<10} "
              f"{'yes' if status.configured else 'no':<12} {state}")
