# Scenario Management Guide

This guide covers working with scenarios in the Markdown Use Case Manager (mucm). Scenarios describe specific paths of interaction through a use case, including the main success path, alternative flows, and exception handling.

## Table of Contents

- [Overview](#overview)
- [Scenario Types](#scenario-types)
- [Basic Operations](#basic-operations)
- [Working with Steps](#working-with-steps)
- [Persona Assignment](#persona-assignment)
- [Scenario References](#scenario-references)
- [Common Workflows](#common-workflows)
- [Best Practices](#best-practices)

## Overview

Scenarios are organized within use cases and follow a nested command structure:

```bash
mucm usecase scenario <command> [options]
```

Each scenario:
- Has a unique ID (e.g., `UC-SEC-001-S01`)
- Belongs to a use case
- Has a type (main, alternative, or exception)
- Contains an ordered list of steps
- Can be assigned to a persona
- Can reference other scenarios or use cases

## Scenario Types

### Main Scenarios
The primary success path - the happy path where everything goes as expected.

```bash
mucm usecase scenario add UC-AUTH-001 "Successful Login" \
  --scenario-type main \
  --description "User successfully authenticates with valid credentials"
```

### Alternative Scenarios
Variations of the main flow that still achieve the goal but through a different path.

```bash
mucm usecase scenario add UC-AUTH-001 "Login with 2FA" \
  --scenario-type alternative \
  --description "User logs in with two-factor authentication enabled"
```

### Exception Scenarios
Error handling and failure cases that prevent the goal from being achieved.

```bash
mucm usecase scenario add UC-AUTH-001 "Invalid Password" \
  --scenario-type exception \
  --description "User enters incorrect password"
```

## Basic Operations

### Creating a Scenario

```bash
# Basic scenario
mucm usecase scenario add UC-AUTH-001 "Password Reset Flow" --scenario-type alternative

# With description
mucm usecase scenario add UC-AUTH-001 "Account Locked" \
  --scenario-type exception \
  --description "Account locked after multiple failed attempts"

# With persona assigned
mucm usecase scenario add UC-AUTH-001 "Admin Override Login" \
  --scenario-type alternative \
  --persona admin-user
```

### Listing Scenarios

```bash
# List all scenarios for a use case
mucm usecase scenario list UC-AUTH-001

# Example output:
# Scenarios for UC-AUTH-001:
#   UC-AUTH-001-S01 [main] Successful Login
#   UC-AUTH-001-S02 [alternative] Login with 2FA  
#   UC-AUTH-001-S03 [exception] Invalid Password
```

### Editing a Scenario

```bash
# Update title
mucm usecase scenario edit UC-AUTH-001 UC-AUTH-001-S01 \
  --title "Standard Login Flow"

# Update status
mucm usecase scenario edit UC-AUTH-001 UC-AUTH-001-S01 \
  --status in-progress

# Update multiple fields
mucm usecase scenario edit UC-AUTH-001 UC-AUTH-001-S02 \
  --title "Two-Factor Authentication Login" \
  --description "Enhanced security login with SMS or app-based 2FA" \
  --status completed
```

### Deleting a Scenario

```bash
mucm usecase scenario delete UC-AUTH-001 UC-AUTH-001-S05
```

## Working with Steps

Scenarios consist of ordered steps that describe the interaction flow.

### Adding Steps

```bash
# Add a step (appends to end)
mucm usecase scenario step add UC-AUTH-001 UC-AUTH-001-S01 \
  "User navigates to login page"

# Add a step with specific order
mucm usecase scenario step add UC-AUTH-001 UC-AUTH-001-S01 \
  "User enters username and password" \
  --order 2

# Add a step with actor
mucm usecase scenario step add UC-AUTH-001 UC-AUTH-001-S01 \
  "System validates credentials against database" \
  --actor system
```

### Editing Steps

```bash
# Update step description
mucm usecase scenario step edit UC-AUTH-001 UC-AUTH-001-S01 1 \
  --description "User opens the application login page"

# Change step actor
mucm usecase scenario step edit UC-AUTH-001 UC-AUTH-001-S01 3 \
  --actor authentication-service
```

### Removing Steps

```bash
# Remove step by order number
mucm usecase scenario step remove UC-AUTH-001 UC-AUTH-001-S01 4
```

### Complete Step Example

```bash
# Building a complete scenario with steps
mucm usecase scenario add UC-PAY-001 "Credit Card Payment" --scenario-type main

# Add steps in order
mucm usecase scenario step add UC-PAY-001 UC-PAY-001-S01 \
  "User selects items and proceeds to checkout"

mucm usecase scenario step add UC-PAY-001 UC-PAY-001-S01 \
  "User enters credit card information"

mucm usecase scenario step add UC-PAY-001 UC-PAY-001-S01 \
  "System validates card details" \
  --actor payment-gateway

mucm usecase scenario step add UC-PAY-001 UC-PAY-001-S01 \
  "System processes payment" \
  --actor payment-gateway

mucm usecase scenario step add UC-PAY-001 UC-PAY-001-S01 \
  "System confirms successful payment"
```

## Persona Assignment

Assign personas to scenarios to indicate which user type the scenario is designed for.

### Assigning a Persona

```bash
# Assign persona when creating scenario
mucm usecase scenario add UC-ADMIN-001 "Bulk User Import" \
  --scenario-type main \
  --persona admin-user

# Assign persona to existing scenario
mucm usecase scenario assign-persona UC-AUTH-001 UC-AUTH-001-S01 standard-user
```

### Unassigning a Persona

```bash
mucm usecase scenario unassign-persona UC-AUTH-001 UC-AUTH-001-S01
```

### Finding Scenarios by Persona

```bash
# List all use cases using a specific persona
mucm persona use-cases admin-user
```

## Scenario References

Scenarios can reference other scenarios or use cases to model dependencies and relationships.

### Reference Types

- **scenario**: Reference to another scenario
- **usecase**: Reference to a use case

### Relationship Types

- `includes`: The scenario includes functionality from the target
- `extends`: The scenario extends the target with additional functionality
- `depends-on`: The scenario depends on the target being completed first
- `alternative-to`: The scenario provides an alternative path to the target

### Adding References

```bash
# Scenario depends on another scenario
mucm usecase scenario reference add UC-API-001 UC-API-001-S01 UC-AUTH-001-S01 \
  --ref-type scenario \
  --relationship depends-on \
  --description "Must authenticate before API access"

# Scenario extends another scenario
mucm usecase scenario reference add UC-AUTH-001 UC-AUTH-001-S02 UC-AUTH-001-S01 \
  --ref-type scenario \
  --relationship extends \
  --description "Adds two-factor authentication to standard login"

# Scenario depends on a use case
mucm usecase scenario reference add UC-PAY-001 UC-PAY-001-S01 UC-AUTH-001 \
  --ref-type usecase \
  --relationship depends-on \
  --description "User must be authenticated to make payments"
```

### Listing References

```bash
mucm usecase scenario reference list UC-API-001 UC-API-001-S01
```

### Removing References

```bash
mucm usecase scenario reference remove UC-API-001 UC-API-001-S01 UC-AUTH-001-S01 \
  --relationship depends-on
```

## Common Workflows

### Workflow 1: Creating a Complete Use Case with Scenarios

```bash
# 1. Create the use case
mucm create "User Authentication" --category security

# 2. Add main scenario
mucm usecase scenario add UC-SEC-001 "Successful Login" \
  --scenario-type main \
  --persona standard-user

# 3. Add steps to main scenario
mucm usecase scenario step add UC-SEC-001 UC-SEC-001-S01 "User opens login page"
mucm usecase scenario step add UC-SEC-001 UC-SEC-001-S01 "User enters valid credentials"
mucm usecase scenario step add UC-SEC-001 UC-SEC-001-S01 "System validates credentials" --actor system
mucm usecase scenario step add UC-SEC-001 UC-SEC-001-S01 "System creates session" --actor system
mucm usecase scenario step add UC-SEC-001 UC-SEC-001-S01 "User is redirected to dashboard"

# 4. Add alternative scenarios
mucm usecase scenario add UC-SEC-001 "Login with Remember Me" \
  --scenario-type alternative \
  --persona standard-user

mucm usecase scenario add UC-SEC-001 "Social Login" \
  --scenario-type alternative \
  --persona standard-user

# 5. Add exception scenarios
mucm usecase scenario add UC-SEC-001 "Invalid Credentials" \
  --scenario-type exception

mucm usecase scenario add UC-SEC-001 "Account Locked" \
  --scenario-type exception

# 6. View all scenarios
mucm usecase scenario list UC-SEC-001
```

### Workflow 2: Modeling Scenario Dependencies

```bash
# Create base authentication scenario
mucm usecase scenario add UC-AUTH-001 "Standard Authentication" \
  --scenario-type main

# Create enhanced authentication that extends it
mucm usecase scenario add UC-AUTH-001 "2FA Authentication" \
  --scenario-type alternative

# Add dependency reference
mucm usecase scenario reference add UC-AUTH-001 UC-AUTH-001-S02 UC-AUTH-001-S01 \
  --ref-type scenario \
  --relationship extends \
  --description "Extends standard auth with two-factor verification"

# Create API scenario that depends on auth
mucm create "API Access" --category api
mucm usecase scenario add UC-API-001 "Authenticated API Call" \
  --scenario-type main

mucm usecase scenario reference add UC-API-001 UC-API-001-S01 UC-AUTH-001 \
  --ref-type usecase \
  --relationship depends-on \
  --description "Requires user authentication"
```

### Workflow 3: Iterative Scenario Development

```bash
# Start with basic scenario
mucm usecase scenario add UC-FEAT-001 "New Feature Flow" \
  --scenario-type main \
  --status planned

# Add initial steps
mucm usecase scenario step add UC-FEAT-001 UC-FEAT-001-S01 "User initiates feature"
mucm usecase scenario step add UC-FEAT-001 UC-FEAT-001-S01 "System processes request" --actor system

# Update status as you develop
mucm usecase scenario edit UC-FEAT-001 UC-FEAT-001-S01 --status in-progress

# Add more steps as requirements clarify
mucm usecase scenario step add UC-FEAT-001 UC-FEAT-001-S01 "System validates input" --actor system
mucm usecase scenario step add UC-FEAT-001 UC-FEAT-001-S01 "System stores data" --actor system

# Refine step descriptions
mucm usecase scenario step edit UC-FEAT-001 UC-FEAT-001-S01 1 \
  --description "User clicks 'Start Feature' button in toolbar"

# Mark complete
mucm usecase scenario edit UC-FEAT-001 UC-FEAT-001-S01 --status completed
```

## Best Practices

### Naming Conventions

**Good scenario names:**
- "Successful User Registration"
- "Payment with Discount Code"
- "Export Data to CSV"
- "Handle Network Timeout"

**Avoid:**
- Vague names like "Main Flow" or "Alternative 1"
- Technical jargon unless necessary
- Overly long descriptions in the title

### Scenario Organization

1. **Start with the main scenario** - Define the happy path first
2. **Add key alternatives** - Cover the most common variations
3. **Include critical exceptions** - Focus on errors users will encounter
4. **Don't over-specify** - You don't need every possible variation

### Step Granularity

**Good step level:**
```bash
mucm usecase scenario step add UC-001 UC-001-S01 "User enters email address"
mucm usecase scenario step add UC-001 UC-001-S01 "User enters password"
mucm usecase scenario step add UC-001 UC-001-S01 "User clicks login button"
mucm usecase scenario step add UC-001 UC-001-S01 "System validates credentials" --actor system
```

**Too granular (avoid):**
```bash
mucm usecase scenario step add UC-001 UC-001-S01 "User moves mouse to email field"
mucm usecase scenario step add UC-001 UC-001-S01 "User clicks in email field"
mucm usecase scenario step add UC-001 UC-001-S01 "User types first character of email"
```

**Too high-level (avoid):**
```bash
mucm usecase scenario step add UC-001 UC-001-S01 "User logs in"
```

### Using Personas Effectively

- Assign personas to scenarios when user type matters for the flow
- Use persona assignment to find all scenarios for a specific user type
- Don't assign personas if the scenario applies to all users equally

### Reference Management

- Use `depends-on` for prerequisites (authentication, permissions)
- Use `extends` for enhanced versions of flows
- Use `includes` for shared sub-flows
- Add descriptions to explain why the relationship exists

### Status Tracking

Update scenario status as work progresses:

- `planned` - Defined but not implemented
- `in-progress` - Currently being developed
- `completed` - Implemented and tested
- `deprecated` - No longer supported

```bash
# Track progress through implementation
mucm usecase scenario edit UC-001 UC-001-S01 --status in-progress
# ... development work ...
mucm usecase scenario edit UC-001 UC-001-S01 --status completed
```

## Next Steps

- See [CLI Reference](cli-reference.md) for complete command syntax
- See [Configuration Guide](configuration.md) for project setup
- See [Testing Guide](testing.md) for test generation from scenarios
- Use Interactive Mode (`mucm -i`) for guided scenario management

## Related Commands

- `mucm create` - Create new use cases
- `mucm persona create` - Create personas for scenario assignment
- `mucm persona use-cases` - Find scenarios by persona
- `mucm -i` - Interactive mode for guided workflows
