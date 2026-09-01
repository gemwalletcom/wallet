package com.gemwallet.android.features.confirm.presents.components

import com.gemwallet.android.ui.LocalAssetConfigService
import uniffi.gemstone.GemAcquireAssetFlow
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.confirm.BalanceRequirement
import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.domains.confirm.ConfirmState
import com.gemwallet.android.domains.confirm.FeeUIModel
import com.gemwallet.android.domains.fiat.FiatConfig
import com.gemwallet.android.ext.asset
import com.gemwallet.android.features.confirm.presents.AcquireAssetAction
import com.gemwallet.android.features.confirm.presents.toLabel
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoBottomSheet
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.InfoSheetEntity.BalanceRequiredInfo
import com.gemwallet.android.ui.components.InfoSheetEntity.NetworkBalanceRequiredInfo
import com.gemwallet.android.ui.components.InfoSheetEntity.NetworkFeeRequiredInfo
import com.gemwallet.android.ui.components.list_item.WarningItem
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId

@Composable
internal fun ConfirmErrorInfo(
    state: ConfirmState,
    fee: FeeUIModel.FeeInfo?,
    isShowBottomSheetInfo: Boolean,
    onAcquireAsset: (AcquireAssetAction, AssetId) -> Unit,
) {
    val assetConfig = LocalAssetConfigService.current
    var isShowInfoSheet by remember(isShowBottomSheetInfo) { mutableStateOf(isShowBottomSheetInfo) }
    var isShowGetAssetSheet by remember { mutableStateOf(false) }
    var buyAmount by remember { mutableStateOf<Int?>(null) }

    if (state !is ConfirmState.Error || state.message == ConfirmError.None) {
        return
    }
    val message = state.message
    val requiredAsset = when (message) {
        is ConfirmError.InsufficientBalance -> message.asset
        is ConfirmError.InsufficientFee -> message.chain.asset()
        else -> null
    }
    val onSelectAcquireAsset: (Asset, Int?) -> Unit = { asset, amount ->
        isShowInfoSheet = false
        if (assetConfig.acquireFlow(asset.chain.string) == GemAcquireAssetFlow.OPTIONS) {
            buyAmount = amount
            isShowGetAssetSheet = true
        } else {
            onAcquireAsset(AcquireAssetAction.Buy(amount), asset.id)
        }
    }
    val infoSheetEntity = message.toInfoSheetEntity(fee, onSelectAcquireAsset)

    WarningItem(
        title = stringResource(R.string.errors_error_occurred),
        message = message.toLabel(),
        color = MaterialTheme.colorScheme.error,
        position = ListPosition.Single,
        onClick = infoSheetEntity?.let { { isShowInfoSheet = true } },
    )

    if (isShowInfoSheet) {
        InfoBottomSheet(item = infoSheetEntity) { isShowInfoSheet = false }
    }

    if (isShowGetAssetSheet && requiredAsset != null) {
        GetAssetBottomSheet(
            asset = requiredAsset,
            buyAmount = buyAmount,
            onDismiss = { isShowGetAssetSheet = false },
            onAction = {
                isShowGetAssetSheet = false
                onAcquireAsset(it, requiredAsset.id)
            },
        )
    }
}

@Composable
private fun ConfirmError.toInfoSheetEntity(
    fee: FeeUIModel.FeeInfo?,
    onAcquireAsset: (Asset, Int?) -> Unit,
): InfoSheetEntity? {
    return when (this) {
        is ConfirmError.InsufficientBalance -> {
            val formatted = requirement.formatted(asset)
            BalanceRequiredInfo(
                asset = asset,
                required = formatted.required,
                available = formatted.available,
                shortfall = formatted.shortfall,
                actionLabel = asset.acquireActionLabel(),
                action = { onAcquireAsset(asset, null) },
            )
        }
        is ConfirmError.InsufficientFee -> {
            val asset = chain.asset()
            val formatted = requirement?.formatted(asset)
            if (formatted == null) {
                NetworkFeeRequiredInfo(
                    chain = chain,
                    actionLabel = asset.acquireActionLabel(),
                    action = { onAcquireAsset(asset, FiatConfig.insufficientNetworkFeeBuyAmount) },
                )
            } else {
                NetworkBalanceRequiredInfo(
                    chain = chain,
                    required = fee?.cryptoAmountWithFiat ?: formatted.required,
                    available = formatted.available,
                    shortfall = formatted.shortfall,
                    actionLabel = asset.acquireActionLabel(),
                    action = { onAcquireAsset(asset, FiatConfig.insufficientNetworkFeeBuyAmount) },
                )
            }
        }
        is ConfirmError.MinimumAccountBalanceTooLow -> InfoSheetEntity.MinimumAccountBalanceInfo(
            asset = asset,
            value = ValueFormatter(style = ValueFormatter.Style.Full).string(requirement.required, asset),
        )
        is ConfirmError.DustThreshold -> InfoSheetEntity.DustThresholdInfo(chain = chain)
        else -> null
    }
}

@Composable
private fun Asset.acquireActionLabel(): String = stringResource(
    if (LocalAssetConfigService.current.acquireFlow(chain.string) == GemAcquireAssetFlow.OPTIONS) R.string.asset_get_asset else R.string.asset_buy_asset,
    symbol,
)

private fun BalanceRequirement.formatted(asset: Asset): FormattedBalanceRequirement {
    val formatter = ValueFormatter(style = ValueFormatter.Style.Full)
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
