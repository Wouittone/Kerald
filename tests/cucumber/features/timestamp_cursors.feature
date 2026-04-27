Feature: Timestamp cursor progress
  Kerald clients track progress with nanosecond timestamp cursors.

  Scenario: Payload polling uses timestamp cursor ranges
    Given the earliest payload timestamp cursor is 100 nanoseconds
    And the latest payload timestamp cursor is 300 nanoseconds
    And a payload timestamp cursor is 200 nanoseconds
    When a client opens an inclusive timestamp cursor range
    Then the payload is visible in the polling range
    And the client sees nanosecond timestamp value 200
