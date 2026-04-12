# USER.md — User Profile

> This file contains all information about Tran Thi Ai Nhi that the AI agent needs to know about the user.

---

## Identity

- **Name:** Tran Thi Ai Nhi
- **Email:** tranthiainhi303@gmail.com
- **GitHub:** github.com/nhitranbtc
- **Role:** Blockchain & Backend Engineer (Rust / Substrate)
- **Timezone:** Vietnam (UTC+7)
- **Language:** English (primary), Vietnamese (secondary)

---

## Technical Profile

### Core Stack

| Category | Technologies |
|----------|-------------|
| Languages | Rust (5+ years), Ink!, Solidity, Erlang |
| Blockchain | Substrate Framework, Polkadot-SDK, smart contracts, solochain, parachains, NFT/DeFi/GameFi/DApps |
| Backend | Backend systems design (7+ years), microservices, REST APIs, payment systems, remittance, KYC/KYB, actor-model services |
| Datastores | PostgreSQL, RocksDB (Substrate), Riak NoSQL, ElasticsearchDB |
| Tools | Linux, Docker, CI/CD, Git, MQTT, NAT, Distributed Systems |

### Methodologies

- Agile (Scrum)
- AI-assisted development: Claude AI, LLM workflows, generative AI, prompt engineering (Claude, ChatGPT, GitHub Copilot)

---

## Platforms & Integrations

| Platform | Status | Config |
|----------|--------|--------|
| GitHub | Active | `GITHUB_TOKEN` in `.env` |
| Linear | Active | `LINEAR_API_KEY` in `.env` |
| Slack | Active | `SLACK_BOT_TOKEN` + `SLACK_APP_TOKEN` in `.env` |
| Obsidian | Primary notes app | Vault at `Dynamous/Memory/` |
| Docker | Containerization | Local + remote registry |
| Claude Code | Primary AI assistant | Hooks configured at `.claude/hooks/` |

---

## Active Projects

- _(Update as projects are added)_

---

## Team Context

- _(Update as team members and their roles are added)_

---

## Preferences

| Setting | Value |
|---------|-------|
| Response style | Short, technical, code-first |
| Drafting mode | Advisor (drafts for review, never auto-sends) |
| Notification time | Active hours 09:00–22:00 UTC+7 |
| Daily reflection | 08:00 UTC+7 |

---

## Vault Structure

```
Dynamous/Memory/
├── SOUL.md           # Agent personality + rules
├── USER.md           # This file
├── MEMORY.md         # Persistent memory index (updated by hooks)
├── HABITS.md         # Habit pillars + daily tracking
├── daily/           # Daily append-only logs (YYYY-MM-DD.md)
├── drafts/           # Draft management lifecycle
│   ├── active/      # Drafts awaiting review
│   ├── sent/        # Sent/replied drafts (voice-matching corpus)
│   └── expired/     # Drafts older than 24h
└── HEARTBEAT.md     # Heartbeat monitoring checklist
```

---

## Security Boundaries

The agent **MUST NEVER**:

- Send emails or messages on Tran's behalf
- Post to social media or community platforms
- Deploy or interact with smart contracts on-chain
- Access financial data or make purchases
- Delete anything outside `drafts/expired/`
- Modify files outside the memory vault
- Expose API tokens or secrets

---

## Contact

- Email: tranthiainhi303@gmail.com
- GitHub: github.com/nhitranbtc
