package com.gemwallet.android.features.stake.viewmodels.localization

import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemStakeSection

@StringRes
internal fun GemStakeSection.stringRes(): Int = when (this) {
    GemStakeSection.MANAGE -> R.string.common_manage
    GemStakeSection.RESOURCES -> R.string.asset_resources
    GemStakeSection.DELEGATIONS -> R.string.stake_delegations
}
