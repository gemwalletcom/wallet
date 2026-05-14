package com.gemwallet.android.features.transfer_amount.presents

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.transfer_amount.viewmodels.AmountTitle
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.PerpetualDirection

@Composable
fun AmountTitle.asString(): String = when (this) {
    AmountTitle.Send -> stringResource(R.string.transfer_send_title)
    is AmountTitle.Stake -> stringResource(when (action) {
        is AmountParams.Stake.Delegate -> R.string.transfer_stake_title
        is AmountParams.Stake.Undelegate -> R.string.transfer_unstake_title
        is AmountParams.Stake.Redelegate -> R.string.transfer_redelegate_title
        is AmountParams.Stake.Withdraw -> R.string.transfer_withdraw_title
        is AmountParams.Stake.Rewards -> R.string.transfer_rewards_title
        is AmountParams.Stake.Freeze -> R.string.transfer_freeze_title
        is AmountParams.Stake.Unfreeze -> R.string.transfer_unfreeze_title
    })
    is AmountTitle.Perpetual -> stringResource(when (direction) {
        PerpetualDirection.Short -> R.string.perpetual_short
        else -> R.string.perpetual_long
    })
}
