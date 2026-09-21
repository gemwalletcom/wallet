package com.gemwallet.android.features.referral.views

import uniffi.gemstone.GemFormattedNumber
import uniffi.gemstone.GemNumberDisplay
import uniffi.gemstone.GemNumberNotation
import uniffi.gemstone.GemNumberRounding
import uniffi.gemstone.GemNumberUnit
import uniffi.gemstone.GemPrecision
import uniffi.gemstone.GemRewardsState
import uniffi.gemstone.GemValueTone

internal fun previewRewardsState(
    referralCode: String? = null,
    hasReferralCode: Boolean = false,
    canInvite: Boolean = false,
    canUseReferralCode: Boolean = false,
    showsInfo: Boolean = false,
    canActivatePendingReferral: Boolean = false,
    usedReferralCode: String? = null,
) = GemRewardsState(
    hasReferralCode = hasReferralCode,
    canInvite = canInvite,
    canUseReferralCode = canUseReferralCode,
    showsInfo = showsInfo,
    errorNotice = null,
    statusNotice = null,
    showsPendingActivation = false,
    canActivatePendingReferral = canActivatePendingReferral,
    inviteRewardPoints = GemFormattedNumber(
        value = 100.0,
        unit = GemNumberUnit.Plain,
        display = GemNumberDisplay.Number(GemPrecision.Fraction(0u, 0u)),
        notation = GemNumberNotation.PLAIN,
        tone = GemValueTone.PLAIN,
        rounding = GemNumberRounding.TO_NEAREST,
    ),
    referralCode = referralCode,
    referralLink = null,
    usedReferralCode = usedReferralCode,
    infoRows = emptyList(),
    redemptions = emptyList(),
)
