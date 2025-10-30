---
applyTo: '**.py'
---


# Python Coding Guidelines

These guidelines ensure consistency and quality across all Python code in the project.

## Docstrings

All functions, methods, and classes must include docstrings following the Google style.

Example:
```python
def greet(name: str) -> str:
    """Greets a person by name.

    Args:
        name: The name of the person to greet.

    Returns:
        A greeting message.
    """
    return f"Hello, {name}!"
```

## Comments

Comments should be used sparingly and only to explain *why* a piece of code exists or *what* a complex algorithm is doing, not *how* it works (which should be clear from the code itself).

- **Avoid:** Debugging comments, commented-out code, or comments detailing changes (use Git for this).
- **Remove:** Any existing comments that fall into the "avoid" category.

## Code Formatting, Linting, and Type Checking

Before committing any Python code, ensure it adheres to the following standards:

1.  **Format:** Automatically format your code using `uvx ruff format`.
    ```bash
    uvx ruff format .
    ```
2.  **Lint:** Check for and fix linting issues using `uvx ruff check --fix`.
    ```bash
    uvx ruff check --fix .
    ```
3.  **Type Check:** Perform static type checking using `uvx ty check`.
    ```bash
    uvx ty check .
    ```
