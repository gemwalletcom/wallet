package com.gemwallet.android.features.confirm.presents.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.Dp
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.feeUnitType
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.PropertyNetworkFee
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.alpha10
import com.gemwallet.android.ui.theme.listItemIconSize
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingLarge
import com.gemwallet.android.domains.confirm.FeeRateUIModel
import com.gemwallet.android.domains.confirm.FeeUIModel
import com.wallet.core.primitives.FeePriority
import uniffi.gemstone.GemFeeRate

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun FeeDetails(
    isVisible: Boolean,
    currentFee: FeeUIModel.FeeInfo?,
    feeRates: List<GemFeeRate>,
    feeAssetInfo: AssetInfo?,
    onSelect: (FeePriority) -> Unit,
    onCancel: () -> Unit,
) {
    currentFee ?: return
    feeAssetInfo ?: return
    val selectedRate = feeRates.firstOrNull { it.priority == currentFee.priority.string }
    val feeUnitType = feeAssetInfo.asset.chain.feeUnitType()
    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onCancel,
    ) {
        LazyColumn {

            if (feeRates.size > 1) {
                item {
                    SubheaderItem(R.string.transfer_network_fee)
                }
                itemsPositioned(feeRates) { position, item ->
                    val feeRate = FeeRateUIModel(
                        feeRate = item,
                        feeAsset = feeAssetInfo,
                        feeUnitType = feeUnitType,
                        selectedRate = selectedRate,
                        selectedFeeAmount = currentFee.amount,
                    )
                    FeePriorityView(
                        feeRate,
                        feeRate.priority == currentFee.priority,
                        position,
                    ) { onSelect(feeRate.priority) }
                }
                item {
                    Text(
                        modifier = Modifier.padding(horizontal = paddingLarge, vertical = paddingHalfSmall),
                        text = stringResource(R.string.fee_rates_info),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.secondary
                    )
                }
            }
            item {
                PropertyNetworkFee(
                    currentFee.feeAsset.name,
                    currentFee.feeAsset.symbol,
                    currentFee.cryptoAmount,
                    currentFee.fiatAmount,
                    showedCryptoAmount = true,
                )
            }
        }
    }
}

@Composable
private fun FeePriorityView(fee: FeeRateUIModel, isSelected: Boolean, position: ListPosition, onClick: () -> Unit) {
    ListItem(
        modifier = Modifier.clickable { onClick() },
        leading = {
            EmojiCircle(fee.emoji, listItemIconSize, isSelected)
        },
        title = {
            ListItemTitleText(
                text = when (fee.priority) {
                    FeePriority.Fast -> stringResource(R.string.fee_rates_fast)
                    FeePriority.Normal -> stringResource(R.string.fee_rates_normal)
                    FeePriority.Slow -> stringResource(R.string.fee_rates_slow)
                },
            )
        },
        trailing = {
            Column(horizontalAlignment = Alignment.End) {
                ListItemTitleText(fee.price)
                fee.fiatValue.takeIf { it.isNotEmpty() }?.let { fiatValue ->
                    ListItemSupportText(fiatValue)
                }
            }
        },
        listPosition = position,
        minHeight = ListItemDefaults.defaultMinHeight,
    )
}

@Composable
private fun EmojiCircle(emoji: String, size: Dp, isSelected: Boolean = false) {
    IconWithBadge(
        size = size,
        badge = if (isSelected) {{ SelectionCheckmark(modifier = Modifier.fillMaxSize()) }} else null,
    ) {
        Box(
            modifier = Modifier
                .size(size)
                .background(MaterialTheme.colorScheme.secondary.copy(alpha = alpha10), CircleShape),
            contentAlignment = Alignment.Center,
        ) {
            Text(
                text = emoji,
                style = MaterialTheme.typography.headlineSmall,
            )
        }
    }
}
