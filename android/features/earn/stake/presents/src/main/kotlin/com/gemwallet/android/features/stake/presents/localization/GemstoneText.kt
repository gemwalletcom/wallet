package com.gemwallet.android.features.stake.presents.localization

import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemStakeAction
import uniffi.gemstone.GemStakeInfoRow
import uniffi.gemstone.GemStakeSection

@StringRes
internal fun GemStakeAction.stringRes(): Int = when (this) {
    GemStakeAction.CLAIM_REWARDS -> R.string.transfer_claim_rewards_title
    GemStakeAction.STAKE -> R.string.transfer_stake_title
    GemStakeAction.FREEZE -> R.string.transfer_freeze_title
    GemStakeAction.UNFREEZE -> R.string.transfer_unfreeze_title
}

@StringRes
internal fun GemStakeSection.stringRes(): Int = when (this) {
    GemStakeSection.MANAGE -> R.string.common_manage
    GemStakeSection.RESOURCES -> R.string.asset_resources
    GemStakeSection.DELEGATIONS -> R.string.stake_delegations
}

@StringRes
internal fun GemStakeInfoRow.stringRes(): Int = when (this) {
    GemStakeInfoRow.APR -> R.string.stake_apr
    GemStakeInfoRow.LOCK_TIME -> R.string.stake_lock_time
    GemStakeInfoRow.MINIMUM_AMOUNT -> R.string.stake_minimum_amount
}
