# Contacts

Saved addresses with a name, so the user never types an address twice.

```mermaid
flowchart LR
    A[Contacts] --> B[Create New Contact] --> C[Name] --> D[Add address] --> E[Pick network] --> F[Type, paste or scan] --> G[Saved]
    H[Recipient or transaction] --> I[Add to Contact] --> D
```

- A contact has a name, an optional description, and one or more addresses, each on a network, with a memo where that network uses one.
- An address can be typed as a name and is resolved.
- When sending, the recipient screen offers the contacts that have an address on that network.
- From a recipient or a transaction the user saves the address as a new contact or adds it to an existing one.

## Rules

- Contacts belong to the app, not to a wallet, so every wallet sees the same list.
