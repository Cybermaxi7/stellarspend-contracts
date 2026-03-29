# StellarSpend End-to-End Testing Documentation

## Overview

This document describes the complete E2E testing strategy for the StellarSpend smart contract.

## Coverage Goal

Target: >90% code coverage

## Test Scenarios Covered

### 1. Wallet Creation
- Create user wallet
- Verify initial balance

### 2. Transfers
- Successful transfer
- Transfer with insufficient balance

### 3. Budget Management
- Create budget
- Spend within budget
- Overspend scenario

### 4. Savings Management
- Create savings bucket
- Withdraw from savings
- Over-withdraw failure

### 5. Edge Cases
- Zero transfer attempt
- Non-existent wallet
- Non-existent budget
- Non-existent savings bucket

## Failure Scenarios Tested

- Panic on insufficient funds
- Panic on budget exceed
- Panic on savings insufficient

## Tools Used

- Soroban test environment
- Rust unit testing framework
- Cargo Tarpaulin for coverage

## How to Run Tests
