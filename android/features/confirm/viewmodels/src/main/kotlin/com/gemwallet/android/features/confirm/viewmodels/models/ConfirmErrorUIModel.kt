package com.gemwallet.android.features.confirm.viewmodels.models

import android.content.Context
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.confirm.FeeUIModel
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.confirm.viewmodels.localization.actionLabel
import com.gemwallet.android.features.confirm.viewmodels.localization.text
import com.gemwallet.android.model.AssetPriceValue
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.InfoSheetEntity.BalanceRequiredInfo
import com.gemwallet.android.ui.components.InfoSheetEntity.NetworkBalanceRequiredInfo
import com.gemwallet.android.ui.components.InfoSheetEntity.NetworkFeeRequiredInfo
import com.gemwallet.android.ui.components.InfoSheetEntity.SwapMinimumAmountInfo
import com.wallet.core.primitives.Asset
import java.math.BigInteger
import uniffi.gemstone.GemAcquireAssetFlow
import uniffi.gemstone.GemBalanceRequirement
import uniffi.gemstone.GemConfirmErrorDisplay
import uniffi.gemstone.GemValueStyle

data class ConfirmErrorUIModel(
    val text: String,
    val info: InfoSheetEntity?,
)

data class AcquireAssetRequest(
    val asset: Asset,
    val buyAmount: Int?,
    val offersOptions: Boolean,
)

internal fun GemConfirmErrorDisplay.uiModel(
    context: Context,
    fee: FeeUIModel.FeeInfo?,
    assetPrice: AssetPriceValue?,
    networkFeeBuyAmount: Int,
    acquireFlow: (Asset) -> GemAcquireAssetFlow,
    onAcquire: (Asset, Int?) -> Unit,
): ConfirmErrorUIModel = ConfirmErrorUIModel(
    text = text(context),
    info = infoSheet(context, fee, assetPrice, networkFeeBuyAmount, acquireFlow, onAcquire),
)

private fun GemConfirmErrorDisplay.infoSheet(
    context: Context,
    fee: FeeUIModel.FeeInfo?,
    assetPrice: AssetPriceValue?,
    networkFeeBuyAmount: Int,
    acquireFlow: (Asset) -> GemAcquireAssetFlow,
    onAcquire: (Asset, Int?) -> Unit,
): InfoSheetEntity? = when (this) {
    is GemConfirmErrorDisplay.BalanceRequired -> {
        val asset = asset.toPrimitives()
        val formatted = requirement.formatted(asset)
        BalanceRequiredInfo(
            asset = asset,
            required = formatted.required,
            available = formatted.available,
            shortfall = formatted.shortfall,
            actionLabel = acquireFlow(asset).actionLabel(context, asset.symbol),
            action = { onAcquire(asset, null) },
        )
    }
    is GemConfirmErrorDisplay.NetworkFeeRequired -> {
        val asset = asset.toPrimitives()
        val formatted = requirement.formatted(asset)
        NetworkBalanceRequiredInfo(
            chain = asset.chain,
            required = fee?.cryptoAmountWithFiat ?: formatted.required,
            available = formatted.available,
            shortfall = formatted.shortfall,
            actionLabel = acquireFlow(asset).actionLabel(context, asset.symbol),
            action = { onAcquire(asset, networkFeeBuyAmount) },
        )
    }
    is GemConfirmErrorDisplay.NetworkFeeMissing -> {
        val asset = asset.toPrimitives()
        NetworkFeeRequiredInfo(
            chain = asset.chain,
            actionLabel = acquireFlow(asset).actionLabel(context, asset.symbol),
            action = { onAcquire(asset, networkFeeBuyAmount) },
        )
    }
    is GemConfirmErrorDisplay.SwapMinimum -> {
        val asset = asset.toPrimitives()
        val formatted = requirement.formatted(asset)
        SwapMinimumAmountInfo(
            provider = provider,
            providerName = providerName,
            required = assetPrice.amountWithFiat(requirement.required, asset),
            available = formatted.available,
            shortfall = formatted.shortfall,
            actionLabel = acquireFlow(asset).actionLabel(context, asset.symbol),
            action = { onAcquire(asset, null) },
        )
    }
    is GemConfirmErrorDisplay.MinimumAccountBalance -> {
        val asset = asset.toPrimitives()
        InfoSheetEntity.MinimumAccountBalanceInfo(
            asset = asset,
            value = ValueFormatter(style = GemValueStyle.AUTO).string(required, asset),
        )
    }
    is GemConfirmErrorDisplay.DustThreshold -> InfoSheetEntity.DustThresholdInfo(chain = chain.requireChain())
    is GemConfirmErrorDisplay.Malicious -> InfoSheetEntity.MaliciousTransactionInfo
    is GemConfirmErrorDisplay.MemoRequired -> InfoSheetEntity.MemoRequiredInfo(symbol)
    is GemConfirmErrorDisplay.Offline,
    is GemConfirmErrorDisplay.FeeRatesMissing,
    is GemConfirmErrorDisplay.Cancelled,
    is GemConfirmErrorDisplay.AccountMissing,
    is GemConfirmErrorDisplay.Unknown,
    is GemConfirmErrorDisplay.InsufficientFunds,
    is GemConfirmErrorDisplay.Payment,
    is GemConfirmErrorDisplay.Message -> null
}

private fun AssetPriceValue?.amountWithFiat(value: BigInteger, asset: Asset): String {
    val amount = ValueFormatter(style = GemValueStyle.AUTO).string(value, asset)
    val fiat = this?.let { formatFiat(it.calculateFiat(value)) }.orEmpty()
    return if (fiat.isEmpty()) amount else "$amount (~$fiat)"
}

private fun GemBalanceRequirement.formatted(asset: Asset): FormattedBalanceRequirement {
    val formatter = ValueFormatter(style = GemValueStyle.AUTO)
    return FormattedBalanceRequirement(
        required = formatter.string(required, asset),
        available = formatter.string(available, asset),
        shortfall = formatter.string(shortfall, asset),
    )
}

private data class FormattedBalanceRequirement(
    val required: String,
    val available: String,
    val shortfall: String,
)
