Python source code file structure:
1) Imports. Divided into sections, separated by single newlines. Each section first has the `import _` statements, then the `from _ import _` statements. List of sections:
  - Meta-imports like `annotations.__future__` and the `typing` module
  - Python standard library imports
  - Third-party library imports, grouped by library
  - Imports from own code
2) Two newlines
3) The rest of the fucking owl