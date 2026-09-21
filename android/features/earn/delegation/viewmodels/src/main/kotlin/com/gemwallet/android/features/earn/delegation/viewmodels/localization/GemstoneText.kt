package com.gemwallet.android.features.earn.delegation.viewmodels.localization

import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemDelegationAction

@StringRes
internal fun GemDelegationAction.stringRes(): Int = when (this) {
    GemDelegationAction.REDELEGATE -> R.string.transfer_redelegate_title
    GemDelegationAction.STAKE -> R.string.transfer_stake_title
    GemDelegationAction.UNSTAKE -> R.string.transfer_unstake_title
    GemDelegationAction.WITHDRAW -> R.string.transfer_withdraw_title
    GemDelegationAction.DEPOSIT -> R.string.wallet_deposit
}
