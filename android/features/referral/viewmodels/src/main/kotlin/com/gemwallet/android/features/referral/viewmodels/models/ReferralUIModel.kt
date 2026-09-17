package com.gemwallet.android.features.referral.viewmodels.models

import uniffi.gemstone.GemRewardsState

data class ReferralUIModel(
    val hasReferralCode: Boolean,
    val hasUsedReferralCode: Boolean,
    val canInvite: Boolean,
    val canUseReferralCode: Boolean,
    val showsInfo: Boolean,
    val isUnverified: Boolean,
    val hasPendingReferral: Boolean,
    val canActivatePendingReferral: Boolean,
    val inviteRewardPoints: Int,
    val usedReferralCode: String?,
    val verifyAfter: Long?,
    val disableReason: String?,
)

internal fun GemRewardsState.uiModel(): ReferralUIModel = ReferralUIModel(
    hasReferralCode = hasReferralCode,
    hasUsedReferralCode = hasUsedReferralCode,
    canInvite = canInvite,
    canUseReferralCode = canUseReferralCode,
    showsInfo = showsInfo,
    isUnverified = isUnverified,
    hasPendingReferral = hasPendingReferral,
    canActivatePendingReferral = canActivatePendingReferral,
    inviteRewardPoints = inviteRewardPoints,
    usedReferralCode = usedReferralCode,
    verifyAfter = verifyAfter,
    disableReason = disableReason,
)
