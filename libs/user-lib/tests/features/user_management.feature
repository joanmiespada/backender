Feature: User Management
  As a system administrator
  I want to manage users in the system
  So that I can control access and maintain user data

  Background:
    Given a clean user database

  Scenario: Create a new user
    When I create a user with keycloak_id "kc-john-123"
    Then the user should be created successfully
    And the user should have keycloak_id "kc-john-123"
    And the user should have no roles

  Scenario: Retrieve an existing user
    Given a user exists with keycloak_id "kc-alice-456"
    When I retrieve the user by their ID
    Then the user should be found
    And the user should have keycloak_id "kc-alice-456"

  Scenario: Retrieve a non-existent user
    When I try to retrieve a user with a random ID
    Then the user should not be found

  Scenario: Delete a user
    Given a user exists with keycloak_id "kc-charlie-789"
    When I delete the user
    Then the deletion should be successful
    And the user should no longer exist

  Scenario: List users with pagination
    Given the following users exist:
      | keycloak_id    |
      | kc-user1       |
      | kc-user2       |
      | kc-user3       |
      | kc-user4       |
      | kc-user5       |
    When I request page 1 with page size 2
    Then I should receive 2 users
    And the total count should be 5
    And the total pages should be 3
