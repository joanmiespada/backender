Feature: Role Management
  As a system administrator
  I want to manage roles and assign them to users
  So that I can implement access control in the system

  Background:
    Given a clean user database

  Scenario: Create a new role
    When I create a role with name "admin"
    Then the role should be created successfully
    And the role should have name "admin"

  Scenario: Prevent duplicate role names
    Given a role exists with name "editor"
    When I try to create a role with name "editor"
    Then I should receive a role name already exists error

  Scenario: Assign a role to a user
    Given a user exists with name "John Doe" and email "john@example.com"
    And a role exists with name "admin"
    When I assign the role to the user
    Then the assignment should be successful
    And the user should have the role "admin"

  Scenario: Prevent duplicate role assignment
    Given a user exists with name "Jane Doe" and email "jane@example.com"
    And a role exists with name "editor"
    And the user has the role "editor"
    When I try to assign the role "editor" to the user again
    Then I should receive a user already has role error

  Scenario: Unassign a role from a user
    Given a user exists with name "Bob Wilson" and email "bob@example.com"
    And a role exists with name "viewer"
    And the user has the role "viewer"
    When I unassign the role from the user
    Then the unassignment should be successful
    And the user should not have the role "viewer"

  Scenario: User can have multiple roles
    Given a user exists with name "Alice Smith" and email "alice@example.com"
    And the following roles exist:
      | name     |
      | admin    |
      | editor   |
      | viewer   |
    When I assign all roles to the user
    Then the user should have 3 roles

  Scenario: List roles with pagination
    Given the following roles exist:
      | name       |
      | admin      |
      | editor     |
      | viewer     |
      | moderator  |
    When I request roles page 1 with page size 2
    Then I should receive 2 roles
    And the total roles count should be 4

  Scenario: Get users by role
    Given a role exists with name "developer"
    And the following users have the role "developer":
      | name          | email               |
      | Dev One       | dev1@example.com    |
      | Dev Two       | dev2@example.com    |
    And a user exists with name "Non Dev" and email "nondev@example.com"
    When I get users with role "developer"
    Then I should receive 2 users with that role

  Scenario: Delete a role
    Given a role exists with name "temporary"
    When I delete the role
    Then the role deletion should be successful

  Scenario: Validate role name is required
    When I try to create a role with name ""
    Then I should receive a validation error
    And the error message should contain "role name cannot be empty"
