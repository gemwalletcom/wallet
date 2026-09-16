package com.gemwallet.android.features.referral.views

import uniffi.gemstone.GemRewardsState

internal fun previewRewardsState(
    hasReferralCode: Boolean = false,
    hasUsedReferralCode: Boolean = false,
    canInvite: Boolean = false,
    canUseReferralCode: Boolean = false,
    showsInfo: Boolean = false,
    isUnverified: Boolean = false,
    hasPendingReferral: Boolean = false,
    canActivatePendingReferral: Boolean = false,
    referralCode: String? = null,
    usedReferralCode: String? = null,
    verifyAfter: Long? = null,
    disableReason: String? = null,
    referralCountText: String = "0",
    pointsText: String = "0",
) = GemRewardsState(
    hasReferralCode = hasReferralCode,
    hasUsedReferralCode = hasUsedReferralCode,
    canInvite = canInvite,
    canUseReferralCode = canUseReferralCode,
    showsInfo = showsInfo,
    isUnverified = isUnverified,
    hasPendingReferral = hasPendingReferral,
    canActivatePendingReferral = canActivatePendingReferral,
    inviteRewardPoints = 100,
    referralCode = referralCode,
    referralLink = referralCode?.let { "https://gemwallet.com/join?code=$it" },
    usedReferralCode = usedReferralCode,
    verifyAfter = verifyAfter,
    disableReason = disableReason,
    referralCountText = referralCountText,
    pointsText = pointsText,
    redemptions = emptyList(),
)
