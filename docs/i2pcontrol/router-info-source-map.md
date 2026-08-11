# RouterInfo Selector Source Map

Status: exact-wire source inventory for the pinned-revision closure

This is the reviewed source map for the pinned Proposal 170 revision created and
last updated on `2026-08-11`. M027 independently revalidated the matrix against
the external revision. The machine-readable authority is
`router_info_keys::PROPOSAL_170_CONTRACT` in `emissary-cli/src/i2pcontrol/rpc.rs`.
Summary: 43 total, 37 available, 1 protocol-permitted neutral, and 5 unavailable.
M051 adjudicated the news and ban rows, M054 adjudicated transit-15s, and M055
adjudicated the v4/v6 network-error rows against the pinned proposal and
read-only reference evidence. The proposal specifies their wire types, but
Emissary has no authoritative news-feed, ban-list, request-independent rolling
transit owner, or canonical network-error owner. Empty values, request-local
sampling, and `0`/`No error` from an unset source are not authoritative
capability states and are not emitted.
The table below is intentionally one row per canonical addition. Base selectors and
the nested `Selector` compatibility form are not counted in those totals.

## Disposition vocabulary

| Disposition | Meaning |
|---|---|
| `available` | A truthful current production owner is wired and covered by a focused fixture. |
| `protocol-permitted neutral` | The proposal permits the exact neutral value, here `null` for an absent clock-skew estimate. |
| `unavailable` | No authoritative bounded source exists; the request fails with a sanitized deterministic error. |

All canonical fields use direct parameter presence: the parameter value is
ignored, including `false`, `null`, and non-boolean values. `logs.clear` is the
only mutating selector and clears only the I2PControl log ring. Every other row
is read-only. Actual serialized response size is checked after assembly.

## Canonical Proposal 170 additions

| Wire key | JSON type | Disposition | Owner / reason | Serializer | Fixture | Bound | Base alias |
|---|---|---|---|---|---|---|---|
| `i2p.router.news` | string | unavailable | router-news: no authoritative news-feed owner; empty string is not contract-proven | `serialize_router_news` | `p170.router_news.string` | — | base Router news (same wire key) |
| `i2p.router.id` | string or null | available | startup-retained | `serialize_router_id` | `p170.router_id.nullable_string` | 4 KiB | — |
| `i2p.router.clockskew` | integer or null | protocol-permitted neutral | router-info-control: null when no peer estimate exists | `serialize_clockskew` | `p170.clockskew.nullable_integer` | — | — |
| `i2p.router.info` | string or null | available | startup-retained | `serialize_router_info` | `p170.router_info.nullable_string` | 4 MiB | — |
| `i2p.router.logs` | array&lt;string&gt; | available | i2pcontrol-log-ring | `serialize_log_messages` | `p170.logs.string_list` | 10,000 / 10 MiB | — |
| `i2p.router.logs.clear` | string | available | i2pcontrol-log-ring | `serialize_log_clear` | `p170.logs_clear.success` | — | — |
| `i2p.router.net.total.received.bytes` | integer | available | event-metrics | `serialize_total_received_bytes` | `p170.total_received.integer` | — | — |
| `i2p.router.net.total.sent.bytes` | integer | available | event-metrics | `serialize_total_sent_bytes` | `p170.total_sent.integer` | — | — |
| `i2p.router.net.total.transit.bytes` | integer | available | event-metrics | `serialize_total_transit_bytes` | `p170.total_transit.integer` | — | — |
| `i2p.router.net.bw.transit.15s` | integer | unavailable | transit-bandwidth: no request-independent rolling transit owner | `serialize_transit_bandwidth_15s` | `p170.transit_15s.bytes_per_second` | — | — |
| `i2p.router.net.tunnels.shareratio` | number | available | retained-configuration | `serialize_tunnel_share_ratio` | `p170.share_ratio.number` | — | — |
| `i2p.router.net.tunnels.participating.info` | array&lt;object&gt; | available | tunnel-inspection | `serialize_participating_tunnel_info` | `p170.participating_info.rows` | 10,000 / 4 MiB | — |
| `i2p.router.net.tunnels.i2ptunnel` | array&lt;object&gt; | available | startup-tunnel-inventory | `serialize_i2ptunnel_quick_info` | `p170.i2ptunnel.quick_info` | 1,000 / 4 MiB | — |
| `i2p.router.net.tunnels.exploratory.inbound` | integer | available | tunnel-inspection | `serialize_exploratory_inbound` | `p170.exploratory_inbound.count` | — | — |
| `i2p.router.net.tunnels.exploratory.outbound` | integer | available | tunnel-inspection | `serialize_exploratory_outbound` | `p170.exploratory_outbound.count` | — | — |
| `i2p.router.net.tunnels.exploratory.info.list` | array&lt;object&gt; | available | tunnel-inspection | `serialize_exploratory_info_list` | `p170.exploratory_info.rows` | 10,000 / 4 MiB | — |
| `i2p.router.net.tunnels.client.inbound` | integer | available | tunnel-inspection | `serialize_client_inbound` | `p170.client_inbound.count` | — | — |
| `i2p.router.net.tunnels.client.outbound` | integer | available | tunnel-inspection | `serialize_client_outbound` | `p170.client_outbound.count` | — | — |
| `i2p.router.net.tunnels.client.info.list` | array&lt;object&gt; | available | tunnel-inspection | `serialize_client_info_list` | `p170.client_info.rows` | 10,000 / 4 MiB | — |
| `i2p.router.net.status.v6` | integer | available | network-state: live v6 reachability status | `serialize_network_status_v6` | `p170.status_v6.integer` | — | — |
| `i2p.router.net.error` | integer | unavailable | network-error: no canonical network-error owner | `unavailable` | `p170.error_v4.integer` | — | — |
| `i2p.router.net.error.v6` | integer | unavailable | network-error: no canonical network-error owner | `unavailable` | `p170.error_v6.integer` | — | — |
| `i2p.router.net.testing` | integer | available | network-state: active v4 reachability test | `serialize_network_testing` | `p170.testing_v4.integer` | — | — |
| `i2p.router.net.testing.v6` | integer | available | network-state: active v6 reachability test | `serialize_network_testing_v6` | `p170.testing_v6.integer` | — | — |
| `i2p.router.net.tunnels.successrate` | number | available | tunnel-build-metrics: ordered reference EWMA | `serialize_tunnel_success_rate` | `p170.success_rate.recent.percent` | — | — |
| `i2p.router.net.tunnels.totalsuccessrate` | number | available | event-metrics | `serialize_total_tunnel_success_rate` | `p170.success_rate.total.percent` | — | — |
| `i2p.router.net.tunnels.queue` | integer | available | tunnel-inspection: live pending build depth | `serialize_tunnel_queue` | `p170.tunnel_queue.depth` | bounded | — |
| `i2p.router.net.tunnels.tbmqueue` | integer | available | tunnel-inspection: live transit build-message depth | `serialize_tbm_queue` | `p170.tbm_queue.depth` | bounded | — |
| `i2p.router.netdb.peers` | array&lt;string&gt; | available | live-profile-storage-inspection | `serialize_netdb_peer_hashes` | `p170.netdb.peer_hashes` | 10,000 / 4 MiB | — |
| `i2p.router.netdb.activepeers.info` | array&lt;string&gt; | available | transport-and-live-profile-storage-inspection | `serialize_active_peer_router_infos` | `p170.netdb.active_peer_info` | 10,000 / 4 MiB | — |
| `i2p.router.netdb.ntcp.limit` | integer | available | transport-manager-configuration | `serialize_ntcp_limit` | `p170.netdb.ntcp_limit` | — | — |
| `i2p.router.netdb.ssu.limit` | integer | available | transport-manager-configuration | `serialize_ssu_limit` | `p170.netdb.ssu_limit` | — | — |
| `i2p.router.netdb.bannedpeers` | map&lt;string,map&lt;string,object&gt;&gt; | unavailable | ban-list: no authoritative ban owner; empty map is not contract-proven | `serialize_banned_peers` | `p170.netdb.banned_peers.unavailable` | 10,000 / 4 MiB | — |
| `i2p.router.netdb.activepeers.list` | array&lt;string&gt; | available | transport-manager-inspection | `serialize_active_peer_hashes` | `p170.netdb.active_peers` | 10,000 / 4 MiB | — |
| `i2p.router.netdb.peers.list` | array&lt;string&gt; | available | live-profile-storage-inspection | `serialize_known_peer_hashes` | `p170.netdb.peer_list` | 10,000 / 4 MiB | — |
| `i2p.router.netdb.peers.info` | array&lt;string&gt; | available | live-profile-storage-inspection | `serialize_peer_router_infos` | `p170.netdb.peer_info` | 10,000 / 4 MiB | — |
| `i2p.router.netdb.activepeers.stats` | array&lt;object&gt; | available | transport-manager-inspection: bounded current active session facts | `serialize_active_peer_stats` | `p170.netdb.active_peer_stats` | 10,000 / 4 MiB | — |
| `i2p.router.addressbook.private.list` | array&lt;map&lt;string,string&gt;&gt; | available | runtime-address-book-handle | `serialize_address_book_private_list` | `p170.addressbook.private.list` | 10,000 / 4 MiB | — |
| `i2p.router.addressbook.local.list` | array&lt;map&lt;string,string&gt;&gt; | available | runtime-address-book-handle | `serialize_address_book_local_list` | `p170.addressbook.local.list` | 10,000 / 4 MiB | — |
| `i2p.router.addressbook.router.list` | array&lt;map&lt;string,string&gt;&gt; | available | runtime-address-book-handle | `serialize_address_book_router_list` | `p170.addressbook.router.list` | 10,000 / 4 MiB | — |
| `i2p.router.addressbook.published.list` | array&lt;map&lt;string,string&gt;&gt; | available | runtime-address-book-handle | `serialize_address_book_published_list` | `p170.addressbook.published.list` | 10,000 / 4 MiB | — |
| `i2p.router.addressbook.subscriptions` | object `{path,entries}` | available | runtime-address-book-handle | `serialize_address_book_subscriptions` | `p170.addressbook.subscriptions.object` | 1,000 / 4 MiB | base subscriptions (same wire key) |
| `i2p.router.addressbook.config` | object `{path,entries}` | available | runtime-address-book-handle | `serialize_address_book_config` | `p170.addressbook.config.object` | 1,000 / 4 MiB | base config (same wire key) |

## Base and compatibility separation

The existing base inventory remains `CORE_KEYS` (115 selectors) plus
`ADDRESS_BOOK_KEYS` (6 selectors), for 121 unique names. The canonical set is
the 43-row table above. The intentional exact-name overlap is limited to
Router news and the two address-book object selectors; the source matrix records those aliases
explicitly. The compatibility nested `Selector` form is an envelope and is
never counted as a selector addition.

## Request and failure rules

All requested canonical keys are validated against this matrix before any
source query. If any requested canonical field is unavailable, the entire
request returns one sanitized internal error and no partial result. Neutral
fields serialize only their protocol-permitted neutral value. Available empty
lists and zero counters are valid only when the authoritative source was
queried successfully. No aggregate counter substitutes for a transport,
peer, queue, or tunnel-pool field.
