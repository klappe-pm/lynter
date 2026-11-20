---
name: lynter
description: Settings interface of the Lynter Obsidian plugin, which inherits the host theme.
colors:
  accent: "var(--interactive-accent, #7c3aed)"
  on-accent: "var(--text-on-accent, #ffffff)"
  surface: "var(--background-primary, #ffffff)"
  surface-alt: "var(--background-secondary, #f6f6f6)"
  ink: "var(--text-normal, #222222)"
  muted: "var(--text-muted, #5c5c5c)"
  faint: "var(--text-faint, #ababab)"
  error: "var(--text-error, #e93147)"
  outline: "var(--background-modifier-border, #e0e0e0)"
roles:
  primary: "{colors.accent}"
  on-primary: "{colors.on-accent}"
  surface: "{colors.surface}"
  surface-variant: "{colors.surface-alt}"
  on-surface: "{colors.ink}"
  on-surface-variant: "{colors.muted}"
  outline: "{colors.outline}"
  error: "{colors.error}"
typography:
  body-medium:
    fontFamily: "var(--font-interface, sans-serif)"
    fontSize: "var(--font-ui-medium, 15px)"
    fontWeight: 400
    lineHeight: 1.5
  label-small:
    fontFamily: "var(--font-interface, sans-serif)"
    fontSize: "var(--font-ui-small, 13px)"
    fontWeight: 400
    lineHeight: 1.3
spacing:
  unit: "var(--size-4-1, 4px)"
rounded:
  small: "var(--radius-s, 4px)"
  medium: "var(--radius-m, 8px)"
---

# Lynter design

The plugin renders inside Obsidian and follows the user's theme, so every token is a runtime reference to an Obsidian custom property with a fallback that is used only outside Obsidian. `ds.config.json` lists the Obsidian prefixes in `hostVariables`, so `ds lint` reports hardcoded colors in the settings interface without flagging the host's own variables. The plugin takes no dependency on the design-system package: run `node ../design-system/bin/ds.mjs sync` from a sibling checkout and commit the generated `design/` files.
