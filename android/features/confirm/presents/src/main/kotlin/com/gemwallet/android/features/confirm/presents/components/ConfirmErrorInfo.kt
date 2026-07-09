package com.gemwallet.android.features.confirm.presents.components

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.asset
import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.domains.confirm.ConfirmState
import com.gemwallet.android.features.confirm.presents.GetNetworkFeeAssetAction
import com.gemwallet.android.features.confirm.presents.toLabel
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoBottomSheet
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.InfoSheetEntity.NetworkBalanceRequiredInfo
import com.gemwallet.android.ui.components.list_item.WarningItem
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain

@Composable
internal fun ConfirmErrorInfo(
    state: ConfirmState,
    feeValue: String,
    isShowBottomSheetInfo: Boolean,
    onGetNetworkFeeAssetAction: (GetNetworkFeeAssetAction, AssetId) -> Unit,
) {
    var isShowInfoSheet by remember(isShowBottomSheetInfo) { mutableStateOf(isShowBottomSheetInfo) }
    var isShowGetFeeAssetSheet by remember { mutableStateOf(false) }

    if (state !is ConfirmState.Error || state.message == ConfirmError.None) {
        return
    }
    val message = state.message
    val feeAsset = (message as? ConfirmError.InsufficientFee)?.chain?.asset()
    val getFeeAssetAction = if (message is ConfirmError.InsufficientFee && message.chain == Chain.Tron) {
        {
            isShowInfoSheet = false
            isShowGetFeeAssetSheet = true
        }
    } else {
        null
    }
    val infoSheetEntity = message.toInfoSheetEntity(
        feeValue = feeValue,
        onBuy = { assetId -> onGetNetworkFeeAssetAction(GetNetworkFeeAssetAction.Buy, assetId) },
        onGetFeeAsset = getFeeAssetAction,
    )

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

    if (isShowGetFeeAssetSheet && feeAsset != null) {
        GetNetworkFeeAssetBottomSheet(
            asset = feeAsset,
            onDismiss = { isShowGetFeeAssetSheet = false },
            onAction = {
                isShowGetFeeAssetSheet = false
                onGetNetworkFeeAssetAction(it, feeAsset.id)
            },
        )
    }
}

@Composable
private fun ConfirmError.toInfoSheetEntity(
    feeValue: String,
    onBuy: (AssetId) -> Unit,
    onGetFeeAsset: (() -> Unit)?,
): InfoSheetEntity? = when (this) {
    is ConfirmError.InsufficientFee -> NetworkBalanceRequiredInfo(
        chain = chain,
        value = feeValue,
        actionLabel = if (onGetFeeAsset != null) {
            stringResource(R.string.asset_get_asset, chain.asset().symbol)
        } else {
            stringResource(R.string.asset_buy_asset, chain.asset().symbol)
        },
        action = { onGetFeeAsset?.invoke() ?: onBuy(chain.asset().id) },
    )
    is ConfirmError.MinimumAccountBalanceTooLow -> InfoSheetEntity.MinimumAccountBalanceInfo(
        asset = asset,
        value = ValueFormatter(style = ValueFormatter.Style.Full).string(required.toBigInteger(), asset),
    )
    is ConfirmError.DustThreshold -> InfoSheetEntity.DustThresholdInfo(chain = chain)
    else -> null
}
