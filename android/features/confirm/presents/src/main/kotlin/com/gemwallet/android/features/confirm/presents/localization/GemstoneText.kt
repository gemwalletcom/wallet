package com.gemwallet.android.features.confirm.presents.localization

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.perpetual.title
import com.wallet.core.primitives.FeeUnitType
import uniffi.gemstone.GemConfirmTitle

@Composable
internal fun GemConfirmTitle.string(): String = when (this) {
    GemConfirmTitle.Send -> stringResource(R.string.transfer_send_title)
    GemConfirmTitle.Deposit -> stringResource(R.string.wallet_deposit)
    GemConfirmTitle.Withdraw -> stringResource(R.string.transfer_withdraw_title)
    GemConfirmTitle.Swap -> stringResource(R.string.wallet_swap)
    GemConfirmTitle.Approve -> stringResource(R.string.transfer_approve_title)
    GemConfirmTitle.Request -> stringResource(R.string.transfer_review_request)
    GemConfirmTitle.Payment -> stringResource(R.string.transfer_payment_title)
    GemConfirmTitle.Stake -> stringResource(R.string.transfer_stake_title)
    GemConfirmTitle.Unstake -> stringResource(R.string.transfer_unstake_title)
    GemConfirmTitle.Redelegate -> stringResource(R.string.transfer_redelegate_title)
    GemConfirmTitle.ClaimRewards -> stringResource(R.string.transfer_claim_rewards_title)
    GemConfirmTitle.Freeze -> stringResource(R.string.transfer_freeze_title)
    GemConfirmTitle.Unfreeze -> stringResource(R.string.transfer_unfreeze_title)
    GemConfirmTitle.ActivateAsset -> stringResource(R.string.transfer_activate_asset_title)
    is GemConfirmTitle.PerpetualOpen -> direction.toPrimitives().title()
    is GemConfirmTitle.PerpetualIncrease -> stringResource(R.string.perpetual_increase_direction, direction.toPrimitives().title())
    is GemConfirmTitle.PerpetualReduce -> stringResource(R.string.perpetual_reduce_direction, direction.toPrimitives().title())
    GemConfirmTitle.PerpetualClose -> stringResource(R.string.perpetual_close_position)
    GemConfirmTitle.PerpetualModify -> stringResource(R.string.perpetual_modify_position)
}

@Composable
internal fun FeeUnitType.suffix(assetSymbol: String): String = when (this) {
    FeeUnitType.SatVb -> stringResource(R.string.fee_rate_satvB)
    FeeUnitType.Gwei -> stringResource(R.string.fee_rate_gwei)
    FeeUnitType.Native -> assetSymbol
}
