# Deployment boundary and self-hosting decision

## Current decision

There is no backend deployment for the first local-shell workflow. The ordinary
local shell is a client-side PTY/process path. Deploying a new server would add
operational and security complexity without enabling this workflow.

This source checkout does not establish that the complete Cognix application can
be self-hosted. It has production Cognix endpoints in the OSS channel, proprietary
account/cloud/AI contracts, and channel configuration/update infrastructure
that cannot be replaced by a single generic backend.

## Capability classification

| Capability | Classification | Deployment implication |
|---|---|---|
| Local shell, PTY, terminal input/rendering | **LOCAL** | No backend deployment. Test local shell lifecycle on supported platforms. |
| Local preferences, session state, SQLite | **LOCAL** | Keep existing local paths and migrations; no cloud service required for the local workflow. |
| Cognix account and account-dependent actions | **REQUIRES REPLACEMENT** | A compatible identity/auth service and verified authorization contract would be needed. No replacement is selected. |
| Cognix AI operations | **REQUIRES REPLACEMENT** | Current source does not establish a general alternative-provider or local-inference contract. Define operations/data policy before deploying a provider. |
| Cloud objects and cross-device preference sync | **UNAVAILABLE WITHOUT PROPRIETARY SERVICE** | Local cached state is not a sync server. Do not claim compatibility without a tested service implementation. |
| Telemetry | **OPTIONAL REMOTE** | A fork may decide a separate explicit policy, but must disclose collection, egress, retention, and consent. |
| Crash reporting | **REQUIRES REPLACEMENT** | Use a fork-owned collector/configuration or an explicit supported opt-out; do not silently redirect to a third party. |
| Updates and release distribution | **REQUIRES REPLACEMENT** | Needs fork-owned feed, signing/notarization, provenance, rollout, and artifact verification. |
| Remote SSH/session sharing | **UNAVAILABLE WITHOUT PROPRIETARY SERVICE** until separately contracted | Not part of ordinary local PTY support. |

## Client configuration blocker

`Channel::Oss` ignores server URL overrides, intentionally preventing a shipped
release client from being redirected. Local/internal channels use generated
channel configuration, and `script/install_channel_config` pins a separate
SSH-only tool. Thus a deployment plan cannot be completed just by setting a
public environment variable and running the OSS binary.

Before attempting a self-hosted service, the fork must choose an explicit,
reviewed development/release configuration mechanism that:

- does not weaken release-channel protections;
- separates endpoint configuration from credentials;
- enforces HTTPS/WSS and certificate validation for deployed environments;
- supports test-only local endpoints without becoming a production default;
- records service version and operation compatibility;
- does not embed secrets in binaries or checked-in config.

## Future deployment gate

No server deployment is authorized by this design. A future proposal must name
the user-facing feature, publish its operation contract, define auth and data
retention, include a threat model and privacy review, and pass fake/local
service compatibility tests before deployment details are written.
