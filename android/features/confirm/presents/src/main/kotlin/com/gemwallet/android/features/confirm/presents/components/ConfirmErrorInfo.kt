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
import com.gemwallet.android.domains.confirm.ConfirmState
import com.gemwallet.android.domains.confirm.toPrimitives
import com.gemwallet.android.domains.confirm.FeeUIModel
import com.gemwallet.android.domains.fiat.FiatConfig
import com.gemwallet.android.features.confirm.presents.AcquireAssetAction
import com.gemwallet.android.features.confirm.presents.toPreloadLabel
import com.gemwallet.android.ext.toPrimitives
import uniffi.gemstone.GemConfirmException
import uniffi.gemstone.GemSignerError
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
import com.gemwallet.android.ext.requireChain
import java.math.BigInteger

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

    if (state !is ConfirmState.Error) {
        return
    }
    val error = state.error
    val requiredAsset = when (error) {
        is GemConfirmException.InsufficientBalance -> error.asset.toPrimitives()
        is GemConfirmException.InsufficientNetworkFee -> error.asset.toPrimitives()
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
    val infoSheetEntity = error.toInfoSheetEntity(fee, onSelectAcquireAsset)

    WarningItem(
        title = stringResource(R.string.errors_error_occurred),
        message = error.toPreloadLabel(),
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
private fun Throwable.toInfoSheetEntity(
    fee: FeeUIModel.FeeInfo?,
    onAcquireAsset: (Asset, Int?) -> Unit,
): InfoSheetEntity? {
    return when (this) {
        is GemConfirmException.InsufficientBalance -> {
            val asset = asset.toPrimitives()
            val formatted = requirement.toPrimitives().formatted(asset)
            BalanceRequiredInfo(
                asset = asset,
                required = formatted.required,
                available = formatted.available,
                shortfall = formatted.shortfall,
                actionLabel = asset.acquireActionLabel(),
                action = { onAcquireAsset(asset, null) },
            )
        }
        is GemConfirmException.InsufficientNetworkFee -> {
            val asset = asset.toPrimitives()
            val formatted = requirement?.toPrimitives()?.formatted(asset)
            if (formatted == null) {
                NetworkFeeRequiredInfo(
                    chain = asset.chain,
                    actionLabel = asset.acquireActionLabel(),
                    action = { onAcquireAsset(asset, FiatConfig.insufficientNetworkFeeBuyAmount) },
                )
            } else {
                NetworkBalanceRequiredInfo(
                    chain = asset.chain,
                    required = fee?.cryptoAmountWithFiat ?: formatted.required,
                    available = formatted.available,
                    shortfall = formatted.shortfall,
                    actionLabel = asset.acquireActionLabel(),
                    action = { onAcquireAsset(asset, FiatConfig.insufficientNetworkFeeBuyAmount) },
                )
            }
        }
        is GemConfirmException.MinimumAccountBalanceTooLow -> {
            val asset = asset.toPrimitives()
            InfoSheetEntity.MinimumAccountBalanceInfo(
                asset = asset,
                value = ValueFormatter(style = ValueFormatter.Style.Full).string(BigInteger(requirement.required), asset),
            )
        }
        is GemConfirmException.Sign -> InfoSheetEntity.DustThresholdInfo(chain = chain.requireChain()).takeIf { error == GemSignerError.DustThreshold }
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
