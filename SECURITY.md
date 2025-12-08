# Security Policy

## Supported Versions

We release patches for security vulnerabilities. The following versions are currently being supported with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 1.x.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

We take the security of Product-FARM seriously. If you believe you have found a security vulnerability, please report it to us as described below.

### How to Report

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via email to:

**security@example.com** (replace with your security contact email)

Please include the following information in your report:

1. **Type of vulnerability** (e.g., SQL injection, XSS, authentication bypass, etc.)
2. **Location** of the vulnerability (file path, URL, API endpoint, etc.)
3. **Step-by-step instructions** to reproduce the issue
4. **Proof-of-concept** or exploit code (if possible)
5. **Impact** of the vulnerability (what an attacker could achieve)
6. **Suggested fix** (if you have one)

### What to Expect

- **Acknowledgment**: We will acknowledge receipt of your vulnerability report within 48 hours.
- **Communication**: We will keep you informed of the progress towards a fix.
- **Resolution**: We aim to resolve critical vulnerabilities within 7-14 days.
- **Disclosure**: We will coordinate with you on the disclosure timeline.

### Safe Harbor

We consider security research and vulnerability disclosure activities conducted consistent with this policy to be:

- Authorized in accordance with any applicable anti-hacking laws
- Exempt from restrictions in our Terms of Service that would interfere with conducting security research

We will not pursue civil action or initiate a complaint to law enforcement for accidental, good-faith violations of this policy.

## Security Best Practices

When deploying Product-FARM, we recommend the following security measures:

### Backend Security

1. **API Authentication**: Implement proper authentication (JWT, OAuth2) for all API endpoints
2. **Rate Limiting**: Enable rate limiting to prevent abuse
3. **Input Validation**: All inputs are validated, but additional validation at the application layer is recommended
4. **HTTPS**: Always use HTTPS in production
5. **Database Security**: Secure your DGraph instance with authentication and network policies

### Frontend Security

1. **CORS Configuration**: Configure CORS to allow only trusted origins
2. **Content Security Policy**: Implement CSP headers
3. **XSS Prevention**: The React frontend uses proper escaping, but be careful with `dangerouslySetInnerHTML`

### Infrastructure Security

1. **Network Isolation**: Run services in isolated network segments
2. **Secrets Management**: Use environment variables or secret managers for sensitive configuration
3. **Logging & Monitoring**: Enable audit logging and monitor for suspicious activity
4. **Regular Updates**: Keep all dependencies up to date

## Known Security Considerations

### JSON Logic Expressions

JSON Logic expressions are evaluated in a sandboxed environment. However:

- Avoid storing sensitive data in rule expressions
- Review rules before promoting to production
- Consider implementing rule review workflows

### Rule Evaluation

- Rule evaluation is CPU-bound; complex rules could be used for DoS attacks
- Implement timeouts for rule evaluation in production
- Consider rate limiting the evaluation API

### Data Storage

- DGraph does not encrypt data at rest by default
- Consider enabling encryption for sensitive deployments
- Implement proper backup and recovery procedures

## Security Updates

Security updates will be announced through:

1. GitHub Security Advisories
2. Release notes
3. Direct notification to known affected parties

## Acknowledgments

We appreciate the security research community's efforts in helping keep Product-FARM secure. Contributors who report valid security issues will be acknowledged (with their permission) in our release notes.

---

Thank you for helping keep Product-FARM and its users safe!
