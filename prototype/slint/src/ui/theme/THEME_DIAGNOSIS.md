# Theme Module Diagnosis

## What was broken

The files in this folder were written with a mix of Slint module patterns that do not all work in the Slint version used by this project (`slint = 1.17`). The main problems were:

1. `export const ...` was used at the top level in the token files. Slint rejected that syntax. Those values needed to be modeled as properties inside an `export global` block instead.
2. `export import ...` was used in `theme.slint`. That is not the supported Slint re-export form here. The supported form is `export { ... } from "...";` or `export * from "...";`.
3. `current_theme` is not a standalone top-level exported value. It is a property on the exported `ThemeState` global, so the correct access form is `ThemeState.current_theme`.

## Root cause

This is primarily a Slint language/version mismatch, not a naming problem. The code was written as if tokens could be re-exported like ordinary module values, but Slint’s module system is stricter:

- exported components, enums, structs, and globals are the supported top-level exports
- properties live inside a global or component
- a single property cannot be re-exported as if it were its own top-level symbol

In practice, that means the folder must use exported globals for shared theme state and token groups, then re-export those globals and enums with Slint’s supported export syntax.

## What was changed

- Token groups were converted to exported globals in:
  - `colors.slint`
  - `elevation.slint`
  - `radius.slint`
  - `spacing.slint`
  - `typography.slint`
- `theme.slint` now uses Slint-supported re-export syntax:
  - `export { ... } from "...";`
  - `export * from "...";` where appropriate
- The theme-aware background in `preview.slint` was updated to read from `Colors.color-background`.

## Important limitation

`current_theme` cannot be exported as a standalone module symbol without changing the design. The safe and correct API is:

```slint
ThemeState.current_theme
```

If a standalone alias is ever needed, it would require adding a new global or changing the surrounding design, which would affect the public surface. This repository intentionally avoids that.

## Why this looked confusing

The folder originally used names and comments that were reasonable from a general module perspective, but Slint modules are not JavaScript or Rust modules. Their export rules are narrower, and some forms that look natural are simply not supported.

The quickest way to avoid this class of issue is to treat each theme file as one exported global or enum set, then re-export those items explicitly from a module file.

## Quick reference

Use these forms:

```slint
export global ThemeState {
    in-out property <Theme> current_theme: Theme.Light;
}

export { Theme, ThemeState, Colors } from "./colors.slint";
export { Spacing } from "./spacing.slint";
export { Typography, TextStyle } from "./typography.slint";
```

Avoid these forms in this project’s Slint version:

```slint
export const foo = 1;
export import * from "./file.slint";
export import { Foo } from "./file.slint";
```