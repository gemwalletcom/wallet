package com.gemwallet.android.features.confirm.presents.components

import uniffi.gemstone.GemAcquireAssetFlow
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.asset.chain
import uniffi.gemstone.GemConfirmFailure
import uniffi.gemstone.GemConfirmStage
import com.gemwallet.android.domains.confirm.FeeUIModel
import com.gemwallet.android.features.confirm.presents.AcquireAssetAction
import com.gemwallet.android.features.confirm.presents.localization.text
import com.gemwallet.android.ext.toPrimitives
import uniffi.gemstone.GemBalanceRequirement
import uniffi.gemstone.GemConfirmErrorDisplay
import com.gemwallet.android.model.AssetPriceValue
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoBottomSheet
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.InfoSheetEntity.BalanceRequiredInfo
import com.gemwallet.android.ui.components.InfoSheetEntity.SwapMinimumAmountInfo
import com.gemwallet.android.ui.components.InfoSheetEntity.NetworkBalanceRequiredInfo
import com.gemwallet.android.ui.components.InfoSheetEntity.NetworkFeeRequiredInfo
import com.gemwallet.android.ui.components.list_item.WarningItem
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.gemwallet.android.ext.requireChain
import java.math.BigInteger
import uniffi.gemstone.GemValueStyle

@Composable
internal fun ConfirmErrorInfo(
    failure: GemConfirmFailure?,
    fee: FeeUIModel.FeeInfo?,
    isShowBottomSheetInfo: Boolean,
    onDismissBottomSheetInfo: () -> Unit,
    assetPrice: AssetPriceValue?,
    acquireFlow: (Asset) -> GemAcquireAssetFlow,
    networkFeeBuyAmount: Int,
    onAcquireAsset: (AcquireAssetAction, AssetId) -> Unit,
) {
    var isShowInfoSheet by remember { mutableStateOf(false) }
    var isShowGetAssetSheet by remember { mutableStateOf(false) }
    var buyAmount by remember { mutableStateOf<Int?>(null) }

    val error = failure?.takeIf { it.stage == GemConfirmStage.LOAD }?.error ?: return
    val display = error.display()
    val requiredAsset = when (display) {
        is GemConfirmErrorDisplay.BalanceRequired -> display.asset.toPrimitives()
        is GemConfirmErrorDisplay.NetworkFeeRequired -> display.asset.toPrimitives()
        is GemConfirmErrorDisplay.NetworkFeeMissing -> display.asset.toPrimitives()
        is GemConfirmErrorDisplay.SwapMinimum -> display.asset.toPrimitives()
        is GemConfirmErrorDisplay.MinimumAccountBalance,
        is GemConfirmErrorDisplay.DustThreshold,
        is GemConfirmErrorDisplay.Offline,
        is GemConfirmErrorDisplay.Malicious,
        is GemConfirmErrorDisplay.MemoRequired,
        is GemConfirmErrorDisplay.FeeRatesMissing,
        is GemConfirmErrorDisplay.Cancelled,
        is GemConfirmErrorDisplay.AccountMissing,
        is GemConfirmErrorDisplay.Unknown,
        is GemConfirmErrorDisplay.InsufficientFunds,
        is GemConfirmErrorDisplay.Message -> null
    }
    val onSelectAcquireAsset: (Asset, Int?) -> Unit = { asset, amount ->
        isShowInfoSheet = false
        onDismissBottomSheetInfo()
        if (acquireFlow(asset) == GemAcquireAssetFlow.OPTIONS) {
            buyAmount = amount
            isShowGetAssetSheet = true
        } else {
            onAcquireAsset(AcquireAssetAction.Buy(amount), asset.id)
        }
    }
    val infoSheetEntity = display.toInfoSheetEntity(fee, assetPrice, acquireFlow, networkFeeBuyAmount, onSelectAcquireAsset)

    WarningItem(
        title = stringResource(R.string.errors_error_occurred),
        message = display.text(),
        color = MaterialTheme.colorScheme.error,
        position = ListPosition.Single,
        onClick = infoSheetEntity?.let { { isShowInfoSheet = true } },
    )

    if (isShowInfoSheet || isShowBottomSheetInfo) {
        InfoBottomSheet(item = infoSheetEntity) {
            isShowInfoSheet = false
            onDismissBottomSheetInfo()
        }
    }

    GetAssetBottomSheet(
        asset = requiredAsset?.takeIf { isShowGetAssetSheet },
        buyAmount = buyAmount,
        onDismiss = { isShowGetAssetSheet = false },
        onAction = { action ->
            isShowGetAssetSheet = false
            requiredAsset?.let { onAcquireAsset(action, it.id) }
        },
    )
}

@Composable
private fun GemConfirmErrorDisplay.toInfoSheetEntity(
    fee: FeeUIModel.FeeInfo?,
    assetPrice: AssetPriceValue?,
    acquireFlow: (Asset) -> GemAcquireAssetFlow,
    networkFeeBuyAmount: Int,
    onAcquireAsset: (Asset, Int?) -> Unit,
): InfoSheetEntity? = when (this) {
    is GemConfirmErrorDisplay.BalanceRequired -> {
        val asset = asset.toPrimitives()
        val formatted = requirement.formatted(asset)
        BalanceRequiredInfo(
            asset = asset,
            required = formatted.required,
            available = formatted.available,
            shortfall = formatted.shortfall,
            actionLabel = asset.acquireActionLabel(acquireFlow(asset)),
            action = { onAcquireAsset(asset, null) },
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
            actionLabel = asset.acquireActionLabel(acquireFlow(asset)),
            action = { onAcquireAsset(asset, networkFeeBuyAmount) },
        )
    }
    is GemConfirmErrorDisplay.NetworkFeeMissing -> {
        val asset = asset.toPrimitives()
        NetworkFeeRequiredInfo(
            chain = asset.chain,
            actionLabel = asset.acquireActionLabel(acquireFlow(asset)),
            action = { onAcquireAsset(asset, networkFeeBuyAmount) },
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
            actionLabel = asset.acquireActionLabel(acquireFlow(asset)),
            action = { onAcquireAsset(asset, null) },
        )
    }
    is GemConfirmErrorDisplay.MinimumAccountBalance -> {
        val asset = asset.toPrimitives()
        InfoSheetEntity.MinimumAccountBalanceInfo(
            asset = asset,
            value = ValueFormatter(style = GemValueStyle.FULL).string(required, asset),
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
    is GemConfirmErrorDisplay.Message -> null
}

@Composable
private fun Asset.acquireActionLabel(flow: GemAcquireAssetFlow): String = stringResource(
    if (flow == GemAcquireAssetFlow.OPTIONS) R.string.asset_get_asset else R.string.asset_buy_asset,
    symbol,
)

private fun AssetPriceValue?.amountWithFiat(value: BigInteger, asset: Asset): String {
    val amount = ValueFormatter(style = GemValueStyle.FULL).string(value, asset)
    val fiat = this?.let { formatFiat(it.calculateFiat(value)) }.orEmpty()
    return if (fiat.isEmpty()) amount else "$amount (~$fiat)"
}

private fun GemBalanceRequirement.formatted(asset: Asset): FormattedBalanceRequirement {
    val formatter = ValueFormatter(style = GemValueStyle.FULL)
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
