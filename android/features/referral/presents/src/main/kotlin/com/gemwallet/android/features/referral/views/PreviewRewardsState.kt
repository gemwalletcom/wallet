package com.gemwallet.android.features.referral.views

import uniffi.gemstone.GemDurationPart
import uniffi.gemstone.GemRewardsState

internal fun previewRewardsState(
    referralCode: String? = null,
    referralCountText: String = "0",
    pointsText: String = "0",
    hasReferralCode: Boolean = false,
    hasUsedReferralCode: Boolean = false,
    canInvite: Boolean = false,
    canUseReferralCode: Boolean = false,
    showsInfo: Boolean = false,
    isUnverified: Boolean = false,
    hasPendingReferral: Boolean = false,
    canActivatePendingReferral: Boolean = false,
    usedReferralCode: String? = null,
    pendingCountdown: List<GemDurationPart> = emptyList(),
    disableReason: String? = null,
) = GemRewardsState(
    hasReferralCode = hasReferralCode,
    hasUsedReferralCode = hasUsedReferralCode,
    canInvite = canInvite,
    canUseReferralCode = canUseReferralCode,
    showsInfo = showsInfo,
    isUnverified = isUnverified,
    hasPendingReferral = hasPendingReferral,
    canActivatePendingReferral = canActivatePendingReferral,
    inviteRewardPointsText = "100",
    referralCode = referralCode,
    referralLink = null,
    usedReferralCode = usedReferralCode,
    pendingCountdown = pendingCountdown,
    disableReason = disableReason,
    referralCountText = referralCountText,
    pointsText = pointsText,
    redemptions = emptyList(),
)
