package com.gemwallet.android.features.confirm.viewmodels.models

import android.content.Context
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.confirm.viewmodels.localization.actionLabel
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.InfoSheetEntity.BalanceRequiredInfo
import com.gemwallet.android.ui.components.InfoSheetEntity.NetworkBalanceRequiredInfo
import com.gemwallet.android.ui.components.InfoSheetEntity.NetworkFeeRequiredInfo
import com.gemwallet.android.ui.components.InfoSheetEntity.SwapMinimumAmountInfo
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemConfirmErrorInfo
import uniffi.gemstone.GemConfirmErrorSheet
import uniffi.gemstone.GemFormattedNumber

data class ConfirmErrorUIModel(val text: String, val info: InfoSheetEntity?)

data class AcquireAssetRequest(val asset: Asset, val buyAmount: Int?, val offersOptions: Boolean, val swapPayAssetId: AssetId? = null)

internal fun GemConfirmErrorInfo.infoSheet(context: Context, networkFeeBuyAmount: Int, onAcquire: (Asset, Int?) -> Unit): InfoSheetEntity? {
    val asset = asset?.toPrimitives()
    val label = { asset?.let { acquire?.actionLabel(context, it.symbol) } }
    val acquireAction = { buyAmount: Int? -> asset?.let { { onAcquire(it, buyAmount) } } }
    return when (val sheet = sheet) {
        GemConfirmErrorSheet.BalanceRequired -> BalanceRequiredInfo(
            asset = asset ?: return null,
            required = required.text(),
            available = available.text(),
            shortfall = shortfall.text(),
            actionLabel = label().orEmpty(),
            action = acquireAction(null) ?: return null,
        )

        GemConfirmErrorSheet.NetworkFeeRequired -> NetworkBalanceRequiredInfo(
            chain = (asset ?: return null).chain,
            required = requiredWithFiat(),
            available = available.text(),
            shortfall = shortfall.text(),
            actionLabel = label().orEmpty(),
            action = acquireAction(networkFeeBuyAmount) ?: return null,
        )

        GemConfirmErrorSheet.NetworkFeeMissing -> NetworkFeeRequiredInfo(
            chain = (asset ?: return null).chain,
            title = title,
            actionLabel = label().orEmpty(),
            action = acquireAction(networkFeeBuyAmount) ?: return null,
        )

        GemConfirmErrorSheet.MinimumAccountBalance -> InfoSheetEntity.MinimumAccountBalanceInfo(
            asset = asset ?: return null,
            value = required.text(),
        )

        is GemConfirmErrorSheet.SwapMinimum -> SwapMinimumAmountInfo(
            provider = sheet.provider,
            providerName = sheet.providerName,
            required = requiredWithFiat(),
            available = available.text(),
            shortfall = shortfall.text(),
            actionLabel = label().orEmpty(),
            action = acquireAction(null) ?: return null,
        )

        is GemConfirmErrorSheet.DustThreshold -> InfoSheetEntity.DustThresholdInfo(chain = sheet.chain.requireChain())

        GemConfirmErrorSheet.Malicious -> InfoSheetEntity.MaliciousTransactionInfo

        is GemConfirmErrorSheet.MemoRequired -> InfoSheetEntity.MemoRequiredInfo(sheet.symbol)
    }
}

// / The amount a sheet asks for, with the fiat it is worth when a price is known.
private fun GemConfirmErrorInfo.requiredWithFiat(): String {
    val amount = required.text()
    val fiat = requiredFiat?.text() ?: return amount
    return "$amount (~$fiat)"
}

private fun GemFormattedNumber?.text(): String = this?.text().orEmpty()
