package com.gemwallet.android.features.referral.views

import com.gemwallet.android.features.referral.viewmodels.models.ReferralUIModel

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
    verifyAfter: Long? = null,
    disableReason: String? = null,
) = ReferralUIModel(
    hasReferralCode = hasReferralCode,
    hasUsedReferralCode = hasUsedReferralCode,
    canInvite = canInvite,
    canUseReferralCode = canUseReferralCode,
    showsInfo = showsInfo,
    isUnverified = isUnverified,
    hasPendingReferral = hasPendingReferral,
    canActivatePendingReferral = canActivatePendingReferral,
    inviteRewardPoints = 100,
    usedReferralCode = usedReferralCode,
    verifyAfter = verifyAfter,
    disableReason = disableReason,
)
