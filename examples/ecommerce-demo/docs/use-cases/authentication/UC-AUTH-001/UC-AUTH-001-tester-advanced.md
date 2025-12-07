# Test Specification: User Registration

**Use Case ID:** UC-AUTH-001  
**Test Status:**   
**Test Priority:** High  
**Test Plan Date:** 03/12/2025

---

**Test Type:** [e2e, integration, security, accessibility]  
**Priority:** P0-Critical  
**Test Environments:**
- Staging environment with email service integration
- Database with clean state before each test run
- Mock OAuth providers for social registration tests
- Various browsers and devices for front-end validation tests

### Coverage Areas
- Input validation (client-side and server-side)
- Database record creation and integrity
- Email delivery and content verification
- Session creation and management
- OAuth registration flows
- Security measures (CSRF, rate limiting, password hashing)
- Accessibility (WCAG 2.1 AA compliance)

## Preconditions
- User must have a valid email address
- User does not already have an account

## Postconditions
- User account created in database
- Verification email sent to user&#x27;s email address
- User logged in with unverified status
- Session created and stored in cache
- User can access basic platform features (but not checkout until verified)

## UC-AUTH-001-S01 - Successful User Registration

**Primary Actor:** [Guest User](../../../actors/guest-guest-user.md)

**Supporting Actors:** [Database](../../../actors/database.md), [E-commerce Platform](../../../actors/e-commerce-platform.md), [Cache](../../../actors/cache.md), [Email Service](../../../actors/email-service.md)

```mermaid
sequenceDiagram
participant Guest User
Guest User->>E-commerce Platform: 1. navigates to registration page
E-commerce Platform->>Guest User: 2. displays registration form
Guest User->>E-commerce Platform: 3. ubmits registration form (email, password, name, ...)
E-commerce Platform->>E-commerce Platform: 4. validates email format and password requirements
E-commerce Platform->>Database: 5. checks email is not already registered
Database->>E-commerce Platform: 6. returns 'email available'
E-commerce Platform->>Database: 7. creates new user record with hashed password
Database->>E-commerce Platform: 8. confirms user created
E-commerce Platform->>Cache: 9. creates session for new user
Cache->>E-commerce Platform: 10. returns session token
E-commerce Platform->>Email Service: 11. sends verification email request
Email Service->>E-commerce Platform: 12. confirms email queued
E-commerce Platform->>Guest User: 13. displays 'Account created successfully' and redirects to dashboard
```

### Steps
1. **Guest User** → **E-commerce Platform**: navigates to registration page
2. **E-commerce Platform** → **Guest User**: displays registration form
3. **Guest User** → **E-commerce Platform**: ubmits registration form (email, password, name, ...)
4. **E-commerce Platform**: validates email format and password requirements
5. **E-commerce Platform** → **Database**: checks email is not already registered
6. **Database** → **E-commerce Platform**: returns &quot;email available&quot;
7. **E-commerce Platform** → **Database**: creates new user record with hashed password
8. **Database** → **E-commerce Platform**: confirms user created
9. **E-commerce Platform** → **Cache**: creates session for new user
10. **Cache** → **E-commerce Platform**: returns session token
11. **E-commerce Platform** → **Email Service**: sends verification email request
12. **Email Service** → **E-commerce Platform**: confirms email queued
13. **E-commerce Platform** → **Guest User**: displays &quot;Account created successfully&quot; and redirects to dashboard

## UC-AUTH-001-S02 - Registration with Social Login (Extension)

**Primary Actor:** [Guest User](../../../actors/guest-guest-user.md)

**Supporting Actors:** [E-commerce Platform](../../../actors/e-commerce-platform.md), [Database](../../../actors/database.md), [Cache](../../../actors/cache.md)

**Preconditions:**
- User must have a valid social account

```mermaid
sequenceDiagram
participant Guest User
Guest User->>E-commerce Platform: 1. selects 'Sign up with Google' (or other social profile)
E-commerce Platform->>Guest User: 2. redirects to Google OAuth
Guest User->>E-commerce Platform: 3. authenticates with Google
E-commerce Platform->>Database: 4. checks email is not already registered
Database->>E-commerce Platform: 5. returns 'email available'
E-commerce Platform->>Database: 6. creates new user record marked as OAuth-verified
Database->>E-commerce Platform: 7. confirms user created
E-commerce Platform->>Cache: 8. creates session for new user
Cache->>E-commerce Platform: 9. returns session token
```

### Steps
1. **Guest User** → **E-commerce Platform**: selects &quot;Sign up with Google&quot; (or other social profile)
2. **E-commerce Platform** → **Guest User**: redirects to Google OAuth
3. **Guest User** → **E-commerce Platform**: authenticates with Google
4. **E-commerce Platform** → **Database**: checks email is not already registered
5. **Database** → **E-commerce Platform**: returns &quot;email available&quot;
6. **E-commerce Platform** → **Database**: creates new user record marked as OAuth-verified
7. **Database** → **E-commerce Platform**: confirms user created
8. **E-commerce Platform** → **Cache**: creates session for new user
9. **Cache** → **E-commerce Platform**: returns session token

*Extends scenario UC-AUTH-001-S01 at step 3*, returns at step 13

## UC-AUTH-001-S03 - Email Already Registered (Extension)

**Primary Actor:** [Guest User](../../../actors/guest-guest-user.md)

**Supporting Actors:** [Database](../../../actors/database.md), [E-commerce Platform](../../../actors/e-commerce-platform.md)

**Preconditions:**
- Used email is already registered

```mermaid
sequenceDiagram
participant Guest User
Database->>E-commerce Platform: 1. returns 'email already exists'
E-commerce Platform->>Guest User: 2. displays error: 'An account with this email already exists. Please login or reset your password.'
E-commerce Platform->>Guest User: 3. offers links to login page and password reset
```

### Steps
1. **Database** → **E-commerce Platform**: returns &quot;email already exists&quot;
2. **E-commerce Platform** → **Guest User**: displays error: &quot;An account with this email already exists. Please login or reset your password.&quot;
3. **E-commerce Platform** → **Guest User**: offers links to login page and password reset

*Extends scenario UC-AUTH-001-S01 at step 6*

## Test Data Requirements
10 valid email addresses for testing (test+1@example.com through test+10@example.com), 5 known registered emails for duplicate testing, mock SendGrid API with configurable responses (success, failure, timeout), database with clean state before each test run.

## Test Environments
- Staging environment with email service integration
- Database with clean state before each test run
- Mock OAuth providers for social registration tests
- Various browsers and devices for front-end validation tests

## Coverage Areas
- Input validation (client-side and server-side)
- Database record creation and integrity
- Email delivery and content verification
- Session creation and management
- OAuth registration flows
- Security measures (CSRF, rate limiting, password hashing)
- Accessibility (WCAG 2.1 AA compliance)

---

**Last Updated:** 07/12/2025

---

**Navigation:** [← Back to Authentication](../README.md) | [← Back to All Use Cases](../../README.md)