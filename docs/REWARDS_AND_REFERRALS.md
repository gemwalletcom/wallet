# Rewards and Referrals

## Activation Flow

```
[New User] ─> create_username ─> [Unverified] ─> worker checks activity ─> [Verified] or [Trusted]
```

## Referral Flow

```
User1 (Verified/Trusted/Attribution): shares code
  ─> User2 redeems (POST /devices/rewards/referrals/use)
     ─> delay = compute_verification_delay(base, multiplier, referrer_status)
        Trusted referrer: no delay (immediate verification)
        Verified/Attribution referrer: base_delay / verified_multiplier
        Other: base_delay
     ─> if delay: status = Pending, verify_after = now + delay
     ─> if no delay: verified immediately, both get rewards unless User1 is Attribution
  ─> delay passes
  ─> User2 calls same endpoint again
     ─> status reset to Unverified, verify_after cleared
     ─> referral marked verified_at = now
     ─> both get reward events (InviteNew / Joined), or only Joined when User1 is Attribution
  ─> worker later promotes User2 Unverified ─> Verified
```

## Validation Pipeline

Two paths depending on whether the referred user is confirming a pending referral:

**Pending confirmation path** (user already redeemed, delay passed, calling again):
1. `is_pending_referral` — checks Pending status, matching referrer+device+unverified referral
2. `get_referrer_info` — verifies referrer can still refer (Verified/Trusted/Attribution)
3. `use_or_verify_referral` — confirms the referral and creates normal reward events for the referred user; Attribution referrers receive none

**New referral path** (first-time redemption):
1. `get_referrer_info` — fetches referrer status, referral_count, wallet_id (single query)
2. Attribution referrer — validates referral identity, records the referral without a risk signal, and skips to the normal referred-user verification delay
3. Other referrers — apply cooldown, hourly, daily, and weekly rate limits
4. `validate_referral_use` — device/wallet eligibility, subscription age, self-refer check
5. DB connection released
6. Android device token validation (async)
7. IP check + geo restrictions (async, tor, ineligible countries)
8. New DB connection acquired
9. Global rate limits — daily total, per-device, per-IP (daily + weekly), per-country
10. Risk scoring — fingerprint, abuse patterns, device model rings
11. Signal storage + threshold check

## Statuses

| Status | Can Invite | Description |
|--------|-----------|-------------|
| `Unverified` | No | Default after username creation. Awaiting promotion by worker. |
| `Pending` | No | Used a referral code, awaiting `verify_after` delay. |
| `Verified` | Yes | Promoted by worker. Can share referral code. |
| `Trusted` | Yes | Higher-tier verified. Higher referral limits, no verification delay. |
| `Attribution` | Yes | Internal attribution account. Skips referrer checks and rewards while referred users follow the normal lifecycle. |
| `Disabled` | No | Account disabled. |

## Worker Promotion

`RewardsEligibilityChecker` promotes `Unverified` users to `Verified` when activity thresholds are met (`RewardsEligibilityActiveDuration`, `RewardsEligibilityTransactionsCount`). No explicit user action needed.

## Client UI States

| State | UI |
|-------|----|
| No username | "Get Started" button |
| `Unverified`, no `verify_after` | Rewards not active yet message |
| `verify_after` in future | "Bonus Pending" + countdown, confirm disabled |
| `verify_after` in past | "Your bonus is ready!", confirm enabled |
| `Verified`/`Trusted` | Invite Friends + share button |
| `Attribution` | Internal account; no redemption options or referrer rewards |
| `Disabled` | Error with `disable_reason` |

## Key Config

| Key | Purpose |
|-----|---------|
| `RewardsEligibilityActiveDuration` | Min activity duration for promotion |
| `RewardsEligibilityTransactionsCount` | Min confirmed transactions for promotion |
| `RewardsTimerEligibilityChecker` | Worker check interval |
| `RewardsEligibilityPromotionLimit` | Max users promoted per worker run |
| `ReferralVerificationDelay` | Base delay before referral confirmation |
| `ReferralVerifiedMultiplier` | Divides delay for Verified referrers (also scales rate limits) |
| `ReferralTrustedMultiplier` | Scales rate limits for Trusted referrers (delay = 0) |
