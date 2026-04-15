package com.gemwallet.android.features.confirm.presents.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.asset
import com.gemwallet.android.model.format
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoBottomSheet
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.InfoSheetEntity.NetworkBalanceRequiredInfo
import com.gemwallet.android.ui.components.list_item.WarningItem
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.AssetIdAction
import com.gemwallet.android.ui.theme.Spacer8
import com.gemwallet.android.ui.theme.alpha50
import com.gemwallet.android.ui.theme.smallIconSize
import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.domains.confirm.ConfirmState
import com.gemwallet.android.features.confirm.presents.toLabel

@Composable
internal fun ConfirmErrorInfo(state: ConfirmState, feeValue: String, isShowBottomSheetInfo: Boolean, onBuy: AssetIdAction) {
    var isShowInfoSheet by remember(isShowBottomSheetInfo) { mutableStateOf(isShowBottomSheetInfo) }

    if (state !is ConfirmState.Error || state.message == ConfirmError.None) {
        return
    }
    val message = state.message
    val infoSheetEntity = message.toInfoSheetEntity(feeValue, onBuy)

    WarningItem(
        title = stringResource(R.string.errors_error_occured),
        message = message.toLabel(),
        color = MaterialTheme.colorScheme.error,
        position = ListPosition.Single,
        onClick = infoSheetEntity?.let { { isShowInfoSheet = true } },
        trailing = infoSheetEntity?.let {
            {
                Row {
                    Icon(
                        modifier = Modifier
                            .clip(RoundedCornerShape(percent = 50))
                            .size(smallIconSize)
                            .clickable(onClick = { isShowInfoSheet = true }),
                        imageVector = Icons.Outlined.Info,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.secondary.copy(alpha = alpha50),
                    )
                    Spacer8()
                }
            }
        },
    )

    if (isShowInfoSheet) {
        InfoBottomSheet(item = infoSheetEntity) { isShowInfoSheet = false }
    }
}

@Composable
private fun ConfirmError.toInfoSheetEntity(feeValue: String, onBuy: AssetIdAction): InfoSheetEntity? = when (this) {
    is ConfirmError.InsufficientFee -> NetworkBalanceRequiredInfo(
        chain = chain,
        value = feeValue,
        actionLabel = stringResource(R.string.asset_buy_asset, chain.asset().symbol),
        action = { onBuy(chain.asset().id) },
    )
    is ConfirmError.MinimumAccountBalanceTooLow -> InfoSheetEntity.MinimumAccountBalanceInfo(
        asset = asset,
        value = asset.format(required.toBigInteger(), dynamicPlace = true),
    )
    is ConfirmError.DustThreshold -> InfoSheetEntity.DustThresholdInfo(chain = chain)
    else -> null
}
