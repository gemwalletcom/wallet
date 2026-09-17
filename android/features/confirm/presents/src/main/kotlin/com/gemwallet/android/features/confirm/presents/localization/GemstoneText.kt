package com.gemwallet.android.features.confirm.presents.localization

import com.gemwallet.android.features.confirm.viewmodels.localization.text
import com.wallet.core.primitives.FeeUnitType
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.asset.title
import com.gemwallet.android.ext.toGemErrorText
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.components.perpetual.title
import uniffi.gemstone.GemConfirmButtonKind
import uniffi.gemstone.GemConfirmDestination
import uniffi.gemstone.GemConfirmException
import uniffi.gemstone.GemConfirmScreen
import uniffi.gemstone.GemConfirmTitle


@Composable
internal fun GemConfirmTitle.string(): String = when (this) {
    GemConfirmTitle.Send -> stringResource(R.string.transfer_send_title)
    GemConfirmTitle.Deposit -> stringResource(R.string.wallet_deposit)
    GemConfirmTitle.Withdraw -> stringResource(R.string.transfer_withdraw_title)
    GemConfirmTitle.Swap -> stringResource(R.string.wallet_swap)
    GemConfirmTitle.Approve -> stringResource(R.string.transfer_approve_title)
    GemConfirmTitle.Request -> stringResource(R.string.transfer_review_request)
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
internal fun GemConfirmScreen.buttonLabel(kind: GemConfirmButtonKind): String = when {
    failure?.error is GemConfirmException.AccountMissing -> stringResource(R.string.errors_wallet_account_missing)
    kind == GemConfirmButtonKind.RETRY -> stringResource(R.string.common_try_again)
    else -> stringResource(R.string.transfer_confirm)
}

@Composable
internal fun Throwable.toBroadcastLabel(): String = (this as? GemConfirmException)?.display()?.text(LocalContext.current)
    ?: toGemErrorText()?.text(LocalContext.current)
    ?: "${stringResource(R.string.errors_transfer_error)}: ${message ?: toString()}"

@Composable
internal fun FeeUnitType.suffix(assetSymbol: String): String = when (this) {
    FeeUnitType.SatVb -> stringResource(R.string.fee_rate_satvB)
    FeeUnitType.Gwei -> stringResource(R.string.fee_rate_gwei)
    FeeUnitType.Native -> assetSymbol
}

