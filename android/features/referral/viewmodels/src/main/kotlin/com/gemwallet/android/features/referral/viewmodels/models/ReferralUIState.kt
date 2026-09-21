package com.gemwallet.android.features.referral.viewmodels.models

import com.gemwallet.android.model.text
import uniffi.gemstone.GemIncomingCode
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemRewardsState

data class ReferralUIState(
    val joinPointsCost: String = "",
    val canInvite: Boolean = false,
    val hasCode: Boolean = false,
    val canUseReferralCode: Boolean = false,
    val errorNotice: GemListRow? = null,
    val statusNotice: GemListRow? = null,
    val pendingCode: String? = null,
    val canActivatePending: Boolean = false,
    val showsInfo: Boolean = false,
)

fun GemRewardsState.uiState() = ReferralUIState(
    joinPointsCost = inviteRewardPoints.text(),
    canInvite = canInvite,
    hasCode = hasReferralCode,
    canUseReferralCode = canUseReferralCode,
    errorNotice = errorNotice,
    statusNotice = statusNotice,
    pendingCode = usedReferralCode?.takeIf { showsPendingActivation },
    canActivatePending = canActivatePendingReferral,
    showsInfo = showsInfo,
)

data class IncomingCodeUIModel(val confirm: String? = null, val activate: String? = null)

fun GemIncomingCode?.uiModel() = IncomingCodeUIModel(
    confirm = (this as? GemIncomingCode.Confirm)?.code,
    activate = (this as? GemIncomingCode.Activate)?.code,
)
