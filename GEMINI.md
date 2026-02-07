# Project: Leash AI

## Project Overview

Leash AI is a comprehensive, production-ready permission and access management system designed specifically for OpenClaw AI agents running on **macOS**. Its core purpose is to provide secure, auditable access to sensitive resources like API keys (secrets), system packages, and CLI commands. It features a human-in-the-loop approval workflow and maintains a full audit trail of all agent actions.

## Key Technologies

*   **Language:** Python 3.9+
*   **Web Framework:** FastAPI
*   **CLI Framework:** Typer
*   **Database ORM:** SQLAlchemy (with aiosqlite for SQLite backend)
*   **Cryptography:** `cryptography` library
*   **Configuration:** YAML for policies
*   **Platform-specific:** macOS Keychain for secure secret storage, Homebrew for package management.

## Architecture

The system is built with a modular design utilizing **Abstract Base Classes** to ensure extensibility. Key components include:

*   **Abstract Backend Layer:** For secrets, package managers, and CLI execution.
*   **Concrete Implementations:** macOS Keychain backend for secrets, Homebrew for packages.
*   **Permission Policy Engine:** Defines fine-grained, YAML-based policies with features like time-based access and auto-approval patterns.
*   **Client SDK:** A simple async API for OpenClaw instances to request permissions.
*   **Management CLI (`leash`):** For daemon control, policy management, audit log viewing, and approval workflows.
*   **Daemon (`leashd`):** The core background service that evaluates policies and manages resource access.

## Building and Running

### Installation

*   **Standard Installation:** `pip install leash-ai`
*   **Development Installation:** `pip install -e .` (from the project root, typically `design/`)

### Starting the Daemon

*   To start the Leash AI daemon:
    ```bash
    leash start
    ```

### Storing Secrets (Example for macOS Keychain)

*   To store an API key in the macOS Keychain, making it accessible to Leash AI:
    ```bash
    security add-generic-password -s leash-ai -a "anthropic/api-key" -w "$ANTHROPIC_KEY"
    ```

### Loading Policies

*   To add policy definitions from a YAML file:
    ```bash
    leash policy add examples/policies/openclaw-quickstart.yaml
    ```

### Testing

*   The project uses `pytest` for testing, with `pytest-asyncio` for asynchronous tests.
    ```bash
    pytest
    ```

### Linting and Formatting

*   **Formatting:** `black`
*   **Linting:** `ruff`
*   **Type Checking:** `mypy`

    Specific commands can be inferred from `pyproject.toml` (e.g., `black .`, `ruff check .`, `mypy src/`).

## Development Conventions

*   **Python Version:** Requires Python 3.9 or newer.
*   **Code Formatting:** Enforced using `black` with a line length of 100 characters.
*   **Linting:** Performed by `ruff` with a line length of 100 characters.
*   **Type Hinting:** Strictly enforced using `mypy`.
*   **Testing:** Adheres to `pytest` conventions, including asynchronous testing patterns.
*   **Policy Definition:** Policies are defined in human-readable YAML files.
*   **Modularity:** Emphasizes a modular design using Abstract Base Classes for new backend implementations and extensions.
