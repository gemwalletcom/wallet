package com.gemwallet.android.features.confirm.presents.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.Dp
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.confirm.CustomFee
import com.gemwallet.android.domains.confirm.FeeRateUIModel
import com.gemwallet.android.domains.confirm.FeeUIModel
import com.gemwallet.android.ext.feeRateDecimals
import com.gemwallet.android.ext.feeUnitType
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.FeeSelection
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.SuffixTextField
import com.gemwallet.android.ui.components.title
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyNetworkFee
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.alpha10
import com.gemwallet.android.ui.theme.listItemIconSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingLarge
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.FeeUnitType
import uniffi.gemstone.Config
import uniffi.gemstone.GemFeeRate

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun FeeDetails(
    isVisible: Boolean,
    currentFee: FeeUIModel.FeeInfo?,
    selection: FeeSelection,
    feeRates: List<GemFeeRate>,
    feeAssetInfo: AssetInfo?,
    onSelect: (FeeSelection) -> Unit,
    onCancel: () -> Unit,
) {
    currentFee ?: return
    feeAssetInfo ?: return
    val chain = feeAssetInfo.asset.chain
    val feeUnitType = chain.feeUnitType()
    val unitSuffix = feeUnitSuffix(feeUnitType, feeAssetInfo.asset.symbol)
    val unitSymbol = unitSuffix.trim()
    val feeConfig = remember(chain) { Config().getFeeConfig(chain.string) }
    val supportsCustomFee = feeConfig.customFeeEnabled && feeRates.size > 1
    val decimals = feeRateDecimals(feeUnitType, feeConfig, feeAssetInfo.asset.decimals)
    val maxMultiplier = feeConfig.maxMultiplier.toInt()
    val minimumCustomFeeRate = feeConfig.minimumCustomFeeRate?.toLong()?.toBigInteger()

    val selectedCustomRate = (selection as? FeeSelection.Custom)?.gasPrice
    var showCustom by remember(isVisible) { mutableStateOf(false) }
    val customModel = remember(showCustom) {
        NetworkFeeCustomViewModel(currentFee, feeRates, selection, decimals, maxMultiplier, minimumCustomFeeRate, selectedCustomRate)
    }

    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onCancel,
        skipPartiallyExpanded = true,
        title = null,
        dragHandle = {},
    ) {
        FeeSheetHeader(
            title = stringResource(if (showCustom) R.string.fee_rate_custom else R.string.transfer_network_fee),
            navIcon = if (showCustom) AppIcons.ArrowBack else AppIcons.Close,
            onNavigate = { if (showCustom) showCustom = false else onCancel() },
            actionIcon = if (showCustom) AppIcons.Check else null,
            actionEnabled = showCustom && customModel.isConfirmEnabled,
            onAction = {
                customModel.rate?.let {
                    onSelect(FeeSelection.Custom(it))
                    onCancel()
                }
            },
        )
        if (showCustom) {
            CustomFeeInput(
                model = customModel,
                unitSuffix = unitSuffix,
                unitSymbol = unitSymbol,
            )
        } else {
            FeeRates(
                currentFee = currentFee,
                selection = selection,
                feeRates = feeRates,
                feeAssetInfo = feeAssetInfo,
                feeUnitType = feeUnitType,
                decimals = decimals,
                unitSymbol = unitSymbol,
                supportsCustomFee = supportsCustomFee,
                customRateText = selectedCustomRate?.let { CustomFee.formatRate(it, decimals, unitSymbol) },
                customFiat = selectedCustomRate?.let { currentFee.fiatAmount },
                onSelect = { onSelect(it); onCancel() },
                onCustom = { showCustom = true },
            )
        }
    }
}

@Composable
private fun FeeRates(
    currentFee: FeeUIModel.FeeInfo,
    selection: FeeSelection,
    feeRates: List<GemFeeRate>,
    feeAssetInfo: AssetInfo,
    feeUnitType: FeeUnitType?,
    decimals: Int,
    unitSymbol: String,
    supportsCustomFee: Boolean,
    customRateText: String?,
    customFiat: String?,
    onSelect: (FeeSelection) -> Unit,
    onCustom: () -> Unit,
) {
    val selectedRate = feeRates.firstOrNull { it.priority == currentFee.priority.string }
    LazyColumn {
        if (feeRates.size > 1) {
            val totalCount = feeRates.size + if (supportsCustomFee) 1 else 0
            itemsPositioned(feeRates, totalCount = totalCount) { position, item ->
                val feeRate = FeeRateUIModel(item, feeAssetInfo, feeUnitType, decimals, selectedRate, currentFee.amount, unitSymbol)
                FeeRow(
                    emoji = feeRate.emoji,
                    title = feeRate.priority.title(),
                    rate = feeRate.price,
                    fiat = feeRate.fiatValue,
                    isSelected = selection is FeeSelection.Preset && selection.priority == feeRate.priority,
                    position = position,
                    onClick = { onSelect(FeeSelection.Preset(feeRate.priority)) },
                )
            }
            if (supportsCustomFee) {
                item {
                    FeeRow(
                        emoji = "⚙️",
                        title = stringResource(R.string.fee_rate_custom),
                        rate = customRateText,
                        fiat = customFiat,
                        isSelected = selection is FeeSelection.Custom,
                        position = ListPosition.getPosition(feeRates.size, totalCount),
                        onClick = onCustom,
                    )
                }
            }
            item {
                Text(
                    modifier = Modifier.padding(horizontal = paddingLarge, vertical = paddingHalfSmall),
                    text = stringResource(R.string.fee_rates_info),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.secondary,
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

@Composable
private fun ColumnScope.CustomFeeInput(
    model: NetworkFeeCustomViewModel,
    unitSuffix: String,
    unitSymbol: String,
) {
    val focusRequester = remember { FocusRequester() }
    Row(
        modifier = Modifier.fillMaxWidth().listItem(ListPosition.Single).padding(paddingDefault),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        ListItemTitleText(stringResource(R.string.fee_rate_custom))
        SuffixTextField(
            modifier = Modifier.weight(1f),
            value = model.input,
            onValueChange = model::onInputChange,
            suffix = unitSuffix,
            placeholder = model.placeholder,
            focusRequester = focusRequester,
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
        )
    }
    Text(
        modifier = Modifier.padding(horizontal = paddingLarge, vertical = paddingHalfSmall),
        text = when {
            model.isOverMax -> stringResource(R.string.common_maximum_value, "${model.maxRateText} $unitSymbol")
            model.isBelowMinimum -> stringResource(R.string.common_minimum_value, "${model.minRateText} $unitSymbol")
            else -> ""
        },
        style = MaterialTheme.typography.bodySmall,
        color = MaterialTheme.colorScheme.error,
    )
    PropertyNetworkFee(
        model.networkFee.feeAsset.name,
        model.networkFee.feeAsset.symbol,
        model.networkFee.cryptoAmount,
        model.networkFee.fiatAmount,
        showedCryptoAmount = true,
    )
    LaunchedEffect(Unit) { runCatching { focusRequester.requestFocus() } }
}

@Composable
private fun FeeSheetHeader(
    title: String,
    navIcon: ImageVector,
    onNavigate: () -> Unit,
    actionIcon: ImageVector?,
    actionEnabled: Boolean,
    onAction: () -> Unit,
) {
    Box(
        modifier = Modifier.fillMaxWidth().padding(paddingSmall),
        contentAlignment = Alignment.Center,
    ) {
        IconButton(
            modifier = Modifier.align(Alignment.CenterStart),
            onClick = onNavigate,
            colors = IconButtonDefaults.iconButtonColors(
                containerColor = MaterialTheme.colorScheme.secondary.copy(alpha = alpha10),
            ),
        ) {
            Icon(imageVector = navIcon, contentDescription = null)
        }
        Text(text = title, style = MaterialTheme.typography.titleMedium)
        actionIcon?.let { icon ->
            IconButton(
                modifier = Modifier.align(Alignment.CenterEnd),
                onClick = onAction,
                enabled = actionEnabled,
                colors = IconButtonDefaults.iconButtonColors(
                    containerColor = MaterialTheme.colorScheme.secondary.copy(alpha = alpha10),
                ),
            ) {
                Icon(imageVector = icon, contentDescription = null)
            }
        }
    }
}

@Composable
private fun FeeRow(
    emoji: String,
    title: String,
    rate: String?,
    fiat: String?,
    isSelected: Boolean,
    position: ListPosition,
    onClick: () -> Unit,
) {
    ListItem(
        modifier = Modifier.clickable { onClick() },
        leading = {
            EmojiCircle(emoji, listItemIconSize, isSelected)
        },
        title = {
            ListItemTitleText(title)
        },
        trailing = {
            DataBadgeChevron(isShowChevron = true) {
                Column(horizontalAlignment = Alignment.End) {
                    rate?.takeIf { it.isNotEmpty() }?.let { ListItemTitleText(it) }
                    fiat?.takeIf { it.isNotEmpty() }?.let { ListItemSupportText(it) }
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
        badge = if (isSelected) {{ SelectionCheckmark() }} else null,
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

@Composable
private fun feeUnitSuffix(feeUnitType: FeeUnitType?, assetSymbol: String): String = when (feeUnitType) {
    FeeUnitType.SatVb -> stringResource(R.string.fee_rate_satvB, "")
    FeeUnitType.Gwei -> stringResource(R.string.fee_rate_gwei, "")
    else -> " $assetSymbol"
}
