# Allgemein

- Hinterfrage mich kritisch wenn nötig bevor du etwas tust
- Benutze in der jeweiligen Sprache best practices und aktuelle Versionen

---

# Projektüberblick

Dieses Repository enthält eine Fullstack-Anwendung bestehende aus:

- **Backend**: Rust (API, Business-Logik, DB)
- **Frontend**: React (UI, API-Integration)
- **Infrastructure**: Docker-basierte Entwicklungs- und Deployment-Umgebung

Ziel ist eine klar getrennte, skalierbare Architektur mit reproduzierbaren Builds und konsistenter Dev-Experience.

## Inhalt des Projekts

- Name Whatsup
- Funktion: Eine Anwendung mit der Benutzer eine Gruppe bilden können
    - Benutzer haben Spiele die sie spielen wollen
    - Gruppen sehen alle Spiele und wie viele Leute das gleiche Spiel spielen wollen
    - Man sieht den Status "Lust zu spielen"
    - Man kann auch sagen "An dem Tag"

---

# 2. Architekturprinzipien

- Strikte Trennung von Frontend und Backend
- API-first Design
- Stateless Backend Services
- Containerisiertes Deployment und Containergestützte Entwicklung
- Keine Business-Logik im Frontend

---

# 3. Backend (Rust)

### Technologie

- Rust (Stable)
- Webframework
- ...

### Prinzipien

- Domänen modular strukturieren
- Kein unwrap()
- DTOs getrennt von Domain Models
- Konsistente API Responses

---

# 4. Frontend (React)

## Technologie

- React + TypeScript
- React Query oder Zustand
- Vite

## Prinzipien

- UI / Logik Trennung
- API Calls nur in services
- Wiederverwendbarkeit
- Keine Business Logik
- Atomic Design

---

# 5. API Kommunikation

- Primär REST (JSON)
- Sekundär später angedacht: Websockets oder vergleichbares
- Versionierung der API über /api/v1/
- Zentrale API Client Schicht
- Einheitliche Error Responses

---

# 6. Infrastructure

## Ziele

- Reproduzierbarkeit
- Einfache lokale Entwicklung
- Klare Services-Trennung

---

# 7. Sicherheit

- Input Validation im Backend
- CORS explizit setzen

---

# 8. Coding-Standards

## Rust

- rustfmt verpflichtend
- clippy ohne Warnungen
- sauberes Error-Handling

## React

- ESLint + Prettier
- TypeScript bevorzugt
- Keine Side Effects in UI Komponenten

---

# 9. Erweiterbarkeit

- Modular erweiterbares Backend / Frontend
- Ggf. teilweise Umstellung auf WebSockets oder ähnliche Technologie wo es Sinn ergeben würde

---

# 10. Datenbank

## Technologie

- SurrealDB und REST

---

# 📌 Commit Convention (Backend + Frontend + Infrastructure)

Dieses Projekt folgt einer klaren Commit-Struktur, um Änderungen nachvollziehbar, automatisierbar und KI-lesbar zu halten.

---

## 🧱 Grundformat

<type>(<scope>): <kurze Beschreibung>

Optional:

<type>(<scope>): <kurze Beschreibung>

- Detail 1
- Detail 2

---

## 🏷️ Types (Commit-Arten)

### ✨ feat
Neue Funktionalität

feat(auth): add JWT login flow 🔐
feat(api): add user profile endpoint 👤

---

### 🐛 fix
Bugfixes

fix(frontend): resolve login redirect issue 🧭
fix(backend): handle null pointer in user service 🧯

---

### ♻️ refactor
Code-Verbesserung ohne funktionale Änderung

refactor(backend): simplify auth middleware ♻️
refactor(frontend): restructure API client layer 🧹

---

### 🎨 style
Nur UI / Formatierung / Styling

style(frontend): improve button spacing 🎨
style(ui): adjust dark mode colors 🌙

---

### ⚡ perf
Performance-Optimierungen

perf(api): reduce DB query count ⚡
perf(frontend): memoize heavy components 🚀

---

### 🧪 test
Tests hinzufügen oder ändern

test(auth): add login unit tests 🧪
test(api): improve integration coverage ✅

---

### 📚 docs
Dokumentation

docs(readme): update setup instructions 📚
docs(api): add endpoint documentation 📝

---

### 🔧 chore
Build, tooling, dependencies, infra

chore(deps): update Rust dependencies 🔧
chore(docker): improve compose setup 🐳
chore(ci): add GitHub Actions pipeline ⚙️

---

### 🚀 infra
Infrastructure / Docker / Deployment

infra(docker): add multi-stage backend build 🐳
infra(k8s): add staging deployment config ☸️

---

### 🔐 security
Security fixes

security(auth): fix JWT expiration handling 🔐
security(api): sanitize user input 🛡️

---

## 📦 Scope Beispiele

backend
frontend
api
auth
db
ui
docker
ci
infra

---

## 📌 Beispiele für gute Commits

### Backend
feat(backend): add user registration endpoint 👤
fix(backend): prevent SQL injection in search query 🛡️
refactor(backend): split service layer into modules ♻️

---

### Frontend
feat(frontend): add dashboard overview page 📊
fix(frontend): resolve state sync issue in profile page 🔄
style(frontend): improve form validation UI 🎨

---

### Infrastructure
infra(docker): optimize frontend build stage 🐳
chore(ci): add lint and test pipeline ⚙️

---

## 🚫 Nicht erlaubt

- "fix bug"
- "update code"
- "stuff"
- unklare oder nicht beschreibende Messages

---

## 📏 Regeln

- maximal 1 funktionale Änderung pro Commit
- klare technische Beschreibung
- keine subjektiven Aussagen
- Englisch als Standard
- Emojis optional, aber konsistent

---

## 🧠 Ziel

- nachvollziehbare Historie
- CI/CD-freundlich
- gut lesbar für Menschen & Tools
- skalierbar für Teams und KI-Agenten