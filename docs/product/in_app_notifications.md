# In-app Notifications

The Notifications list in Settings: messages for the user, each able to open the screen or page it is about.

```mermaid
flowchart LR
    A[Settings] --> B[Notifications] --> C[Messages, newest tagged New] --> D[Tap one] --> E{Link the app handles?}
    E -- yes --> F[That screen opens]
    E -- no --> G[Browser]
```

1. The Notifications list in Settings shows the messages for the user.
2. A notification with a link opens it.

## Expected results

| When | Expected | Why |
|---|---|---|
| The link is one the app handles | that screen opens in the app | |
| Any other link | it opens in the browser | |
| A notification arrived since the last visit | tagged "New", for this visit only | |
| Notifications opened on another wallet | this wallet's unseen notifications stay "New" | each wallet's list is read on its own |

## Platform differences

None recorded.
