package com.gemwallet.android.features.confirm.viewmodels.localization

import android.content.Context
import androidx.annotation.StringRes
import com.gemwallet.android.domains.asset.title
import com.gemwallet.android.domains.confirm.ConfirmProperty
import com.gemwallet.android.ext.boldMarkdown
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.perpetual.title
import com.gemwallet.android.ui.localization.text
import com.wallet.core.primitives.Asset
import java.math.BigInteger
import uniffi.gemstone.GemAcquireAssetFlow
import uniffi.gemstone.GemConfirmDestination
import uniffi.gemstone.GemConfirmErrorDisplay
import uniffi.gemstone.GemValueStyle

fun GemConfirmErrorDisplay.text(context: Context): String = when (this) {
    is GemConfirmErrorDisplay.Offline -> context.getString(R.string.errors_network_offline)
    is GemConfirmErrorDisplay.Malicious -> context.getString(R.string.errors_scan_transaction_malicious_description)
    is GemConfirmErrorDisplay.MemoRequired -> context.getString(R.string.errors_scan_transaction_memo_required, symbol)
    is GemConfirmErrorDisplay.FeeRatesMissing -> context.getString(R.string.errors_unable_estimate_network_fee)
    is GemConfirmErrorDisplay.Cancelled -> context.getString(R.string.errors_cancelled)
    is GemConfirmErrorDisplay.AccountMissing -> context.getString(R.string.errors_wallet_account_missing)
    is GemConfirmErrorDisplay.Unknown -> context.getString(R.string.errors_unknown)
    is GemConfirmErrorDisplay.BalanceRequired -> {
        val asset = asset.toPrimitives()
        context.getString(
            R.string.info_balance_required_description,
            amount(requirement.required, asset).boldMarkdown(),
            amount(requirement.available, asset).boldMarkdown(),
            amount(requirement.shortfall, asset).boldMarkdown(),
        )
    }
    is GemConfirmErrorDisplay.NetworkFeeRequired -> {
        val asset = asset.toPrimitives()
        context.getString(
            R.string.info_insufficient_network_fee_balance_description,
            amount(requirement.required, asset).boldMarkdown(),
            asset.id.chain.networkName().boldMarkdown(),
            amount(requirement.available, asset).boldMarkdown(),
            amount(requirement.shortfall, asset).boldMarkdown(),
        )
    }
    is GemConfirmErrorDisplay.NetworkFeeMissing ->
        context.getString(R.string.transfer_insufficient_network_fee_balance, asset.toPrimitives().title.boldMarkdown())
    is GemConfirmErrorDisplay.MinimumAccountBalance ->
        context.getString(R.string.transfer_minimum_account_balance, amount(required, asset.toPrimitives()).boldMarkdown())
    is GemConfirmErrorDisplay.SwapMinimum -> {
        val asset = asset.toPrimitives()
        context.getString(
            R.string.info_swap_minimum_amount_description,
            providerName.boldMarkdown(),
            amount(requirement.required, asset).boldMarkdown(),
            amount(requirement.available, asset).boldMarkdown(),
            amount(requirement.shortfall, asset).boldMarkdown(),
        )
    }
    is GemConfirmErrorDisplay.DustThreshold -> context.getString(R.string.errors_dust_threshold_short)
    is GemConfirmErrorDisplay.InsufficientFunds -> context.getString(R.string.info_insufficient_balance_title)
    is GemConfirmErrorDisplay.Message -> msg
}

internal fun GemAcquireAssetFlow.actionLabel(context: Context, symbol: String): String = context.getString(
    when (this) {
        GemAcquireAssetFlow.OPTIONS -> R.string.asset_get_asset
        GemAcquireAssetFlow.FIAT -> R.string.asset_buy_asset
    },
    symbol,
)

private fun amount(value: BigInteger, asset: Asset): String = ValueFormatter(style = GemValueStyle.FULL).string(value, asset)

@StringRes
fun GemConfirmDestination.title(): Int = when (this) {
    is GemConfirmDestination.Recipient -> R.string.transfer_recipient_title
    is GemConfirmDestination.Contract -> R.string.asset_contract
    is GemConfirmDestination.Validator -> R.string.stake_validator
    is GemConfirmDestination.Resource -> R.string.stake_resource
    is GemConfirmDestination.Provider -> R.string.common_provider
}

@StringRes
fun ConfirmProperty.Destination.titleRes(): Int = kind?.title() ?: R.string.wallet_connect_app
