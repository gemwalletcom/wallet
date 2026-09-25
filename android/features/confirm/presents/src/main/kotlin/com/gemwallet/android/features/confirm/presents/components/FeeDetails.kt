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
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.Dp
import com.gemwallet.android.domains.asset.icon
import com.gemwallet.android.domains.confirm.FeeAssetUIModel
import com.gemwallet.android.domains.confirm.FeeDetailsModel
import com.gemwallet.android.domains.confirm.FeeUIModel
import com.gemwallet.android.features.confirm.presents.localization.suffix
import com.gemwallet.android.features.confirm.viewmodels.models.FeeRateRowUIModel
import com.gemwallet.android.features.confirm.viewmodels.models.FeeSelectionUIModel
import com.gemwallet.android.features.confirm.viewmodels.models.NetworkFeeCustomViewModel
import com.gemwallet.android.features.confirm.viewmodels.models.customFeeRowUIModel
import com.gemwallet.android.features.confirm.viewmodels.models.rowUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.SuffixTextField
import com.gemwallet.android.ui.components.image.AsyncImage
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.getBalanceInfo
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.SheetExpansion
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.alpha10
import com.gemwallet.android.ui.theme.listItemIconSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingLarge
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.FeePriority
import com.wallet.core.primitives.FeeUnitType
import java.math.BigInteger

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun FeeDetails(
    isVisible: Boolean,
    currentFee: FeeUIModel.FeeInfo?,
    feeItems: List<ListItemModel>,
    feeListItem: ListItemModel?,
    selection: FeeSelectionUIModel,
    feeDetailsModel: (FeeUIModel.FeeInfo, FeeAssetUIModel) -> FeeDetailsModel?,
    feeAsset: FeeAssetUIModel?,
    feeAssets: List<FeeAssetUIModel>,
    showFeeAssets: Boolean,
    onSelectPriority: (FeePriority) -> Unit,
    onSelectCustom: (BigInteger) -> Unit,
    onSelectFeeAsset: (AssetId) -> Unit,
    onCancel: () -> Unit,
) {
    currentFee ?: return
    feeAsset ?: return
    val context = LocalContext.current
    val model = remember(currentFee, feeAsset, selection) {
        feeDetailsModel(currentFee, feeAsset)
    } ?: return
    val unitSymbol = feeUnitSuffix(model.feeUnitType, feeAsset.asset.symbol)

    val selectedCustomRate = selection.customRate
    var page by remember(isVisible) { mutableStateOf(FeeDetailsPage.Details) }
    val customModel = remember(page, model, selection) {
        NetworkFeeCustomViewModel(model, selectedCustomRate)
    }
    val navigateToDetails: () -> Unit = { page = FeeDetailsPage.Details }
    val confirmCustomFee: () -> Unit = {
        customModel.rate?.let {
            onSelectCustom(it)
            onCancel()
        }
    }
    val onBack: (() -> Unit)? = when (page) {
        FeeDetailsPage.Details -> null

        FeeDetailsPage.CustomFee,
        FeeDetailsPage.FeeAssets,
        -> navigateToDetails
    }
    val onConfirm: (() -> Unit)? = when (page) {
        FeeDetailsPage.Details -> onCancel
        FeeDetailsPage.CustomFee -> confirmCustomFee
        FeeDetailsPage.FeeAssets -> null
    }

    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onCancel,
        expansion = SheetExpansion.Full,
        title = null,
        dragHandle = {},
    ) {
        FeeSheetHeader(
            title = stringResource(
                when (page) {
                    FeeDetailsPage.Details -> R.string.transfer_network_fee
                    FeeDetailsPage.CustomFee -> R.string.fee_rate_custom
                    FeeDetailsPage.FeeAssets -> R.string.assets_select_asset
                },
            ),
            onBack = onBack,
            onConfirm = onConfirm,
            isConfirmEnabled = page != FeeDetailsPage.CustomFee || customModel.isConfirmEnabled,
        )
        when (page) {
            FeeDetailsPage.Details -> FeeRates(
                feeItems = feeItems,
                feeListItem = feeListItem,
                feeRateRows = model.feeRateModels().map { it.rowUIModel(context) },
                customRow = if (model.supportsCustomFee) {
                    customFeeRowUIModel(context, model.customRate, selectedCustomRate?.let { currentFee.fiatAmount })
                } else {
                    null
                },
                showsOptions = model.showsOptions,
                feeAsset = feeAsset,
                showFeeAssets = showFeeAssets,
                onSelectPriority = {
                    onSelectPriority(it)
                    onCancel()
                },
                onCustom = { page = FeeDetailsPage.CustomFee },
                onFeeAssets = { page = FeeDetailsPage.FeeAssets },
            )

            FeeDetailsPage.CustomFee -> CustomFeeInput(
                model = customModel,
                unitSymbol = unitSymbol,
            )

            FeeDetailsPage.FeeAssets -> FeeAssets(
                assets = feeAssets,
                selectedAssetId = currentFee.feeAsset.id,
                onSelect = {
                    onSelectFeeAsset(it)
                    onCancel()
                },
            )
        }
    }
}

@Composable
private fun FeeRates(
    feeItems: List<ListItemModel>,
    feeListItem: ListItemModel?,
    feeRateRows: List<FeeRateRowUIModel>,
    customRow: FeeRateRowUIModel?,
    showsOptions: Boolean,
    feeAsset: FeeAssetUIModel,
    showFeeAssets: Boolean,
    onSelectPriority: (FeePriority) -> Unit,
    onCustom: () -> Unit,
    onFeeAssets: () -> Unit,
) {
    LazyColumn {
        if (showFeeAssets) {
            item { SubheaderItem(R.string.swap_you_pay) }
            item {
                FeeAssetRow(
                    feeAsset = feeAsset,
                    isSelected = false,
                    listPosition = ListPosition.Single,
                    onClick = onFeeAssets,
                )
            }
        }
        if (showsOptions) {
            val totalCount = feeRateRows.size + if (customRow != null) 1 else 0
            itemsPositioned(feeRateRows, totalCount = totalCount) { position, row ->
                FeeRow(
                    row = row,
                    position = position,
                    onClick = { row.priority?.let { onSelectPriority(it) } },
                )
            }
            if (customRow != null) {
                item {
                    FeeRow(
                        row = customRow,
                        position = ListPosition.getPosition(feeRateRows.size, totalCount),
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
        itemsIndexed(feeItems) { index, item ->
            ListItem(model = item, listPosition = ListPosition.getPosition(index, feeItems.size + 1))
        }
        item {
            feeListItem?.let { ListItem(model = it, listPosition = ListPosition.getPosition(feeItems.size, feeItems.size + 1)) }
        }
    }
}

@Composable
private fun FeeAssets(assets: List<FeeAssetUIModel>, selectedAssetId: AssetId, onSelect: (AssetId) -> Unit) {
    LazyColumn {
        itemsIndexed(assets) { index, feeAsset ->
            FeeAssetRow(
                feeAsset = feeAsset,
                isSelected = feeAsset.asset.id == selectedAssetId,
                listPosition = ListPosition.getPosition(index, assets.size),
                onClick = { onSelect(feeAsset.asset.id) },
            )
        }
    }
}

@Composable
private fun FeeAssetRow(feeAsset: FeeAssetUIModel, isSelected: Boolean, listPosition: ListPosition, onClick: () -> Unit) {
    val asset = feeAsset.asset
    ListItem(
        modifier = Modifier.clickable(onClick = onClick),
        listPosition = listPosition,
        leading = {
            IconWithBadge(
                badge = if (isSelected) {
                    { SelectionCheckmark() }
                } else {
                    null
                },
            ) {
                AsyncImage(model = asset, placeholderText = asset.id.icon().placeholder, size = listItemIconSize)
            }
        },
        title = { ListItemTitleText(asset.symbol) },
        subtitle = asset.name.takeUnless { it == asset.symbol }?.let { { ListItemSupportText(it) } },
        trailing = {
            DataBadgeChevron {
                getBalanceInfo(feeAsset.balance, feeAsset.equivalent, feeAsset.isZeroBalance).invoke()
            }
        },
    )
}

@Composable
private fun ColumnScope.CustomFeeInput(model: NetworkFeeCustomViewModel, unitSymbol: String) {
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
            suffix = unitSymbol,
            placeholder = model.placeholder,
            focusRequester = focusRequester,
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
        )
    }
    Text(
        modifier = Modifier.padding(horizontal = paddingLarge, vertical = paddingHalfSmall),
        text = model.errorText(LocalContext.current).orEmpty(),
        style = MaterialTheme.typography.bodySmall,
        color = MaterialTheme.colorScheme.error,
    )
    ListItem(
        model = model.networkFeeItem(LocalContext.current),
        listPosition = ListPosition.Single,
    )
    LaunchedEffect(Unit) { runCatching { focusRequester.requestFocus() } }
}

@Composable
private fun FeeSheetHeader(title: String, onBack: (() -> Unit)?, onConfirm: (() -> Unit)?, isConfirmEnabled: Boolean) {
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
private fun FeeRow(row: FeeRateRowUIModel, position: ListPosition, onClick: () -> Unit) {
    ListItem(
        modifier = Modifier.clickable { onClick() },
        leading = {
            EmojiCircle(row.emoji, listItemIconSize, row.isSelected)
        },
        title = {
            ListItemTitleText(row.model.title)
        },
        trailing = {
            DataBadgeChevron(isShowChevron = true) {
                Column(horizontalAlignment = Alignment.End) {
                    row.model.subtitle?.let { ListItemTitleText(it) }
                    row.model.subtitleExtra?.let { ListItemSupportText(it) }
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
        badge = if (isSelected) {
            { SelectionCheckmark() }
        } else {
            null
        },
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
private fun feeUnitSuffix(feeUnitType: FeeUnitType?, assetSymbol: String): String = feeUnitType?.suffix(assetSymbol) ?: assetSymbol

private enum class FeeDetailsPage {
    Details,
    CustomFee,
    FeeAssets,
}
