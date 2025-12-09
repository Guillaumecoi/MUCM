# Use Case: UC-AUTH-002: User Login

**Status:**  | **Priority:** Critical

**Category:** Authentication

## Description

Allow registered users to authenticate and access their accounts by providing email and password. The system validates credentials, creates a session, and redirects users to their personalized dashboard or previous page.

## Preconditions

- User must have a registered account||UC:UC-AUTH-001:depend
- User must have valid credentials
- User is authenticated and logged in

## Postconditions

- User is authenticated and logged in
- Session created and stored in cache
- Session cookie set in browser
- User redirected to dashboard or previous page
- Last login timestamp updated in database



## Available Views

This use case has multiple methodology views available:

- **[developer (normal)](./UC-AUTH-002-developer-normal.md)** - Detailed technical specifications for developers
- **[tester (advanced)](./UC-AUTH-002-tester-advanced.md)** - Comprehensive testing scenarios and acceptance criteria

## UC-AUTH-002-S01 - Successful Login with Email and Password

**Primary Actor:** [Maria Garcia](../../../actors/customer-support-agent-maria-garcia.md)

**Supporting Actors:** [E-commerce Platform](../../../actors/e-commerce-platform.md), [Database](../../../actors/database.md), [Cache](../../../actors/cache.md)

```mermaid
sequenceDiagram
participant Maria Garcia
participant E-commerce Platform
participant Database
participant Cache
Maria Garcia->>E-commerce Platform: 1. navigates to login page
E-commerce Platform->>Maria Garcia: 2. displays login form
Maria Garcia->>E-commerce Platform: 3. submits login credentials (email, password)
E-commerce Platform->>E-commerce Platform: 4. validates input
E-commerce Platform->>Database: 5. retrieves user record by email
Database->>E-commerce Platform: 6. returns user record
E-commerce Platform->>E-commerce Platform: 7. verifies password matches stored hash 
E-commerce Platform->>E-commerce Platform: 8. checks account is not locked or suspended
E-commerce Platform->>Database: 9. updates last_login timestamp
Database->>E-commerce Platform: 10. confirms update
E-commerce Platform->>Cache: 11. creates new session with 30-day expiry
Cache->>E-commerce Platform: 12. returns session token
E-commerce Platform->>Maria Garcia: 13. sets session cookie, displays 'Login successful', and redirects to dashboard
```

### Steps
1. **Maria Garcia** → **E-commerce Platform**: navigates to login page
2. **E-commerce Platform** → **Maria Garcia**: displays login form
3. **Maria Garcia** → **E-commerce Platform**: submits login credentials (email, password)
4. **E-commerce Platform**: validates input
5. **E-commerce Platform** → **Database**: retrieves user record by email
6. **Database** → **E-commerce Platform**: returns user record
7. **E-commerce Platform**: verifies password matches stored hash 
8. **E-commerce Platform**: checks account is not locked or suspended
9. **E-commerce Platform** → **Database**: updates last_login timestamp
10. **Database** → **E-commerce Platform**: confirms update
11. **E-commerce Platform** → **Cache**: creates new session with 30-day expiry
12. **Cache** → **E-commerce Platform**: returns session token
13. **E-commerce Platform** → **Maria Garcia**: sets session cookie, displays &quot;Login successful&quot;, and redirects to dashboard


---

**Navigation:** [← Back to Authentication](../README.md) | [← Back to All Use Cases](../../README.md)
