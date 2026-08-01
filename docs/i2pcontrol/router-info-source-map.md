# RouterInfo Selector Source Map

Status: M026 bounded-source disposition accepted; final subsystem closure remains M027

This is the reviewed source map for the pinned Proposal 170 revision created and
last updated on `2026-08-01`. The machine-readable authority is
`router_info_keys::PROPOSAL_170_CONTRACT` in `emissary-cli/src/i2pcontrol/rpc.rs`.
Summary: 43 total, 16 available, 1 protocol-permitted neutral, and 26 unavailable.
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
| `i2p.router.news` | string | unavailable | router-news: no router news owner | `serialize_router_news` | `p170.router_news.string` | — | base Router news (same wire key) |
| `i2p.router.id` | string or null | available | startup-retained | `serialize_router_id` | `p170.router_id.nullable_string` | 4 KiB | — |
| `i2p.router.clockskew` | integer or null | protocol-permitted neutral | router-info-control: null when no peer estimate exists | `serialize_clockskew` | `p170.clockskew.nullable_integer` | — | — |
| `i2p.router.info` | string or null | available | startup-retained | `serialize_router_info` | `p170.router_info.nullable_string` | 4 MiB | — |
| `i2p.router.logs` | array&lt;string&gt; | available | i2pcontrol-log-ring | `serialize_log_messages` | `p170.logs.string_list` | 10,000 / 10 MiB | — |
| `i2p.router.logs.clear` | string | available | i2pcontrol-log-ring | `serialize_log_clear` | `p170.logs_clear.success` | — | — |
| `i2p.router.net.total.received.bytes` | integer | available | event-metrics | `serialize_total_received_bytes` | `p170.total_received.integer` | — | — |
| `i2p.router.net.total.sent.bytes` | integer | available | event-metrics | `serialize_total_sent_bytes` | `p170.total_sent.integer` | — | — |
| `i2p.router.net.total.transit.bytes` | integer | available | event-metrics | `serialize_total_transit_bytes` | `p170.total_transit.integer` | — | — |
| `i2p.router.net.bw.transit.15s` | integer | unavailable | traffic-metrics: no rolling 15s transit source | `serialize_transit_bandwidth_15s` | `p170.transit_15s.unavailable` | — | — |
| `i2p.router.net.tunnels.shareratio` | number | available | retained-configuration | `serialize_tunnel_share_ratio` | `p170.share_ratio.number` | — | — |
| `i2p.router.net.tunnels.participating.info` | array&lt;object&gt; | unavailable | tunnel-pool: no bounded participating tunnel detail snapshot | `serialize_participating_tunnel_info` | `p170.participating_info.unavailable` | 10,000 / 4 MiB | — |
| `i2p.router.net.tunnels.i2ptunnel` | array&lt;object&gt; | available | startup-tunnel-inventory | `serialize_i2ptunnel_quick_info` | `p170.i2ptunnel.quick_info` | 1,000 / 4 MiB | — |
| `i2p.router.net.tunnels.exploratory.inbound` | integer | unavailable | tunnel-pool: no bounded exploratory tunnel count source | `serialize_exploratory_inbound` | `p170.exploratory_inbound.unavailable` | — | — |
| `i2p.router.net.tunnels.exploratory.outbound` | integer | unavailable | tunnel-pool: no bounded exploratory tunnel count source | `serialize_exploratory_outbound` | `p170.exploratory_outbound.unavailable` | — | — |
| `i2p.router.net.tunnels.exploratory.info.list` | array&lt;object&gt; | unavailable | tunnel-pool: no bounded exploratory tunnel detail snapshot | `serialize_exploratory_info_list` | `p170.exploratory_info.unavailable` | 10,000 / 4 MiB | — |
| `i2p.router.net.tunnels.client.inbound` | integer | unavailable | tunnel-pool: no bounded client tunnel count source | `serialize_client_inbound` | `p170.client_inbound.unavailable` | — | — |
| `i2p.router.net.tunnels.client.outbound` | integer | unavailable | tunnel-pool: no bounded client tunnel count source | `serialize_client_outbound` | `p170.client_outbound.unavailable` | — | — |
| `i2p.router.net.tunnels.client.info.list` | array&lt;object&gt; | unavailable | tunnel-pool: no bounded client tunnel detail snapshot | `serialize_client_info_list` | `p170.client_info.unavailable` | 10,000 / 4 MiB | — |
| `i2p.router.net.status.v6` | integer | unavailable | network: no transport-specific v6 status code mapping | `serialize_network_status_v6` | `p170.status_v6.unavailable` | — | — |
| `i2p.router.net.error` | integer | unavailable | network: no transport-specific v4 error code mapping | `serialize_network_error` | `p170.error_v4.unavailable` | — | — |
| `i2p.router.net.error.v6` | integer | unavailable | network: no transport-specific v6 error code mapping | `serialize_network_error_v6` | `p170.error_v6.unavailable` | — | — |
| `i2p.router.net.testing` | integer | unavailable | network: no canonical v4 testing-state source | `serialize_network_testing` | `p170.testing_v4.unavailable` | — | — |
| `i2p.router.net.testing.v6` | integer | unavailable | network: no canonical v6 testing-state source | `serialize_network_testing_v6` | `p170.testing_v6.unavailable` | — | — |
| `i2p.router.net.tunnels.successrate` | number | unavailable | tunnel-build-metrics: no rolling tunnel build success-rate source | `serialize_tunnel_success_rate` | `p170.success_rate.recent.unavailable` | — | — |
| `i2p.router.net.tunnels.totalsuccessrate` | number | available | event-metrics | `serialize_total_tunnel_success_rate` | `p170.success_rate.total.percent` | — | — |
| `i2p.router.net.tunnels.queue` | integer | unavailable | tunnel-pool: no bounded tunnel build queue snapshot | `serialize_tunnel_queue` | `p170.tunnel_queue.unavailable` | — | — |
| `i2p.router.net.tunnels.tbmqueue` | integer | unavailable | tunnel-pool: no bounded tunnel build message queue snapshot | `serialize_tbm_queue` | `p170.tbm_queue.unavailable` | — | — |
| `i2p.router.netdb.peers` | array&lt;string&gt; | unavailable | netdb: no bounded known-peer hash snapshot | `serialize_netdb_peer_hashes` | `p170.netdb.peers.unavailable` | 10,000 / 4 MiB | — |
| `i2p.router.netdb.activepeers.info` | array&lt;string&gt; | unavailable | netdb: no bounded active-peer RouterInfo snapshot | `serialize_active_peer_router_infos` | `p170.netdb.active_peer_info.unavailable` | 10,000 / 4 MiB | — |
| `i2p.router.netdb.ntcp.limit` | integer | unavailable | peer-limits: no authoritative NTCP limit owner | `serialize_ntcp_limit` | `p170.netdb.ntcp_limit.unavailable` | — | — |
| `i2p.router.netdb.ssu.limit` | integer | unavailable | peer-limits: no authoritative SSU limit owner | `serialize_ssu_limit` | `p170.netdb.ssu_limit.unavailable` | — | — |
| `i2p.router.netdb.bannedpeers` | map&lt;string,map&lt;string,object&gt;&gt; | unavailable | ban-list: no authoritative ban owner | `serialize_banned_peers` | `p170.netdb.banned_peers.unavailable` | 10,000 / 4 MiB | — |
| `i2p.router.netdb.activepeers.list` | array&lt;string&gt; | unavailable | peer-list: no bounded active peer RouterInfo snapshot | `serialize_active_peer_hashes` | `p170.netdb.active_peers.unavailable` | 10,000 / 4 MiB | — |
| `i2p.router.netdb.peers.list` | array&lt;string&gt; | unavailable | peer-list: no bounded known peer RouterInfo snapshot | `serialize_known_peer_hashes` | `p170.netdb.peer_list.unavailable` | 10,000 / 4 MiB | — |
| `i2p.router.netdb.peers.info` | array&lt;string&gt; | unavailable | peer-list: no bounded peer RouterInfo snapshot | `serialize_peer_router_infos` | `p170.netdb.peer_info.unavailable` | 10,000 / 4 MiB | — |
| `i2p.router.netdb.activepeers.stats` | array&lt;object&gt; | unavailable | peer-stats: no bounded active peer statistics snapshot | `serialize_active_peer_stats` | `p170.netdb.active_peer_stats.unavailable` | 10,000 / 4 MiB | — |
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
