Feature: User Management
  As a system administrator
  I want to manage users in the system
  So that I can control access and maintain user data

  Background:
    Given a clean user database

  Scenario: Create a new user
    When I create a user with name "John Doe" and email "john@example.com"
    Then the user should be created successfully
    And the user should have name "John Doe"
    And the user should have email "john@example.com"
    And the user should have no roles

  Scenario: Prevent duplicate email registration
    Given a user exists with name "Jane Doe" and email "jane@example.com"
    When I try to create a user with name "Another Jane" and email "jane@example.com"
    Then I should receive an email already exists error

  Scenario: Retrieve an existing user
    Given a user exists with name "Alice Smith" and email "alice@example.com"
    When I retrieve the user by their ID
    Then the user should be found
    And the user should have name "Alice Smith"

  Scenario: Retrieve a non-existent user
    When I try to retrieve a user with a random ID
    Then the user should not be found

  Scenario: Update user information
    Given a user exists with name "Bob Wilson" and email "bob@example.com"
    When I update the user's name to "Robert Wilson" and email to "robert@example.com"
    Then the update should be successful
    And the user should have name "Robert Wilson"
    And the user should have email "robert@example.com"

  Scenario: Delete a user
    Given a user exists with name "Charlie Brown" and email "charlie@example.com"
    When I delete the user
    Then the deletion should be successful
    And the user should no longer exist

  Scenario: List users with pagination
    Given the following users exist:
      | name          | email               |
      | User One      | user1@example.com   |
      | User Two      | user2@example.com   |
      | User Three    | user3@example.com   |
      | User Four     | user4@example.com   |
      | User Five     | user5@example.com   |
    When I request page 1 with page size 2
    Then I should receive 2 users
    And the total count should be 5
    And the total pages should be 3

  Scenario: Validate user name is required
    When I try to create a user with name "" and email "test@example.com"
    Then I should receive a validation error
    And the error message should contain "name cannot be empty"

  Scenario: Validate email format
    When I try to create a user with name "Test User" and email "invalid-email"
    Then I should receive a validation error
    And the error message should contain "email"
