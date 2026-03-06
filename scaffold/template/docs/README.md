# Starter Docs Index

These docs are starter-local supplements.

Use them when the topic is about customizing the generated project itself.
For shared framework behavior, HTTP/API semantics, generated model APIs, and cookbook recipes, use the framework docs in `core-docs` as the primary reference.

## Ownership rule

- Framework docs (`core-docs`): shipped features, request/response semantics, generated model API, cookbook recipes.
- Starter docs (`docs/`): project-local setup, extension playbooks, and migration recipes for the generated app.
- `AGENTS.md`: contributor instructions for extending the generated starter correctly.

## Starter-local guides

- `add-new-portal.md`: add a new portal beyond `user` and `admin`.
- `computed-model-values.md`: extend generated view models with computed helpers.
- `country-iso2-linkage.md`: use `country_iso2` and migrate country references safely.
- `custom-project-commands.md`: add app-specific `./console` commands.
- `realtime-setup.md`: wire websocket state, policy, auth, and publish flow in the starter.

## Practical rule

If the answer should apply to every Rustforge app, document it in framework docs.
If the answer is about how this generated starter is organized or customized, document it here.
