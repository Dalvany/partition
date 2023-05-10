# language: en

Feature: Test API

  Background:
    Given partition is running

  @serial
  Scenario: Get server information
    When accessing "/api/v1/"
    Then version match Cargo.toml
