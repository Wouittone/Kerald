Feature: Broker cluster startup
  Kerald brokers always start from cluster configuration without changing
  client-facing topic semantics.

  Scenario: Single-node cluster starts with local write admission enabled
    Given a broker is configured for a single-node cluster
    When the broker starts
    Then the cluster quorum is 1
    And write admission is enabled for local operation

  Scenario: Multi-node cluster starts with inter-broker communication settings
    Given the expected cluster size is 3
    And the inter-broker port is 9000
    When the broker starts
    Then a broker UUID is generated
    Then the cluster quorum is 2
    And write admission is rejected until voter discovery reaches quorum
