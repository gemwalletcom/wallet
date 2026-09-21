package com.gemwallet.android.features.transfer_amount.viewmodels.localization

import android.content.Context
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.localization.stringRes
import uniffi.gemstone.GemAmountErrorDisplay
import uniffi.gemstone.GemAmountTitle
import uniffi.gemstone.GemValueStyle

fun GemAmountTitle.text(context: Context): String = when (this) {
    GemAmountTitle.Send -> context.getString(R.string.transfer_send_title)
    GemAmountTitle.Deposit -> context.getString(R.string.wallet_deposit)
    GemAmountTitle.Withdraw -> context.getString(R.string.wallet_withdraw)
    GemAmountTitle.Stake -> context.getString(R.string.transfer_stake_title)
    GemAmountTitle.Unstake -> context.getString(R.string.transfer_unstake_title)
    GemAmountTitle.Redelegate -> context.getString(R.string.transfer_redelegate_title)
    GemAmountTitle.Rewards -> context.getString(R.string.transfer_claim_rewards_title)
    GemAmountTitle.Freeze -> context.getString(R.string.transfer_freeze_title)
    GemAmountTitle.Unfreeze -> context.getString(R.string.transfer_unfreeze_title)
    is GemAmountTitle.PerpetualOpen -> context.getString(direction.toPrimitives().stringRes())
    is GemAmountTitle.PerpetualIncrease -> context.getString(R.string.perpetual_increase_direction, context.getString(direction.toPrimitives().stringRes()))
    is GemAmountTitle.PerpetualReduce -> context.getString(R.string.perpetual_reduce_direction, context.getString(direction.toPrimitives().stringRes()))
}

fun GemAmountErrorDisplay.text(context: Context): String = when (this) {
    is GemAmountErrorDisplay.None -> ""

    is GemAmountErrorDisplay.InvalidAmount -> context.getString(R.string.errors_invalid_amount)

    is GemAmountErrorDisplay.BelowMinimum -> context.getString(
        R.string.transfer_minimum_amount,
        ValueFormatter(style = GemValueStyle.AUTO).string(minimum, asset.decimals, asset.symbol),
    )

    is GemAmountErrorDisplay.InsufficientBalance -> context.getString(R.string.transfer_insufficient_balance, title)
}
