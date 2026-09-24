package com.gemwallet.android.features.confirm.viewmodels.models

import android.content.Context
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toAssetId
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
import uniffi.gemstone.GemAcquireAsset
import uniffi.gemstone.GemAcquireAssetFlow
import uniffi.gemstone.GemConfirmErrorInfo
import uniffi.gemstone.GemConfirmErrorSheet
import uniffi.gemstone.GemFormattedNumber

data class ConfirmErrorUIModel(val text: String, val info: InfoSheetEntity?)

data class AcquireAssetRequest(val asset: Asset, val acquire: GemAcquireAsset) {
    val offersOptions: Boolean get() = acquire.flow == GemAcquireAssetFlow.OPTIONS
    val buyAmount: Int? get() = acquire.buyAmount
    val swapPayAssetId: AssetId? get() = acquire.swapPair.payAssetId?.toAssetId()
}

internal fun GemConfirmErrorInfo.infoSheet(context: Context, onAcquire: (Asset, GemAcquireAsset) -> Unit): InfoSheetEntity? {
    val asset = asset?.toPrimitives()
    val label = { asset?.let { acquire?.flow?.actionLabel(context, it.symbol) } }
    val acquireAction = { asset?.let { asset -> acquire?.let { acquire -> { onAcquire(asset, acquire) } } } }
    return when (val sheet = sheet) {
        GemConfirmErrorSheet.BalanceRequired -> BalanceRequiredInfo(
            asset = asset ?: return null,
            required = required.text(),
            available = available.text(),
            shortfall = shortfall.text(),
            actionLabel = label().orEmpty(),
            action = acquireAction() ?: return null,
        )

        GemConfirmErrorSheet.NetworkFeeRequired -> NetworkBalanceRequiredInfo(
            chain = (asset ?: return null).chain,
            required = requiredWithFiat(),
            available = available.text(),
            shortfall = shortfall.text(),
            actionLabel = label().orEmpty(),
            action = acquireAction() ?: return null,
        )

        GemConfirmErrorSheet.NetworkFeeMissing -> NetworkFeeRequiredInfo(
            chain = (asset ?: return null).chain,
            title = title,
            actionLabel = label().orEmpty(),
            action = acquireAction() ?: return null,
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
            action = acquireAction() ?: return null,
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
