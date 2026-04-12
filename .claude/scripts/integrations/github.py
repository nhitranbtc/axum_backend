#!/usr/bin/env python3
"""
github.py — GitHub integration.

Auth: GITHUB_TOKEN (PAT) via PyGithub.
Python handles all auth — LLM only sees data, never tokens.
"""

from __future__ import annotations

import os
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
from typing import Optional

import requests

TOKEN = os.environ.get("GITHUB_TOKEN")
BASE_URL = "https://api.github.com"
HEADERS = {
    "Authorization": f"Bearer {TOKEN}",
    "Accept": "application/vnd.github.v3+json",
    "X-GitHub-Api-Version": "2022-11-28",
}


# ---------------------------------------------------------------------------
# Data models
# ---------------------------------------------------------------------------

class PullRequestState(Enum):
    OPEN = "open"
    CLOSED = "closed"
    MERGED = "merged"


class IssueState(Enum):
    OPEN = "open"
    CLOSED = "closed"


@dataclass(frozen=True)
class User:
    login: str
    id: int
    avatar_url: str


@dataclass(frozen=True)
class PullRequest:
    number: int
    title: str
    state: str
    user: User
    body: Optional[str]
    created_at: str
    updated_at: str
    merged_at: Optional[str]
    draft: bool
    repo: str
    url: str
    review_requested: bool = False
    labels: tuple[str, ...] = field(default_factory=tuple)

    @property
    def is_open(self) -> bool:
        return self.state == "open"

    @property
    def is_merged(self) -> bool:
        return self.merged_at is not None


@dataclass(frozen=True)
class Issue:
    number: int
    title: str
    state: str
    body: Optional[str]
    created_at: str
    updated_at: str
    closed_at: Optional[str]
    user: User
    labels: tuple[str, ...]
    assignee: Optional[str]
    repo: str
    url: str

    @property
    def is_open(self) -> bool:
        return self.state == "open"


@dataclass(frozen=True)
class CommitStatus:
    state: str
    description: str
    context: str


# ---------------------------------------------------------------------------
# API client
# ---------------------------------------------------------------------------

class GitHubClient:
    """GitHub API client with rate-limit awareness."""

    def __init__(self, token: str = TOKEN):
        if not token:
            raise ValueError("GITHUB_TOKEN not set in environment")
        self.token = token
        self.session = requests.Session()
        self.session.headers.update({
            "Authorization": f"Bearer {token}",
            "Accept": "application/vnd.github.v3+json",
            "X-GitHub-Api-Version": "2022-11-28",
        })

    def _get(self, url: str, params: dict = None) -> dict | list:
        resp = self.session.get(url, params=params)
        if resp.status_code == 403 and "rate limit" in resp.text.lower():
            reset_at = int(resp.headers.get("X-RateLimit-Reset", 0))
            raise RuntimeError(f"GitHub API rate limit hit. Resets at {reset_at}")
        resp.raise_for_status()
        return resp.json()

    def _paginate(self, url: str, params: dict = None) -> list:
        """Auto-paginate through all pages."""
        results = []
        page = 1
        while True:
            p = (params or {}).copy()
            p.setdefault("per_page", 100)
            p["page"] = page
            resp = self.session.get(url, params=p)
            if resp.status_code == 403 and "rate limit" in resp.text.lower():
                reset_at = int(resp.headers.get("X-RateLimit-Reset", 0))
                raise RuntimeError(f"GitHub API rate limit hit. Resets at {reset_at}")
            resp.raise_for_status()
            data = resp.json()
            if isinstance(data, list):
                results.extend(data)
            else:
                results.extend(data.get("items", []))
            if isinstance(data, list) and len(data) < 100:
                break
            if "next" not in resp.links:
                break
            page += 1
        return results

    # ---------------------------------------------------------------------------
    # Pull Requests
    # ---------------------------------------------------------------------------

    def list_prs(
        self,
        owner: str,
        repo: str,
        state: str = "open",
        sort: str = "updated",
        direction: str = "desc",
    ) -> list[PullRequest]:
        """List pull requests for a repository."""
        url = f"{BASE_URL}/repos/{owner}/{repo}/pulls"
        params = {"state": state, "sort": sort, "direction": direction}
        data = self._paginate(url, params)
        return [self._pr_from_dict(owner, repo, d) for d in data]

    def get_pr(self, owner: str, repo: str, number: int) -> PullRequest:
        """Get a single PR with full detail."""
        url = f"{BASE_URL}/repos/{owner}/{repo}/pulls/{number}"
        data = self._get(url)
        return self._pr_from_dict(owner, repo, data)

    def list_prs_review_requested(
        self,
        owner: str,
        repo: str,
        username: str,
    ) -> list[PullRequest]:
        """List PRs where review is requested from a specific user."""
        url = f"{BASE_URL}/repos/{owner}/{repo}/pulls"
        params = {"state": "open", "sort": "updated", "direction": "desc"}
        all_prs = self._paginate(url, params)
        requested = []
        for pr_data in all_prs:
            reviewers = pr_data.get("requested_reviewers", []) or []
            reviewer_logins = [r.get("login", "") for r in reviewers]
            if username in reviewer_logins:
                requested.append(self._pr_from_dict(owner, repo, pr_data))
        return requested

    def get_pr_files(self, owner: str, repo: str, number: int) -> list[str]:
        """List files changed in a PR."""
        url = f"{BASE_URL}/repos/{owner}/{repo}/pulls/{number}/files"
        data = self._paginate(url)
        return [f["filename"] for f in data]

    # ---------------------------------------------------------------------------
    # Issues
    # ---------------------------------------------------------------------------

    def list_issues(
        self,
        owner: str,
        repo: str,
        state: str = "open",
        labels: Optional[str] = None,
        assignee: Optional[str] = None,
    ) -> list[Issue]:
        """List issues for a repository (not PRs)."""
        url = f"{BASE_URL}/repos/{owner}/{repo}/issues"
        params = {"state": state, "sort": "updated", "direction": "desc"}
        if labels:
            params["labels"] = labels
        if assignee:
            params["assignee"] = assignee
        data = self._paginate(url, params)
        return [self._issue_from_dict(owner, repo, d) for d in data if not d.get("pull_request")]

    def get_issue(self, owner: str, repo: str, number: int) -> Issue:
        """Get a single issue."""
        url = f"{BASE_URL}/repos/{owner}/{repo}/issues/{number}"
        data = self._get(url)
        return self._issue_from_dict(owner, repo, data)

    # ---------------------------------------------------------------------------
    # Commits & Status
    # ---------------------------------------------------------------------------

    def get_commit_statuses(
        self,
        owner: str,
        repo: str,
        ref: str,
    ) -> list[CommitStatus]:
        """Get combined status for a commit ref."""
        url = f"{BASE_URL}/repos/{owner}/{repo}/commits/{ref}/status"
        data = self._get(url)
        statuses = []
        for s in data.get("statuses", []):
            statuses.append(CommitStatus(
                state=s["state"],
                description=s["description"],
                context=s["context"],
            ))
        return statuses

    def get_workflow_runs(
        self,
        owner: str,
        repo: str,
        branch: Optional[str] = None,
    ) -> list[dict]:
        """Get recent workflow runs."""
        url = f"{BASE_URL}/repos/{owner}/{repo}/actions/runs"
        params = {}
        if branch:
            params["branch"] = branch
        data = self._get(url, params)
        return data.get("workflow_runs", [])

    # ---------------------------------------------------------------------------
    # Repos & Search
    # ---------------------------------------------------------------------------

    def search_repos(self, query: str, top_k: int = 10) -> list[dict]:
        """Search repositories."""
        url = f"{BASE_URL}/search/repositories"
        params = {"q": query, "per_page": top_k}
        data = self._get(url, params)
        return data.get("items", [])

    def get_repo(self, owner: str, repo: str) -> dict:
        """Get repository metadata."""
        url = f"{BASE_URL}/repos/{owner}/{repo}"
        return self._get(url)

    # ---------------------------------------------------------------------------
    # Internal
    # ---------------------------------------------------------------------------

    def _pr_from_dict(self, owner: str, repo: str, data: dict) -> PullRequest:
        label_names = tuple(lb.get("name", "") for lb in data.get("labels", []))
        return PullRequest(
            number=data["number"],
            title=data["title"],
            state=data["state"],
            user=User(login=data["user"]["login"], id=data["user"]["id"],
                     avatar_url=data["user"]["avatar_url"]),
            body=data.get("body"),
            created_at=data["created_at"],
            updated_at=data["updated_at"],
            merged_at=data.get("merged_at"),
            draft=data.get("draft", False),
            repo=f"{owner}/{repo}",
            url=data["html_url"],
            labels=label_names,
        )

    def _issue_from_dict(self, owner: str, repo: str, data: dict) -> Issue:
        label_names = tuple(lb.get("name", "") for lb in data.get("labels", []))
        return Issue(
            number=data["number"],
            title=data["title"],
            state=data["state"],
            body=data.get("body"),
            created_at=data["created_at"],
            updated_at=data["updated_at"],
            closed_at=data.get("closed_at"),
            user=User(login=data["user"]["login"], id=data["user"]["id"],
                     avatar_url=data["user"]["avatar_url"]),
            labels=label_names,
            assignee=data.get("assignee", {}).get("login") if data.get("assignee") else None,
            repo=f"{owner}/{repo}",
            url=data["html_url"],
        )


# ---------------------------------------------------------------------------
# Context formatters — produce readable summaries for LLM consumption
# ---------------------------------------------------------------------------

def format_pr(pr: PullRequest) -> str:
    """Format a PR for human-readable display."""
    state = "📦 DRAFT" if pr.draft else ("✅ OPEN" if pr.is_open else "❌ CLOSED")
    labels = f" [{', '.join(pr.labels)}]" if pr.labels else ""
    return (
        f"## PR #{pr.number}: {pr.title}\n"
        f"   State: {state}{labels}\n"
        f"   Author: @{pr.user.login} | Updated: {pr.updated_at[:10]}\n"
        f"   URL: {pr.url}\n"
    )


def format_prs(prs: list[PullRequest]) -> str:
    if not prs:
        return "_No pull requests found._"
    return "\n".join(format_pr(pr) for pr in prs)


def format_issue(issue: Issue) -> str:
    """Format an issue for human-readable display."""
    state = "✅ OPEN" if issue.is_open else "❌ CLOSED"
    labels = f" [{', '.join(issue.labels)}]" if issue.labels else ""
    assignee = f" | Assignee: @{issue.assignee}" if issue.assignee else ""
    return (
        f"## Issue #{issue.number}: {issue.title}\n"
        f"   State: {state}{labels}{assignee}\n"
        f"   Author: @{issue.user.login} | Updated: {issue.updated_at[:10]}\n"
        f"   URL: {issue.url}\n"
    )


def format_issues(issues: list[Issue]) -> str:
    if not issues:
        return "_No issues found._"
    return "\n".join(format_issue(i) for i in issues)


def format_file_list(files: list[str]) -> str:
    """Format a list of changed files."""
    if not files:
        return "_No files changed._"
    return "\n".join(f"  - {f}" for f in files)
