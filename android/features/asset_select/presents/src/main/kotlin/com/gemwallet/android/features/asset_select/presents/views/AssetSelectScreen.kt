package com.gemwallet.android.features.asset_select.presents.views

import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.ext.type
import com.gemwallet.android.features.asset_select.viewmodels.BaseAssetSelectViewModel
import com.gemwallet.android.features.asset_select.viewmodels.RecentsSheetViewModel
import com.gemwallet.android.features.asset_select.viewmodels.models.AssetRowSubtitleStyle
import com.gemwallet.android.features.asset_select.viewmodels.models.AssetRowTrailingStyle
import com.gemwallet.android.ui.components.clipboard.clipboardManager
import com.gemwallet.android.ui.components.clipboard.setPlainText
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.assetPriceSupport
import com.gemwallet.android.ui.components.list_item.getBalanceInfo
import com.gemwallet.android.ui.components.screen.SceneTitle
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.compactIconSize
import com.gemwallet.android.ui.theme.iconSize
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetSubtype
import kotlinx.collections.immutable.toImmutableList

@Composable
fun AssetSelectScreen(
    titleContent: (@Composable () -> Unit)? = null,
    closeIcon: Boolean = false,
    onCancel: () -> Unit,
    onSelect: ((AssetId) -> Unit)? = null,
    onSelectRecent: ((AssetId) -> Unit)? = null,
    onAddAsset: (() -> Unit)? = null,
    showFilter: Boolean? = null,
    actions: @Composable RowScope.() -> Unit = {},
    viewModel: BaseAssetSelectViewModel,
    recentsViewModel: RecentsSheetViewModel = hiltViewModel(),
) {
    val flow = viewModel.flowUIModel
    val title = flow.title
    val context = LocalContext.current
    val clipboardManager = LocalContext.current.clipboardManager()
    val support: (AssetInfoDataAggregate) -> (@Composable () -> Unit)? = when (flow.subtitle) {
        AssetRowSubtitleStyle.Network -> { item ->
            if (item.asset.id.type() == AssetSubtype.NATIVE) null else {
                @Composable { ListItemSupportText(item.asset.id.chain.networkName()) }
            }
        }
        AssetRowSubtitleStyle.Price -> { item -> assetPriceSupport(item.price) }
    }
    val itemTrailing: (@Composable (AssetInfoDataAggregate) -> Unit)? = when (flow.trailing) {
        AssetRowTrailingStyle.Balance -> { item -> getBalanceInfo(item)() }
        AssetRowTrailingStyle.Toggle -> { item ->
            Switch(
                checked = item.balanceEnabled,
                onCheckedChange = { viewModel.onChangeVisibility(item.asset.id, it) },
            )
        }
        AssetRowTrailingStyle.Copy -> { item ->
            IconButton(
                onClick = {
                    viewModel.onChangeVisibility(item.asset.id, true)
                    clipboardManager.setPlainText(context, item.accountAddress)
                },
                modifier = Modifier.size(iconSize),
            ) {
                Icon(
                    imageVector = AppIcons.ContentCopyOutlined,
                    contentDescription = "",
                    modifier = Modifier.size(compactIconSize),
                    tint = MaterialTheme.colorScheme.secondary,
                )
            }
        }
        AssetRowTrailingStyle.None -> null
    }
    val uiStates by viewModel.uiState.collectAsStateWithLifecycle()
    val popular by viewModel.popular.collectAsStateWithLifecycle()
    val pinned by viewModel.pinned.collectAsStateWithLifecycle()
    val unpinned by viewModel.unpinned.collectAsStateWithLifecycle()
    val recent by viewModel.recent.collectAsStateWithLifecycle()
    val showsRecents by viewModel.showsRecents.collectAsStateWithLifecycle()
    val showRecents = showsRecents && onSelectRecent != null
    val isAddAvailable by viewModel.isAddAssetAvailable.collectAsStateWithLifecycle()
    val isChainFilterAvailable by viewModel.isChainFilterAvailable.collectAsStateWithLifecycle()
    val availableChains by viewModel.availableChains.collectAsStateWithLifecycle()
    val chainsFilter by viewModel.chainFilter.collectAsStateWithLifecycle()
    val balanceFilter by viewModel.balanceFilter.collectAsStateWithLifecycle()

    val selectAsset: ((Asset) -> Unit)? = onSelect?.let { select ->
        { asset ->
            viewModel.onSelected(asset)
            select(asset.id)
        }
    }

    AssetSelectScene(
        title = titleContent ?: { SceneTitle(title) },
        titleBadge = { item -> if (flow.showsSymbol) getAssetBadge(item) else null },
        closeIcon = closeIcon,
        support = support,
        query = viewModel.queryState,
        pinned = pinned,
        popular = popular,
        unpinned = unpinned,
        recent = if (showRecents) recent else emptyList<Asset>().toImmutableList(),
        state = uiStates,
        isAddAvailable = isAddAvailable && onAddAsset != null,
        availableChains = availableChains,
        chainsFilter = chainsFilter,
        balanceFilter = balanceFilter,
        showFilter = showFilter ?: isChainFilterAvailable,
        showBalanceFilter = flow.showsBalanceFilter,
        onAction = { action ->
            when (action) {
                is AssetSelectAction.ChainFilter -> viewModel.onChainFilter(action.chain)
                is AssetSelectAction.BalanceFilter -> viewModel.onBalanceFilter(action.onlyWithBalance)
                AssetSelectAction.ClearFilters -> viewModel.onClearFilters()
                is AssetSelectAction.Select -> selectAsset?.invoke(action.asset)
                is AssetSelectAction.SelectRecent -> onSelectRecent?.invoke(action.asset.id)
                AssetSelectAction.OpenRecentsSheet -> recentsViewModel.show(filters = viewModel.assetFilters(), types = viewModel.recentTypes)
                AssetSelectAction.Cancel -> onCancel()
                AssetSelectAction.AddAsset -> onAddAsset?.invoke()
                AssetSelectAction.ShowAllAssets -> Unit
            }
        },
        recentsSheetEnabled = showRecents,
        itemTrailing = itemTrailing,
        actions = actions,
    )

    if (showRecents) {
        RecentsSheetHost(viewModel = recentsViewModel, onSelect = { onSelectRecent(it.id) })
    }
}
