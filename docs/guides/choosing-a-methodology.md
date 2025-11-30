# Choosing a Methodology

MUCM supports four different methodologies, each optimized for different team perspectives and workflows. This guide helps you choose the right one for your project.

## Quick Decision Guide

| Your Team Focus | Choose This | Why |
|----------------|-------------|-----|
| **Engineering & Implementation** | Developer | Technical details, API design, system architecture |
| **Quality Assurance & Testing** | Tester | Test coverage, quality metrics, test scenarios |
| **Business Analysis & Requirements** | Business | Stakeholder value, ROI, business requirements |
| **Agile & User Stories** | Feature | User-centric stories, acceptance criteria, sprint planning |

## Methodology Comparison

### Developer Methodology

**Best For:** Engineering teams, technical implementation, system design

**Focus:**
- API endpoints and service interfaces
- Database schema and data models
- Performance requirements
- Technical constraints and dependencies
- Implementation notes

**Custom Fields:**
- API Endpoint
- Database Schema
- Performance Requirements
- Technical Dependencies
- Implementation Notes

**When to Use:**
- Backend/API development
- System architecture documentation
- Technical design reviews
- Developer onboarding

**Example Use Case:**
```
Title: User Authentication API
API Endpoint: POST /api/v1/auth/login
Database Schema: users table (id, email_hash, password_hash, created_at)
Performance: < 200ms response time
```

---

### Tester Methodology

**Best For:** QA teams, test-driven development, quality assurance

**Focus:**
- Test scenarios and test cases
- Test coverage metrics
- Edge cases and error conditions
- Quality assurance criteria
- Defect tracking

**Custom Fields:**
- Test Scenarios
- Test Coverage
- Edge Cases
- Quality Metrics
- Expected Test Results

**When to Use:**
- QA-driven projects
- Test plan documentation
- Quality assurance processes
- Regression testing planning

**Example Use Case:**
```
Title: User Login Validation
Test Scenarios: Valid credentials, Invalid password, Locked account, Expired session
Test Coverage: 95% code coverage target
Edge Cases: SQL injection attempts, XSS in username field
```

---

### Business Methodology

**Best For:** Product managers, business analysts, stakeholder communication

**Focus:**
- Business value and ROI
- Stakeholder requirements
- Success metrics
- Cost-benefit analysis
- Business constraints

**Custom Fields:**
- Business Value
- ROI Estimate
- Stakeholders
- Success Metrics
- Budget Constraints

**When to Use:**
- Business requirement gathering
- Stakeholder presentations
- Product roadmap planning
- Investment justification

**Example Use Case:**
```
Title: Premium Subscription Feature
Business Value: Increase MRR by 25%
ROI Estimate: 300% in first year
Stakeholders: CEO, CFO, Marketing Director
Success Metrics: 15% conversion rate, < 5% churn
```

---

### Feature Methodology

**Best For:** Agile teams, user story workflows, feature development

**Focus:**
- User stories and personas
- Acceptance criteria
- Feature scope
- Sprint planning
- User value delivery

**Custom Fields:**
- User Story Format
- Acceptance Criteria
- Story Points
- Epic Link
- Sprint Assignment

**When to Use:**
- Scrum/Agile workflows
- Sprint planning
- User-centric development
- Feature team coordination

**Example Use Case:**
```
Title: Password Reset Flow
User Story: As a user who forgot my password, I want to reset it via email
Acceptance Criteria:
  - User receives reset link within 1 minute
  - Link expires after 24 hours
  - Old password cannot be reused
Story Points: 5
```

## Template Levels

Each methodology supports two detail levels:

### Normal Level
- **Use For:** Standard documentation, everyday use cases
- **Contains:** Essential fields and common scenarios
- **Recommended For:** Most use cases (80% of your documentation)

### Advanced Level
- **Use For:** Complex features, comprehensive documentation
- **Contains:** All fields including extended metadata
- **Recommended For:** Critical features, regulatory compliance, complex integrations

## Multi-View Support

You can create use cases with multiple methodology perspectives simultaneously:

```bash
# Create with both developer and tester views
mucm create "Payment Processing" --category payment \
  --views developer:normal,tester:advanced

# Create with business and feature views
mucm create "User Registration" --category auth \
  --views business:advanced,feature:normal
```

**When to use multi-view:**
- Cross-functional teams need different perspectives
- Technical and business documentation in one place
- QA and development working closely together
- Transitioning between methodologies

## Switching Methodologies

You can regenerate any use case with a different methodology:

```bash
# Regenerate with different methodology
mucm regenerate UC-SEC-001 --methodology tester

# Regenerate all use cases
mucm regenerate --all
```

**Note:** Regeneration updates the markdown documentation but preserves your TOML data.

## Mixing Methodologies

Different use case categories can use different methodologies:

```toml
# .config/.mucm/mucm.toml
[project]
default_methodology = "developer"

# Then use specific methodologies per use case
# Security use cases → tester methodology
# API use cases → developer methodology  
# Feature use cases → feature methodology
# Business use cases → business methodology
```

## Decision Flowchart

```
Start
  │
  ├─ Is your team primarily writing code?
  │  └─ YES → Developer Methodology
  │
  ├─ Is your team primarily testing?
  │  └─ YES → Tester Methodology
  │
  ├─ Are you documenting for stakeholders?
  │  └─ YES → Business Methodology
  │
  └─ Are you working in agile sprints?
     └─ YES → Feature Methodology
```

## Common Combinations

### Backend API Project
- Primary: **Developer** (technical implementation)
- Secondary: **Tester** (API test cases)

### SaaS Product
- Primary: **Business** (stakeholder requirements)
- Secondary: **Feature** (user stories)

### Mobile App
- Primary: **Feature** (user-centric development)
- Secondary: **Developer** (technical architecture)

### Enterprise Software
- Primary: **Business** (compliance and requirements)
- Secondary: **Developer** (technical implementation)
- Secondary: **Tester** (quality assurance)

## Tips

1. **Start Simple:** Begin with one methodology and add views later if needed
2. **Match Your Workflow:** Choose the methodology that matches your team's daily workflow
3. **Be Consistent:** Use the same methodology for related use cases
4. **Experiment:** Try different methodologies with `mucm regenerate` to see what works
5. **Documentation First:** Choose based on your primary audience (developers, testers, stakeholders, users)

## Getting More Information

```bash
# List all methodologies
mucm methodologies

# Get detailed information about a methodology
mucm methodology-info developer
mucm methodology-info tester
mucm methodology-info business
mucm methodology-info feature
```

## Need Help?

- **Still unsure?** Use `mucm -i` for interactive mode with guided selection
- **Want to try them?** Create test use cases with each methodology
- **Have questions?** Check out the [Configuration Guide](configuration.md) for advanced setup
