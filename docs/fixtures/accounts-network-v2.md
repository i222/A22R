# Synthetic account-network fixture

`accounts-network-v2.json` is an importable JSON v2 dataset for relationship
and search overviews. It contains 60 items, 141 directed item links, four
timers, and four item-timer links.

The fixture is intentionally safe to share:

- every email uses the reserved `.invalid` top-level domain;
- every phone uses the reserved North American `202-555-01xx` range;
- account, card, customer, policy, and device identifiers are marked `DEMO`;
- no real credentials, tokens, or payment numbers are present.

The dataset is split into five connected groups:

- identity anchors: phones, mailboxes, and two profile records;
- finance: fictional banks, cards, wallets, and transfer services;
- service accounts: cloud, security, messaging, shopping, travel, work, and
  infrastructure;
- bills and merchants: utilities, insurance, subscriptions, purchases,
  donations, payroll, accounting, and invoices;
- recovery and inventory: devices, backup codes, document vault, contacts, and
  the account index.

## Suggested relationship overview

Select the item in the list, open its Relationships view, and let the graph
settle before reviewing the result. The graph shows only direct neighbors of
the selected item.

![A22R item workspace with the Phone Work fixture item selected](../screenshots/Screenshot%202026-11-24%20at%2021.58.10.png)

*Figure 1. The fixture loaded into the item workspace with the `phone` tag
filter active and `Phone · Work` selected.*

![A22R relationship graph for the Phone Work fixture item](../screenshots/Screenshot%202026-11-24%20at%2021.58.41.png)

*Figure 2. Direct fixture neighbors and their typed `sms_phone` relationships
around `Phone · Work`.*

- `phone-primary` — broad personal sign-in cluster with 19 neighbors:
  banks, wallets, payments, messaging, shopping, travel, utilities, device,
  and both secondary profiles.
- `email-work` — work and finance cluster with 14 neighbors:
  Northstar Bank, Nimbus Drive, Task Harbor, Forgebox, Quiet Tunnel, hosting,
  tax, payroll, accounting, invoices, and Dana.
- `email-recovery` — compact recovery cluster with nine neighbors:
  Cedarline, Keyforge, Carepath, support, insurance, the recovery mailbox,
  document vault, and the Cedarline virtual card.
- `card-northstar` — payment fan-out with ten neighbors:
  its bank, wallet, shopping, VPN, smart-home, domain, hosting, insurance, and
  home-supply records.
- `wallet-aurora` — mixed payment cluster with eight neighbors:
  main email, primary phone, card, delivery, ride, coffee merchant, device,
  and the inventory note.
- `password-keyforge` — security cluster with eight neighbors:
  both recovery mailboxes, primary phone, trusted MacBook, Forgebox, backup
  codes, document vault, and the inventory note.
- `profile-shopping` — small focused cluster with five neighbors for a less
  crowded overview.
- `profile-travel` — another five-neighbor cluster combining travel, rail,
  wallet, and the main identity anchors.

Relationship types are intentionally explicit (`login_email`, `sms_phone`,
`paid_with`, `recovery_email`, `trusted_device`, and similar) so incoming and
outgoing connections remain readable in the details panel.
