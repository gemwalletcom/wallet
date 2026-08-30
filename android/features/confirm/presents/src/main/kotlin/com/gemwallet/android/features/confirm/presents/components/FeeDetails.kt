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
import androidx.compose.foundation.lazy.itemsIndexed
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
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.Dp
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.asset.aggregates.toAssetInfoDataAggregate
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
import com.gemwallet.android.ui.components.list_item.AssetListItem
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.getBalanceInfo
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
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.FeeUnitType
import uniffi.gemstone.Config
import uniffi.gemstone.GemFeeRate
import uniffi.gemstone.GemFeeService

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun FeeDetails(
    isVisible: Boolean,
    currentFee: FeeUIModel.FeeInfo?,
    selection: FeeSelection,
    feeRates: List<GemFeeRate>,
    feeService: GemFeeService,
    feeAssetInfo: AssetInfo?,
    feeAssets: List<AssetInfo>,
    onSelect: (FeeSelection) -> Unit,
    onSelectFeeAsset: (AssetId) -> Unit,
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

    val selectedTotalFee = feeRates.firstOrNull { it.priority == currentFee.priority.string }
        ?.let { feeService.totalFee(it.gasPriceType).toBigInteger() }
    val feeRateModels = feeRates.map { rate ->
        FeeRateUIModel(
            feeRate = rate,
            feeAsset = feeAssetInfo,
            feeUnitType = feeUnitType,
            feeRateDecimals = decimals,
            totalFee = feeService.totalFee(rate.gasPriceType).toBigInteger(),
            selectedTotalFee = selectedTotalFee,
            selectedFeeAmount = currentFee.amount,
            unitSymbol = unitSymbol,
        )
    }

    val selectedCustomRate = (selection as? FeeSelection.Custom)?.gasPrice
    val showFeeAssets = feeAssets.any { it.asset.id != currentFee.feeAsset.id }
    var page by remember(isVisible) { mutableStateOf(FeeDetailsPage.Details) }
    val customModel = remember(page, currentFee, feeRates, selection) {
        NetworkFeeCustomViewModel(currentFee, feeRates, selection, decimals, maxMultiplier, minimumCustomFeeRate, selectedCustomRate, feeService)
    }
    val navigateToDetails: () -> Unit = { page = FeeDetailsPage.Details }
    val confirmCustomFee: () -> Unit = {
        customModel.rate?.let {
            onSelect(FeeSelection.Custom(it))
            onCancel()
        }
    }
    val onBack: (() -> Unit)? = when (page) {
        FeeDetailsPage.Details -> null
        FeeDetailsPage.CustomFee,
        FeeDetailsPage.FeeAssets -> navigateToDetails
    }
    val onConfirm: (() -> Unit)? = when (page) {
        FeeDetailsPage.Details -> onCancel
        FeeDetailsPage.CustomFee -> confirmCustomFee
        FeeDetailsPage.FeeAssets -> null
    }

    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onCancel,
        skipPartiallyExpanded = true,
        title = null,
        dragHandle = {},
    ) {
        FeeSheetHeader(
            title = stringResource(
                when (page) {
                    FeeDetailsPage.Details -> R.string.transfer_network_fee
                    FeeDetailsPage.CustomFee -> R.string.fee_rate_custom
                    FeeDetailsPage.FeeAssets -> R.string.assets_select_asset
                }
            ),
            onBack = onBack,
            onConfirm = onConfirm,
            isConfirmEnabled = page != FeeDetailsPage.CustomFee || customModel.isConfirmEnabled,
        )
        when (page) {
            FeeDetailsPage.Details -> FeeRates(
                currentFee = currentFee,
                selection = selection,
                feeRateModels = feeRateModels,
                feeAssetInfo = feeAssetInfo,
                unitSymbol = unitSymbol,
                supportsCustomFee = supportsCustomFee,
                customRateText = selectedCustomRate?.let { CustomFee.formatRate(it, decimals, unitSymbol) },
                customFiat = selectedCustomRate?.let { currentFee.fiatAmount },
                showFeeAssets = showFeeAssets,
                onSelect = { onSelect(it); onCancel() },
                onCustom = { page = FeeDetailsPage.CustomFee },
                onFeeAssets = { page = FeeDetailsPage.FeeAssets },
            )
            FeeDetailsPage.CustomFee -> CustomFeeInput(
                model = customModel,
                unitSuffix = unitSuffix,
                unitSymbol = unitSymbol,
            )
            FeeDetailsPage.FeeAssets -> FeeAssets(
                assets = feeAssets,
                selectedAssetId = currentFee.feeAsset.id,
                onSelect = {
                    onSelectFeeAsset(it)
                    page = FeeDetailsPage.Details
                },
            )
        }
    }
}

@Composable
private fun FeeRates(
    currentFee: FeeUIModel.FeeInfo,
    selection: FeeSelection,
    feeRateModels: List<FeeRateUIModel>,
    feeAssetInfo: AssetInfo,
    unitSymbol: String,
    supportsCustomFee: Boolean,
    customRateText: String?,
    customFiat: String?,
    showFeeAssets: Boolean,
    onSelect: (FeeSelection) -> Unit,
    onCustom: () -> Unit,
    onFeeAssets: () -> Unit,
) {
    LazyColumn {
        if (showFeeAssets) {
            item { SubheaderItem(R.string.swap_you_pay) }
            item {
                FeeAssetRow(
                    assetInfo = feeAssetInfo,
                    isSelected = false,
                    showChevron = true,
                    listPosition = ListPosition.Single,
                    onClick = onFeeAssets,
                )
            }
        }
        if (feeRateModels.size > 1) {
            val totalCount = feeRateModels.size + if (supportsCustomFee) 1 else 0
            itemsPositioned(feeRateModels, totalCount = totalCount) { position, feeRate ->
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
                        position = ListPosition.getPosition(feeRateModels.size, totalCount),
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
private fun FeeAssets(
    assets: List<AssetInfo>,
    selectedAssetId: AssetId,
    onSelect: (AssetId) -> Unit,
) {
    LazyColumn {
        itemsIndexed(assets) { index, assetInfo ->
            FeeAssetRow(
                assetInfo = assetInfo,
                isSelected = assetInfo.asset.id == selectedAssetId,
                showChevron = false,
                listPosition = ListPosition.getPosition(index, assets.size),
                onClick = { onSelect(assetInfo.asset.id) },
            )
        }
    }
}

@Composable
private fun FeeAssetRow(
    assetInfo: AssetInfo,
    isSelected: Boolean,
    showChevron: Boolean,
    listPosition: ListPosition,
    onClick: () -> Unit,
) {
    val model = remember(assetInfo) {
        assetInfo.toAssetInfoDataAggregate(displayedAmount = assetInfo.balance.balanceAmount.available)
    }
    AssetListItem(
        asset = model,
        modifier = Modifier.clickable(onClick = onClick),
        listPosition = listPosition,
        badge = assetInfo.asset.symbol.takeUnless { it == model.title },
        trailing = {
            Row(verticalAlignment = Alignment.CenterVertically) {
                getBalanceInfo(model).invoke()
                when {
                    isSelected -> SelectionCheckmark(modifier = Modifier.padding(start = paddingSmall))
                    showChevron -> DataBadgeChevron()
                }
            }
        },
    )
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
    onBack: (() -> Unit)?,
    onConfirm: (() -> Unit)?,
    isConfirmEnabled: Boolean,
) {
    Box(
        modifier = Modifier.fillMaxWidth().padding(paddingSmall),
        contentAlignment = Alignment.Center,
    ) {
        onBack?.let {
            IconButton(
                modifier = Modifier.align(Alignment.CenterStart),
                onClick = it,
                colors = IconButtonDefaults.iconButtonColors(
                    containerColor = MaterialTheme.colorScheme.secondary.copy(alpha = alpha10),
                ),
            ) {
                Icon(imageVector = AppIcons.ArrowBack, contentDescription = null)
            }
        }
        Text(text = title, style = MaterialTheme.typography.titleMedium)
        onConfirm?.let {
            IconButton(
                modifier = Modifier.align(Alignment.CenterEnd),
                onClick = it,
                enabled = isConfirmEnabled,
                colors = IconButtonDefaults.iconButtonColors(
                    containerColor = MaterialTheme.colorScheme.secondary.copy(alpha = alpha10),
                ),
            ) {
                Icon(imageVector = AppIcons.Check, contentDescription = null)
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

private enum class FeeDetailsPage {
    Details,
    CustomFee,
    FeeAssets,
}
