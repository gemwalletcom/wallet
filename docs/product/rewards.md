# Rewards

Earn points by inviting friends, and spend them on assets. Rewards belong to one wallet, the one the app picks as the rewards wallet.

```mermaid
sequenceDiagram
    participant U as User
    participant F as Friend
    U->>U: Create Username, it is the referral code
    Note over U: Rewards activate once the wallet has real use
    U->>F: Share the referral link
    F->>F: Redeem code
    Note over F: Bonus Pending, a waiting period
    F->>F: Confirm when the bonus is ready
    Note over U,F: Both get points
```

- The user opens Rewards from Settings and sees their points, their referrals and the ways to spend.
- "Create Username" sets a nickname for the current wallet (letters and digits, 4 to 16 characters); it is the referral code.
- "Invite Friends" says how many points each friend brings, and the user shares the code as a link.
- A friend taps "Redeem code"; after the waiting period "Your bonus is ready!" and both get points.
- "Ways to Spend" lists assets the user can get for points; redeeming asks for confirmation and shows the result on the same screen.

## Rules

- A username is permanent; a wallet that has one cannot set another.
