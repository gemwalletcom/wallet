package com.gemwallet.android.features.confirm.viewmodels.localization

import android.content.Context
import androidx.annotation.StringRes
import com.gemwallet.android.ext.boldMarkdown
import com.gemwallet.android.ext.errorTextOrNull
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.perpetual.title
import com.gemwallet.android.ui.localization.errorText
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.text
import uniffi.gemstone.GemConfirmButtonKind
import uniffi.gemstone.GemConfirmDestination
import uniffi.gemstone.GemConfirmErrorDisplay
import uniffi.gemstone.GemConfirmException
import uniffi.gemstone.GemConfirmScreen
import uniffi.gemstone.GemSubmitMessage

fun GemConfirmErrorDisplay.text(context: Context): String = when (this) {
    is GemConfirmErrorDisplay.Offline -> context.getString(R.string.errors_network_offline)

    is GemConfirmErrorDisplay.Malicious -> context.getString(R.string.errors_scan_transaction_malicious_description)

    is GemConfirmErrorDisplay.MemoRequired -> context.getString(R.string.errors_scan_transaction_memo_required, symbol)

    is GemConfirmErrorDisplay.FeeRatesMissing -> context.getString(R.string.errors_unable_estimate_network_fee)

    is GemConfirmErrorDisplay.Cancelled -> context.getString(R.string.errors_cancelled)

    is GemConfirmErrorDisplay.AccountMissing -> context.getString(R.string.errors_wallet_account_missing)

    is GemConfirmErrorDisplay.Unknown -> context.getString(R.string.errors_unknown)

    is GemConfirmErrorDisplay.BalanceRequired -> context.getString(
        R.string.info_balance_required_description,
        requirement.required.text().boldMarkdown(),
        requirement.available.text().boldMarkdown(),
        requirement.shortfall.text().boldMarkdown(),
    )

    is GemConfirmErrorDisplay.NetworkFeeRequired -> context.getString(
        R.string.info_insufficient_network_fee_balance_description,
        requirement.required.text().boldMarkdown(),
        asset.toPrimitives().id.chain.networkName().boldMarkdown(),
        requirement.available.text().boldMarkdown(),
        requirement.shortfall.text().boldMarkdown(),
    )

    is GemConfirmErrorDisplay.NetworkFeeMissing ->
        context.getString(R.string.transfer_insufficient_network_fee_balance, title.boldMarkdown())

    is GemConfirmErrorDisplay.MinimumAccountBalance ->
        context.getString(R.string.transfer_minimum_account_balance, required.text().boldMarkdown())

    is GemConfirmErrorDisplay.DestinationAccountActivation ->
        context.getString(R.string.transfer_destination_account_activation, required.text().boldMarkdown())

    is GemConfirmErrorDisplay.SwapMinimum -> context.getString(
        R.string.info_swap_minimum_amount_description,
        providerName.boldMarkdown(),
        requirement.required.text().boldMarkdown(),
        requirement.available.text().boldMarkdown(),
        requirement.shortfall.text().boldMarkdown(),
    )

    is GemConfirmErrorDisplay.DustThreshold -> context.getString(R.string.errors_dust_threshold_short)

    is GemConfirmErrorDisplay.InsufficientFunds -> context.getString(R.string.info_insufficient_balance_title)

    is GemConfirmErrorDisplay.Payment -> status.errorText(context)

    is GemConfirmErrorDisplay.Message -> msg
}

@StringRes
fun GemConfirmDestination.title(): Int = when (this) {
    is GemConfirmDestination.Recipient -> R.string.transfer_recipient_title
    is GemConfirmDestination.Contract -> R.string.asset_contract
    is GemConfirmDestination.Validator -> R.string.stake_validator
    is GemConfirmDestination.Resource -> R.string.stake_resource
    is GemConfirmDestination.Provider -> R.string.common_provider
}

internal fun GemConfirmButtonKind.label(context: Context): String = when (this) {
    GemConfirmButtonKind.CONFIRM -> context.getString(R.string.transfer_confirm)
    GemConfirmButtonKind.RETRY -> context.getString(R.string.common_try_again)
    GemConfirmButtonKind.ACCOUNT_MISSING -> context.getString(R.string.errors_wallet_account_missing)
}

internal fun Throwable.broadcastLabel(context: Context): String = (this as? GemConfirmException)?.display()?.text(context)
    ?: errorTextOrNull()?.text(context)
    ?: "${context.getString(R.string.errors_transfer_error)}: ${message ?: toString()}"

fun GemSubmitMessage.text(context: Context): String = when (this) {
    is GemSubmitMessage.Warning -> text.text(context)
    is GemSubmitMessage.Confirmed -> text.string(context)
}
