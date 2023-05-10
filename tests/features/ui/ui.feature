# language: en

Feature: Accessing different UI path

  Background:
    Given partition is running

  @serial
  Scenario Outline: Accessing UI path
    When accessing "<path>"
    Then the HTTP status is <status>
    And header <header> is <location>

    Examples:
      | path                  | status | header     | location         |
      | /ui/../index.html     | 404    | ""         | ""               |
      | /ui/%2E%2E/index.html | 404    | ""         | ""               |
      | /ui/index.html        | 200    | ""         | ""               |
      | /                     | 308    | "Location" | "/ui/index.html" |
      | /ui                   | 308    | "Location" | "/ui/index.html" |
      | /ui/                  | 308    | "Location" | "/ui/index.html" |
