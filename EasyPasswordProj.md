# EasyPassword — Requirements Specification

## 1. Background & Motivation

For personal security reasons, I want to ensure that **each website or application uses a unique password**.

Manually remembering or generating different passwords for every site is impractical.  
My current workaround is a simple rule-based scheme (e.g. fixed prefix + domain suffix), but it has several drawbacks:

- Low entropy
- Predictable structure
- Requires manual mental computation
- Error-prone when typing

This project aims to **automate deterministic password generation** while keeping the user experience lightweight, fast, and local-only.

---

## 2. Core Goal

Build a **local-only password assistant** that:

- Generates deterministic, high-entropy passwords
- Never stores plaintext passwords
- Never communicates with external services
- Works globally across applications
- Minimizes cognitive and interaction overhead

---

## 3. Target User Experience

### Ideal Interaction Flow

1. User types a trigger pattern such as:

;;github.com


2. The system recognizes the trigger in real time.

3. The trigger text is automatically replaced with a generated password.

4. The generated password:
- Is deterministic (same input → same output)
- Is site-specific
- Has configurable length and character set

5. The entire process happens **locally and instantly**, without explicit copy/paste.

---

## 4. Functional Requirements

### 4.1 Trigger Detection

- The system SHALL monitor user input globally.
- The system SHALL detect a configurable trigger prefix (default: `;;`).
- The trigger target SHALL be parsed as a site identifier (e.g. `github.com`).
- Trigger recognition SHALL occur only after an explicit termination signal:
- Space
- Enter
- Tab

---

### 4.2 Password Generation

- Passwords SHALL be generated from:
- A user-provided master key
- A site identifier
- The generation algorithm SHALL be deterministic.
- The algorithm SHALL be cryptographically strong.
- Password length SHALL be configurable.
- The character set SHOULD include:
- Uppercase letters
- Lowercase letters
- Digits
- Symbols

---

### 4.3 Replacement Behavior

- Upon successful trigger detection:
- The original trigger text SHALL be removed.
- The generated password SHALL be inserted at the same cursor location.
- Replacement SHALL appear atomic to the user.

---

### 4.4 Global Availability

- The system SHALL work across applications, including:
- Web browsers
- Native desktop applications
- Code editors and IDEs
- The system SHALL not require per-application plugins.

---

## 5. Non-Functional Requirements

### 5.1 Security

- The master key SHALL never be stored in plaintext.
- Generated passwords SHALL never be persisted to disk.
- All processing SHALL be local-only.
- The system SHALL avoid unnecessary data retention.

---

### 5.2 Performance

- Trigger detection and replacement SHALL feel instantaneous.
- The system SHALL have negligible CPU and memory overhead while idle.

---

### 5.3 Reliability

- The system SHALL not interfere with normal typing behavior.
- Failure to recognize a trigger SHALL not corrupt user input.
- The system SHALL degrade gracefully if replacement is not possible.

---

## 6. Platform Requirements

- Initial target platforms:
- Windows
- macOS
- Cross-platform architecture is preferred.
- Platform-specific implementations MAY be used internally.

---

## 7. Constraints & Assumptions

- The system is intended for **personal use**.
- No cloud synchronization is required.
- The user is responsible for remembering their master key.
- Losing the master key implies losing access to all derived passwords.

---

## 8. Out of Scope (Initial Version)

- Password sharing
- Multi-user support
- Cloud backup
- Browser-specific integrations
- Mobile platforms

---

## 9. Success Criteria

The project is considered successful when:

- The user can type `;;<site>` in any text field.
- The text is replaced with a strong, deterministic password.
- No manual copying, pasting, or mental computation is required.
- The system feels invisible during normal use.

---

## 10. Long-Term Vision (Optional)

- Site normalization (URL → canonical domain)
- Password rotation strategies
- Per-site configuration
- UI for managing preferences
- Optional integration with OS keychains