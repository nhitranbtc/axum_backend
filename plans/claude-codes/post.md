# Post Feature: Senior Review and Refactored Plan

## 1. Review Findings (Critical First)

1. Pagination strategy is inconsistent.
- The document says Scylla-native cursor paging is preferred, but API logic later says `LIMIT + OFFSET`.
- ScyllaDB does not support offset pagination efficiently; large offsets cause full scans.
- Decision: keep API parameters `page`, `pageSize`, `status` (as requested), but implement internally with cursor/paging-state and return pagination metadata.

2. Data model and API model are partially misaligned.
- Core fields include `slug` and `tags`, but recommended table schema omitted both in some sections.
- Decision: include `slug` and `tags` in source-of-truth table, and include projected fields in read-model tables.

3. `JSONB` is PostgreSQL-specific.
- This backend uses ScyllaDB.
- Decision: store rich content as `TEXT` JSON payload (or `BLOB` if needed later).

4. Read patterns need explicit table-per-query mapping.
- The current plan mentions access patterns but does not clearly map each endpoint to a specific table/query.
- Decision: define source table + read tables and endpoint-query mapping.

5. Slug uniqueness needs explicit scope.
- “Unique slug” can mean global uniqueness or per-author uniqueness.
- Decision: use global unique slug with collision suffix strategy (`my-title`, `my-title-2`, ...).

## 2. Objective
Deliver a production-grade Post feature in existing Axum + ScyllaDB architecture with strong API contracts, predictable query performance, and maintainable DDD boundaries.

## 3. Scope

### 3.1 MVP
- `POST /api/posts` (authenticated)
- `GET /api/posts/:id`
- `GET /api/posts` with params `page`, `pageSize`, `status`

### 3.2 Phase 2
- `PUT /api/posts/:id` (author/admin)
- `DELETE /api/posts/:id` (soft delete)
- `GET /api/users/:id/posts`

### 3.3 Non-Goals
- Comments/reactions
- Full-text search
- Media processing pipeline
- Moderation workflow automation

## 4. Domain Design

### 4.1 Entity: `Post`
- `id: UUID/ULID`
- `title: String` (max 255)
- `slug: String` (URL friendly, globally unique)
- `content: String` (JSON text allowed for rich content)
- `author_id: UUID`
- `status: PostStatus` (`draft`, `published`, `archived`)
- `tags: Vec<String>`
- `created_at: DateTime<Utc>`
- `updated_at: DateTime<Utc>`
- `published_at: Option<DateTime<Utc>>`
- `deleted_at: Option<DateTime<Utc>>`

### 4.2 Invariants
- `title` non-empty, max 255
- `slug` generated from title, normalized lowercase `a-z0-9-`
- `content` non-empty
- status transition rules:
  - `draft -> published|archived`
  - `published -> archived`
  - `archived -> draft` only if explicitly allowed by product rule

### 4.3 Repository Contract
- `create(post)`
- `find_by_id(id)`
- `find_by_slug(slug)`
- `list_recent(status, page_size, cursor)`
- `list_by_author(author_id, status, page_size, cursor)`
- `update(post)`
- `soft_delete(id)`

### 4.4 Post Structure
Rust-style structure (domain model target):

```rust
pub struct Post {
    pub id: uuid::Uuid,
    pub author_id: uuid::Uuid,
    pub title: String,
    pub slug: String,
    pub content: String, // plain text or JSON string
    pub status: PostStatus, // draft | published | archived
    pub tags: Vec<String>,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
```

API response shape (example):

```json
{
  "id": "a8a40cf1-8c8d-4f13-9abc-312f6f9cd501",
  "author_id": "d7d7f54a-89ec-4cb5-8c9a-c2467fce6f1c",
  "title": "Building Post APIs with Axum",
  "slug": "building-post-apis-with-axum",
  "content": "Post body content",
  "status": "draft",
  "tags": ["rust", "axum", "scylla"],
  "created_at": "2026-03-06T12:00:00Z",
  "updated_at": "2026-03-06T12:00:00Z"
}
```

## 5. Data Model Design (ScyllaDB)

Scylla is query-first; model by read patterns.

### 5.1 Canonical Field Spec
| Field | Type | Description |
|---|---|---|
| `id` | `UUID / ULID` | Primary key. ULID recommended for sortable indexing. |
| `title` | `String` | Headline of the post (max 255 chars). |
| `slug` | `String` | Unique, URL-friendly string generated from title. |
| `content` | `Text / JSON` | Body of the post. Rich text stored as JSON string. |
| `author_id` | `UUID` | Links to Users domain identity. |
| `status` | `Enum` | `draft`, `published`, `archived`. |
| `tags` | `Array<String>` | Categorization and SEO tags. |
| `created_at` | `Timestamp` | Auto-set on insert. |
| `updated_at` | `Timestamp` | Auto-updated on changes. |

### 5.2 Tables

1. `posts_by_id` (source of truth)
- Primary key: `(post_id)`
- Columns: all canonical fields + lifecycle fields (`published_at`, `deleted_at`)

2. `posts_by_slug` (lookup)
- Primary key: `(slug)`
- Columns: `post_id`, `author_id`, `status`, `created_at`

3. `posts_by_author`
- Primary key: `((author_id), created_at, post_id)` with desc clustering
- Columns: `title`, `slug`, `status`, `tags`, `deleted_at`

4. `posts_feed_by_day` (recommended for global feed)
- Primary key: `((bucket_date), created_at, post_id)` with desc clustering
- Columns: `author_id`, `title`, `slug`, `status`, `tags`, `deleted_at`

### 5.3 Write Strategy
- Write to `posts_by_id` first.
- Then upsert to read models (`posts_by_slug`, `posts_by_author`, `posts_feed_by_day`).
- If partial failure occurs, log + retry policy + scheduled repair/outbox.

### 5.4 Pagination Strategy
- External API accepts `page`, `pageSize`, `status`.
- Internal DB queries use cursor/paging state, not true offset scans.
- Response metadata includes:
  - `page`
  - `pageSize`
  - `totalItems` (optional/estimated for Scylla)
  - `totalPages` (optional/estimated)
  - `nextCursor` (authoritative for deep pagination)

## 6. API Endpoint Strategy

### 6.1 `POST /api/posts`
- Auth: required
- Body: `{ title, content, status?, tags? }`
- Rules:
  - derive `author_id` from JWT subject
  - generate slug
  - if slug exists, append numeric suffix
- Response: `201` with `PostResponseDto`

### 6.2 `GET /api/posts/:id`
- Auth: optional/required per policy
- Returns one post if not soft-deleted
- Status codes: `200`, `404`

### 6.3 `GET /api/posts`
- Parameters: `page`, `pageSize`, `status`
- Logic: query with `LIMIT`; map `page` to cursor internally for scalable paging
- Returns:
  - `items: PostResponseDto[]`
  - metadata: `page`, `pageSize`, `totalItems`, `totalPages`, `nextCursor`

### 6.4 `PUT /api/posts/:id`
- Auth: required, author/admin only
- Editable fields:
  - `title: String` (regenerates slug)
  - `content: Text/JSON`
  - `status: Enum` (`draft`, `published`, `archived`)
  - `tags: string[]`
- Response: `200` with updated post

### 6.5 `DELETE /api/posts/:id`
- Auth: required, author/admin only
- Behavior: soft delete (`deleted_at` set)
- Response: `200`/`204`

## 7. DTO Strategy
- `CreatePostDto { title, content, status?: PostStatus, tags?: Vec<String> }`
- `UpdatePostDto { title?: String, content?: String, status?: PostStatus, tags?: Vec<String> }`
- `PostResponseDto { id, title, slug, content, author_id, status, tags, created_at, updated_at }`
- `PostListResponseDto { items, page, pageSize, totalItems, totalPages, nextCursor }`

## 8. Authorization Rules
- Create: any authenticated user
- Update/Delete: post author or admin
- Never accept `author_id` from request body

## 9. Testing Plan

### 9.1 Unit
- slug generation and collision handling
- status transition validation
- title/content/tags validation

### 9.2 Integration API
- create post success (`201`)
- create post without auth (`401`)
- invalid payload (`400`)
- list with filters and metadata
- update allowed/forbidden cases (`200/403`)
- soft delete and subsequent `404`

### 9.3 Repository/Scylla
- id lookup consistency
- slug lookup uniqueness behavior
- author listing order
- feed bucket query performance sanity

## 10. Implementation Roadmap
1. Finalize schema + repository interfaces.
2. Add `GET /api/posts/:id` and `GET /api/posts` with metadata contract.
3. Add `PUT /api/posts/:id` (editable fields + slug regeneration).
4. Add soft delete flow.
5. Expand tests and Swagger docs.

## 11. Definition of Done
- All post endpoints documented and implemented
- Green tests for unit + API + repository layers
- No hot-path `ALLOW FILTERING`
- Authz checks proven by tests
- Observability added (logs + endpoint latency metrics)

## 12. Concrete Task Checklist (Exact File/Function Mapping)

### 12.1 Domain Layer
- [x] Extend `Post` entity fields in `src/domain/entities/post.rs`
  - Target struct: `Post`
  - Add fields: `slug`, `status`, `tags`, `published_at`, `deleted_at`
  - Update constructors: `Post::new`, `Post::from_existing`
  - Add methods: `update_content`, `update_title_and_slug`, `update_status`, `soft_delete`
- [x] Add/extend post status type
  - Option A: `src/domain/value_objects/post_status.rs`
  - Option B: enum in `src/domain/entities/post.rs`
- [x] Expand repository trait in `src/domain/repositories/post.rs`
  - Keep: `save`
  - Add: `find_by_id`, `find_by_slug`, `list_recent`, `list_by_author`, `update`, `soft_delete`
  - Add error variants: `NotFound`, `Conflict`, `Internal`

### 12.2 Application Layer (DTO + Use Cases)
- [x] Extend DTOs in `src/application/dto/post.rs`
  - Add request DTOs: `UpdatePostDto`, `ListPostsQueryDto`
  - Extend `CreatePostDto` with optional `status`, `tags`
  - Extend `PostResponseDto` with `slug`, `status`, `tags`
  - Add list wrapper DTO: `PostListResponseDto`
- [x] Register DTO exports in `src/application/dto/mod.rs` (already has `post`; keep synced)
- [x] `CreatePostUseCase` exists in `src/application/use_cases/post/create.rs`
- [x] Add use case files:
  - `src/application/use_cases/post/get.rs` -> `GetPostUseCase::execute(id)`
  - `src/application/use_cases/post/list.rs` -> `ListPostsUseCase::execute(page, page_size, status)`
  - `src/application/use_cases/post/update.rs` -> `UpdatePostUseCase::execute(actor_id, id, dto)`
  - `src/application/use_cases/post/delete.rs` -> `DeletePostUseCase::execute(actor_id, id)`
- [x] Update exports in:
  - `src/application/use_cases/post/mod.rs`
  - `src/application/use_cases/mod.rs`

### 12.3 Infrastructure Layer (Scylla)
- [x] Update schema in `src/infrastructure/database/scylla/connection.rs`
  - Update `CREATE TABLE posts` columns to include `slug`, `status`, `tags`, `published_at`, `deleted_at`
  - Add read-model tables if adopted: `posts_by_slug`, `posts_by_author`, `posts_feed_by_day`
  - Add required indexes only for non-primary-key lookup paths
- [x] Update row model in `src/infrastructure/database/scylla/models/post.rs`
  - Extend `PostRow` fields and static CQL constants
  - Add query constants for list and lookup patterns
- [x] Expand repository implementation in `src/infrastructure/database/scylla/repositories/post.rs`
  - Keep: `RepositoryImpl::new`, `save`
  - Add: `find_by_id`, `find_by_slug`, `list_recent`, `list_by_author`, `update`, `soft_delete`
  - Add row<->entity conversion helpers
- [x] Keep exports wired:
  - `src/infrastructure/database/scylla/models/mod.rs`
  - `src/infrastructure/database/scylla/repositories/mod.rs`
  - `src/infrastructure/database/scylla/mod.rs`
  - `src/infrastructure/database/mod.rs`
  - `src/infrastructure/mod.rs`

### 12.4 Presentation Layer (Routes + Handlers + Docs)
- [x] `create_post` exists in `src/presentation/handlers/post.rs`
- [x] Add handlers in `src/presentation/handlers/post.rs`
  - `get_post`
  - `list_posts`
  - `update_post`
  - `delete_post`
- [x] Update route wiring in `src/presentation/routes/posts.rs`
  - Keep existing `POST /`
  - Add:
    - `GET /:id`
    - `GET /`
    - `PUT /:id`
    - `DELETE /:id`
  - Bind each route with correct `State<Arc<...UseCase>>`
- [x] Update exports in `src/presentation/handlers/mod.rs` and `src/presentation/routes/mod.rs`
- [x] Update Swagger registration in `src/presentation/routes/mod.rs`
  - Add new handler paths in `#[openapi(paths(...))]`
  - Add new schemas in `components(schemas(...))`
- [x] Add response wrappers in `src/presentation/responses/mod.rs`
  - Keep: `PostResponseWrapper`
  - Add: `PostListResponseWrapper`

### 12.5 Authorization & Slug Rules
- [x] In `update_post` handler/use case, enforce actor is author or admin
  - Actor source: JWT claims (`Claims.sub`) from `src/presentation/middleware/auth.rs`
  - Role check source: existing user role facilities (`role` use cases/handlers)
- [x] Implement slug generation and collision policy in use case layer
  - Suggested location: `src/application/use_cases/post/create.rs` and `update.rs`
  - Collision lookup method: repository `find_by_slug`

### 12.6 Testing
- [x] Create endpoint API tests exist in `tests/api/post.rs`
  - `test_create_post_success_with_bearer_token`
  - `test_create_post_requires_authentication`
  - `test_create_post_validation_error`
- [x] Add API tests in `tests/api/post.rs`
  - `test_get_post_success`
  - `test_get_post_not_found`
  - `test_list_posts_with_page_page_size_status`
  - `test_update_post_authorized`
  - `test_update_post_forbidden`
  - `test_delete_post_soft_delete`
- [x] Add integration repository tests
  - Suggested file: `tests/integration/scylla/post.rs`
  - Update module file: `tests/integration/scylla/mod.rs`
- [x] Ensure test module registration in `tests/api_tests.rs` (already includes `pub mod post;`)

### 12.7 Verification Commands
- [x] `cargo fmt`
- [x] `cargo check`
- [x] `cargo test --test api_tests post -- --nocapture`
- [x] `cargo test --test integration_tests` (or targeted Scylla integration module)

## 13. Post Feature Tree Structure

```text
axum_backend/
├── plans/
│   └── claude-codes/
│       └── post.md
├── src/
│   ├── domain/
│   │   ├── entities/
│   │   │   ├── mod.rs
│   │   │   └── post.rs
│   │   ├── repositories/
│   │   │   ├── mod.rs
│   │   │   └── post.rs
│   │   └── mod.rs
│   ├── application/
│   │   ├── dto/
│   │   │   ├── mod.rs
│   │   │   └── post.rs
│   │   └── use_cases/
│   │       ├── mod.rs
│   │       └── post/
│   │           ├── mod.rs
│   │           ├── create.rs
│   │           ├── get.rs
│   │           ├── list.rs
│   │           ├── update.rs
│   │           └── delete.rs
│   ├── infrastructure/
│   │   ├── database/
│   │   │   ├── mod.rs
│   │   │   └── scylla/
│   │   │       ├── connection.rs
│   │   │       ├── mod.rs
│   │   │       ├── models/
│   │   │       │   ├── mod.rs
│   │   │       │   └── post.rs
│   │   │       └── repositories/
│   │   │           ├── mod.rs
│   │   │           └── post.rs
│   │   └── mod.rs
│   ├── presentation/
│   │   ├── handlers/
│   │   │   ├── mod.rs
│   │   │   └── post.rs
│   │   ├── responses/
│   │   │   └── mod.rs
│   │   ├── routes/
│   │   │   ├── mod.rs
│   │   │   └── posts.rs
│   │   └── mod.rs
│   └── shared/
│       └── errors/
│           └── mod.rs
└── tests/
    ├── api/
    │   └── post.rs
    ├── api_tests.rs
    └── common/
        └── server.rs
```
