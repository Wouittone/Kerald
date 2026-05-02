Feature: Payload storage polling
  Kerald stores Arrow payloads independently from notification tracking and
  lets clients poll payloads with nanosecond timestamp cursors.

  Scenario: Payload polling uses strict nanosecond timestamp cursors
    Given local payload storage is initialized
    And partitionless topic "orders.received" has payloads at cursors 100 and 200
    When payloads are polled after cursor 100
    Then only payload cursor 200 is returned
    And no partition or offset input is required for payload polling

  Scenario: Payload polling is empty before a topic has stored payloads
    Given local payload storage is initialized
    And partitionless topic "orders.received" has no stored payloads
    When payloads are polled after cursor 0
    Then no payload batches are returned

  Scenario: Stored payloads remain pollable after local storage is reopened
    Given local payload storage is initialized
    And partitionless topic "orders.received" has payloads at cursors 100 and 200
    When local payload storage is reopened
    And payloads are polled after cursor 100
    Then only payload cursor 200 is returned
    And no partition or offset input is required for payload polling
