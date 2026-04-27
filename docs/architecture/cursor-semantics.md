# Cursor Semantics

Kerald uses nanosecond timestamp cursors for client-visible progress. A cursor
is Arrow's native signed 64-bit timestamp-nanosecond value, described by
`DataType::Timestamp(TimeUnit::Nanosecond, None)` and represented to clients as
nanoseconds since the Unix epoch. It is ordered only by timestamp value and does
not imply partition ownership, broker-local sequence ownership, or storage
layout.

Timestamp cursor ranges are inclusive and form the initial polling boundary for
payload reads. Brokers must validate that a requested range is ordered before it
is used by delivery or persistence paths.

Future delivery, notification, and persistence work should carry
the Arrow timestamp-nanosecond cursor scalar through APIs rather than
introducing numeric positions. When storage needs physical addressing, that
address remains an internal persistence concern and must not become
client-visible progress.
