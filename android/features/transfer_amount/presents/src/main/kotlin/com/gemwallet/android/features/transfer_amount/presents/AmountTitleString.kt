package com.gemwallet.android.features.transfer_amount.presents

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.perpetual.PerpetualPositionAction
import com.gemwallet.android.features.transfer_amount.viewmodels.AmountTitle
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.PerpetualDirection

@Composable
fun AmountTitle.asString(): String = when (this) {
    AmountTitle.Send -> stringResource(R.string.transfer_send_title)
    AmountTitle.Deposit -> stringResource(R.string.wallet_deposit)
    AmountTitle.Withdraw -> stringResource(R.string.wallet_withdraw)
    is AmountTitle.Stake -> stringResource(when (action) {
        is AmountParams.Stake.Delegate -> R.string.transfer_stake_title
        is AmountParams.Stake.Undelegate -> R.string.transfer_unstake_title
        is AmountParams.Stake.Redelegate -> R.string.transfer_redelegate_title
        is AmountParams.Stake.Withdraw -> R.string.transfer_withdraw_title
        is AmountParams.Stake.Rewards -> R.string.transfer_rewards_title
        is AmountParams.Stake.Freeze -> R.string.transfer_freeze_title
        is AmountParams.Stake.Unfreeze -> R.string.transfer_unfreeze_title
    })
    is AmountTitle.Perpetual -> perpetualTitle(action)
}

@Composable
private fun perpetualTitle(action: PerpetualPositionAction): String {
    val direction = when (action) {
        is PerpetualPositionAction.Open,
        is PerpetualPositionAction.Increase -> action.data.direction
        is PerpetualPositionAction.Reduce -> action.positionDirection
    }
    val directionLabel = stringResource(when (direction) {
        PerpetualDirection.Short -> R.string.perpetual_short
        PerpetualDirection.Long -> R.string.perpetual_long
    })
    return when (action) {
        is PerpetualPositionAction.Open -> directionLabel
        is PerpetualPositionAction.Increase -> stringResource(R.string.perpetual_increase_direction, directionLabel)
        is PerpetualPositionAction.Reduce -> stringResource(R.string.perpetual_reduce_direction, directionLabel)
    }
}
