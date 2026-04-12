#!/usr/bin/env python3
"""
query.py — Unified CLI for all integrations.

Usage:
    python query.py github prs --repo owner/repo [--state open|closed|all]
    python query.py github issues --repo owner/repo [--assignee username] [--labels bug]
    python query.py github review-requested --repo owner/repo --user username
    python query.py github status --repo owner/repo --ref develop
    python query.py registry status
    python query.py registry list
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from integrations.github import GitHubClient, format_prs, format_issues, format_file_list


# ---------------------------------------------------------------------------
# GitHub commands
# ---------------------------------------------------------------------------

def cmd_github_prs(args):
    """List PRs for a repository."""
    owner, repo = args.repo.split("/")
    client = GitHubClient()
    prs = client.list_prs(owner, repo, state=args.state, sort=args.sort)
    print(f"PRs in {args.repo} (state={args.state}):")
    print(format_prs(prs))


def cmd_github_review_requested(args):
    """List PRs where review is requested from a user."""
    owner, repo = args.repo.split("/")
    client = GitHubClient()
    prs = client.list_prs_review_requested(owner, repo, args.user)
    print(f"PRs requesting review from @{args.user} in {args.repo}:")
    print(format_prs(prs))


def cmd_github_issues(args):
    """List issues for a repository."""
    owner, repo = args.repo.split("/")
    client = GitHubClient()
    issues = client.list_issues(
        owner, repo,
        state=args.state,
        labels=args.labels,
        assignee=args.assignee,
    )
    print(f"Issues in {args.repo} (state={args.state}):")
    print(format_issues(issues))


def cmd_github_pr_detail(args):
    """Get detail on a single PR including changed files."""
    owner, repo = args.repo.split("/")
    client = GitHubClient()
    pr = client.get_pr(owner, repo, args.number)
    files = client.get_pr_files(owner, repo, args.number)
    from integrations.github import format_pr
    print(format_pr(pr))
    print("\nChanged files:")
    print(format_file_list(files))


def cmd_github_status(args):
    """Get CI/commit status for a ref."""
    owner, repo = args.repo.split("/")
    client = GitHubClient()
    statuses = client.get_commit_statuses(owner, repo, args.ref)
    if not statuses:
        print(f"No status checks found for ref: {args.ref}")
    else:
        print(f"Status checks for {args.ref}:")
        for s in statuses:
            icon = "✅" if s.state == "success" else "❌" if s.state == "failure" else "🔄"
            print(f"  {icon} [{s.context}] {s.description}")


def cmd_github_search(args):
    """Search repositories."""
    client = GitHubClient()
    results = client.search_repos(args.query, top_k=args.top_k)
    if not results:
        print(f"No repos found for query: {args.query}")
        return
    print(f"Top {len(results)} repos matching '{args.query}':")
    for r in results:
        stars = r.get("stargazers_count", 0)
        desc = r.get("description", "")
        print(f"  ★ {stars:>5}  {r['full_name']} — {desc[:70]}")


# ---------------------------------------------------------------------------
# Registry commands
# ---------------------------------------------------------------------------

def cmd_registry_status(args):
    from integrations.registry import print_status
    print_status()


def cmd_registry_list(args):
    from integrations.registry import INTEGRATIONS
    print("Available integrations:")
    for name, config in INTEGRATIONS.items():
        print(f"  - {name}: {config['description']}")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def build_parser():
    parser = argparse.ArgumentParser(description="Second Brain Query CLI")
    sub = parser.add_subparsers(dest="integration", help="Integration to query")

    # GitHub
    gh = sub.add_parser("github", help="GitHub integration")
    gh_sub = gh.add_subparsers(dest="cmd")

    p = gh_sub.add_parser("prs", help="List PRs")
    p.add_argument("--repo", required=True, help="owner/repo")
    p.add_argument("--state", default="open", choices=["open", "closed", "all"])
    p.add_argument("--sort", default="updated", choices=["created", "updated", "popularity"])
    p.set_defaults(func=cmd_github_prs)

    p = gh_sub.add_parser("review-requested", help="PRs requesting review from a user")
    p.add_argument("--repo", required=True, help="owner/repo")
    p.add_argument("--user", required=True, help="GitHub username")
    p.set_defaults(func=cmd_github_review_requested)

    p = gh_sub.add_parser("issues", help="List issues")
    p.add_argument("--repo", required=True, help="owner/repo")
    p.add_argument("--state", default="open", choices=["open", "closed", "all"])
    p.add_argument("--labels", help="Comma-separated label names")
    p.add_argument("--assignee", help="Assignee username")
    p.set_defaults(func=cmd_github_issues)

    p = gh_sub.add_parser("pr-detail", help="Single PR with changed files")
    p.add_argument("--repo", required=True, help="owner/repo")
    p.add_argument("--number", required=True, type=int)
    p.set_defaults(func=cmd_github_pr_detail)

    p = gh_sub.add_parser("status", help="Commit status checks")
    p.add_argument("--repo", required=True, help="owner/repo")
    p.add_argument("--ref", required=True, help="commit SHA, branch, or tag")
    p.set_defaults(func=cmd_github_status)

    p = gh_sub.add_parser("search", help="Search repositories")
    p.add_argument("query", type=str, help="Search query")
    p.add_argument("--top-k", default=10, type=int)
    p.set_defaults(func=cmd_github_search)

    # Linear (stub — populated in Phase 4B)
    lin = sub.add_parser("linear", help="Linear integration (not yet configured)")
    lin.set_defaults(func=lambda args: print("Linear integration not yet built."))

    # Slack (stub — populated in Phase 4C)
    slk = sub.add_parser("slack", help="Slack integration (not yet configured)")
    slk.set_defaults(func=lambda args: print("Slack integration not yet built."))

    # Registry
    reg = sub.add_parser("registry", help="Integration registry")
    reg_sub = reg.add_subparsers(dest="cmd")
    r = reg_sub.add_parser("status", help="Check all integrations")
    r.set_defaults(func=cmd_registry_status)
    r = reg_sub.add_parser("list", help="List available integrations")
    r.set_defaults(func=cmd_registry_list)

    return parser


def main():
    parser = build_parser()
    args = parser.parse_args()
    if not args.integration:
        parser.print_help()
        sys.exit(1)
    if not hasattr(args, "func"):
        parser.parse_args(["--help"])
        sys.exit(1)
    args.func(args)


if __name__ == "__main__":
    main()
