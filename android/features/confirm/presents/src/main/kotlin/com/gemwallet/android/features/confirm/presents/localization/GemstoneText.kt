package com.gemwallet.android.features.confirm.presents.localization

import com.wallet.core.primitives.FeeUnitType
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.asset.title
import com.gemwallet.android.ext.boldMarkdown
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.ext.toGemErrorText
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.components.perpetual.title
import com.wallet.core.primitives.Asset
import java.math.BigInteger
import uniffi.gemstone.GemConfirmButtonKind
import uniffi.gemstone.GemConfirmDestination
import uniffi.gemstone.GemConfirmException
import uniffi.gemstone.GemConfirmErrorDisplay
import uniffi.gemstone.GemConfirmScreen
import uniffi.gemstone.GemConfirmTitle
import uniffi.gemstone.GemValueStyle

internal fun GemConfirmDestination.title(): Int = when (this) {
    is GemConfirmDestination.Recipient -> R.string.transfer_recipient_title
    is GemConfirmDestination.Contract -> R.string.asset_contract
    is GemConfirmDestination.Validator -> R.string.stake_validator
    is GemConfirmDestination.Resource -> R.string.stake_resource
    is GemConfirmDestination.Provider -> R.string.common_provider
}

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
internal fun Throwable.toBroadcastLabel(): String = (this as? GemConfirmException)?.display()?.text()
    ?: toGemErrorText()?.text(LocalContext.current)
    ?: "${stringResource(R.string.errors_transfer_error)}: ${message ?: toString()}"

@Composable
internal fun GemConfirmErrorDisplay.text(): String = when (this) {
    is GemConfirmErrorDisplay.Offline -> stringResource(R.string.errors_network_offline)
    is GemConfirmErrorDisplay.Malicious -> stringResource(R.string.errors_scan_transaction_malicious_description)
    is GemConfirmErrorDisplay.MemoRequired -> stringResource(R.string.errors_scan_transaction_memo_required, symbol)
    is GemConfirmErrorDisplay.FeeRatesMissing -> stringResource(R.string.errors_unable_estimate_network_fee)
    is GemConfirmErrorDisplay.Cancelled -> stringResource(R.string.errors_cancelled)
    is GemConfirmErrorDisplay.AccountMissing -> stringResource(R.string.errors_wallet_account_missing)
    is GemConfirmErrorDisplay.Unknown -> stringResource(R.string.errors_unknown)
    is GemConfirmErrorDisplay.BalanceRequired -> {
        val asset = asset.toPrimitives()
        stringResource(
            R.string.info_balance_required_description,
            amount(requirement.required, asset).boldMarkdown(),
            amount(requirement.available, asset).boldMarkdown(),
            amount(requirement.shortfall, asset).boldMarkdown(),
        )
    }
    is GemConfirmErrorDisplay.NetworkFeeRequired -> {
        val asset = asset.toPrimitives()
        stringResource(
            R.string.info_insufficient_network_fee_balance_description,
            amount(requirement.required, asset).boldMarkdown(),
            asset.id.chain.networkName().boldMarkdown(),
            amount(requirement.available, asset).boldMarkdown(),
            amount(requirement.shortfall, asset).boldMarkdown(),
        )
    }
    is GemConfirmErrorDisplay.NetworkFeeMissing ->
        stringResource(R.string.transfer_insufficient_network_fee_balance, asset.toPrimitives().title.boldMarkdown())
    is GemConfirmErrorDisplay.MinimumAccountBalance ->
        stringResource(R.string.transfer_minimum_account_balance, amount(required, asset.toPrimitives()).boldMarkdown())
    is GemConfirmErrorDisplay.SwapMinimum -> {
        val asset = asset.toPrimitives()
        stringResource(
            R.string.info_swap_minimum_amount_description,
            providerName.boldMarkdown(),
            amount(requirement.required, asset).boldMarkdown(),
            amount(requirement.available, asset).boldMarkdown(),
            amount(requirement.shortfall, asset).boldMarkdown(),
        )
    }
    is GemConfirmErrorDisplay.DustThreshold -> stringResource(R.string.errors_dust_threshold_short)
    is GemConfirmErrorDisplay.InsufficientFunds -> stringResource(R.string.info_insufficient_balance_title)
    is GemConfirmErrorDisplay.Message -> msg
}

private fun amount(value: BigInteger, asset: Asset): String = ValueFormatter(style = GemValueStyle.FULL).string(value, asset)

@Composable
internal fun FeeUnitType.suffix(assetSymbol: String): String = when (this) {
    FeeUnitType.SatVb -> stringResource(R.string.fee_rate_satvB)
    FeeUnitType.Gwei -> stringResource(R.string.fee_rate_gwei)
    FeeUnitType.Native -> assetSymbol
}
