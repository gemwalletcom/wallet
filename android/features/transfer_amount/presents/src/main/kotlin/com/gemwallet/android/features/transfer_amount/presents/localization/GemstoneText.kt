package com.gemwallet.android.features.transfer_amount.presents.localization

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.localization.stringRes
import com.wallet.core.primitives.PerpetualDirection
import uniffi.gemstone.GemAmountErrorDisplay
import uniffi.gemstone.GemAmountTitle
import uniffi.gemstone.GemValueStyle

@Composable
fun GemAmountTitle.asString(): String = when (this) {
    GemAmountTitle.Send -> stringResource(R.string.transfer_send_title)
    GemAmountTitle.Deposit -> stringResource(R.string.wallet_deposit)
    GemAmountTitle.Withdraw -> stringResource(R.string.wallet_withdraw)
    GemAmountTitle.Stake -> stringResource(R.string.transfer_stake_title)
    GemAmountTitle.Unstake -> stringResource(R.string.transfer_unstake_title)
    GemAmountTitle.Redelegate -> stringResource(R.string.transfer_redelegate_title)
    GemAmountTitle.Rewards -> stringResource(R.string.transfer_claim_rewards_title)
    GemAmountTitle.Freeze -> stringResource(R.string.transfer_freeze_title)
    GemAmountTitle.Unfreeze -> stringResource(R.string.transfer_unfreeze_title)
    is GemAmountTitle.PerpetualOpen -> stringResource(direction.toPrimitives().stringRes())
    is GemAmountTitle.PerpetualIncrease -> stringResource(R.string.perpetual_increase_direction, stringResource(direction.toPrimitives().stringRes()))
    is GemAmountTitle.PerpetualReduce -> stringResource(R.string.perpetual_reduce_direction, stringResource(direction.toPrimitives().stringRes()))
}

@Composable
fun GemAmountErrorDisplay.text(): String = when (this) {
    is GemAmountErrorDisplay.None -> ""

    is GemAmountErrorDisplay.InvalidAmount -> stringResource(R.string.errors_invalid_amount)

    is GemAmountErrorDisplay.BelowMinimum -> stringResource(
        R.string.transfer_minimum_amount,
        ValueFormatter(style = GemValueStyle.AUTO).string(minimum, asset.decimals, asset.symbol),
    )

    is GemAmountErrorDisplay.InsufficientBalance -> stringResource(R.string.transfer_insufficient_balance, title)
}
