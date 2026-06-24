package com.gemwallet.android.features.asset_select.presents.views

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.SearchBar
import com.gemwallet.android.ui.components.TabsBar
import com.gemwallet.android.ui.components.labelRes
import com.gemwallet.android.ui.components.filters.AssetsFilter
import com.gemwallet.android.ui.components.image.AssetIcon
import com.gemwallet.android.ui.components.list_item.AssetContextActions
import com.gemwallet.android.ui.components.list_item.AssetContextMenuRow
import com.gemwallet.android.ui.components.list_item.AssetInfoUIModel
import com.gemwallet.android.ui.components.list_item.AssetItemUIModel
import com.gemwallet.android.ui.components.list_item.AssetListItem
import com.gemwallet.android.ui.components.list_item.PinnedAssetsHeaderItem
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator16
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.AssetsGroupType
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.defaultPadding
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.smallIconSize
import com.gemwallet.android.features.asset_select.viewmodels.models.UIState
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetTag
import com.wallet.core.primitives.Chain
import kotlinx.collections.immutable.ImmutableList
import kotlinx.collections.immutable.toImmutableList
import kotlinx.coroutines.flow.drop

@Composable
fun AssetSelectScene(
    title: String,
    popular: ImmutableList<AssetItemUIModel>,
    pinned: ImmutableList<AssetItemUIModel>,
    unpinned: ImmutableList<AssetItemUIModel>,
    recent: ImmutableList<Asset>,
    state: UIState,
    titleBadge: (AssetItemUIModel) -> String?,
    support: ((AssetItemUIModel) -> (@Composable () -> Unit)?)?,
    query: TextFieldState,
    tags: List<AssetTag?>,
    selectedTag: AssetTag?,
    isAddAvailable: Boolean = false,
    availableChains: List<Chain> = emptyList(),
    chainsFilter: List<Chain> = emptyList(),
    balanceFilter: Boolean = false,
    searchable: Boolean = true,
    onAction: (AssetSelectAction) -> Unit,
    itemTrailing: (@Composable (AssetItemUIModel) -> Unit)? = null,
    actions: @Composable RowScope.() -> Unit = {},
    contextActions: AssetContextActions = AssetContextActions.Empty,
    recentsSheetEnabled: Boolean = false,
) {
    AssetSelectScene(
        title = {
            Text(
                modifier = Modifier,
                text = title,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        },
        popular = popular,
        pinned = pinned,
        unpinned = unpinned,
        recent = recent,
        state = state,
        titleBadge = titleBadge,
        support = support,
        query = query,
        tags = tags,
        selectedTag = selectedTag,
        isAddAvailable = isAddAvailable,
        availableChains = availableChains,
        chainsFilter = chainsFilter,
        balanceFilter = balanceFilter,
        onAction = onAction,
        itemTrailing = itemTrailing,
        actions = actions,
        contextActions = contextActions,
        recentsSheetEnabled = recentsSheetEnabled,
    )
}

@Composable
fun AssetSelectScene(
    title: @Composable () -> Unit,
    popular: ImmutableList<AssetItemUIModel>,
    pinned: ImmutableList<AssetItemUIModel>,
    unpinned: ImmutableList<AssetItemUIModel>,
    recent: ImmutableList<Asset>,
    state: UIState,
    titleBadge: (AssetItemUIModel) -> String?,
    support: ((AssetItemUIModel) -> (@Composable () -> Unit)?)?,
    query: TextFieldState,
    tags: List<AssetTag?>,
    selectedTag: AssetTag?,
    isAddAvailable: Boolean = false,
    availableChains: List<Chain> = emptyList(),
    chainsFilter: List<Chain> = emptyList(),
    balanceFilter: Boolean = false,
    searchable: Boolean = true,
    onAction: (AssetSelectAction) -> Unit,
    itemTrailing: (@Composable (AssetItemUIModel) -> Unit)? = null,
    actions: @Composable RowScope.() -> Unit = {},
    contextActions: AssetContextActions = AssetContextActions.Empty,
    recentsSheetEnabled: Boolean = false,
    pinnedPerpetualRows: List<@Composable (ListPosition) -> Unit> = emptyList(),
    perpetualsContent: (LazyListScope.() -> Unit)? = null,
    assetsHeaderRes: Int? = null,
    assetsHeaderClickable: Boolean = false,
) {
    val onSelect: (AssetId) -> Unit = { onAction(AssetSelectAction.Select(it)) }
    val onSelectRecent: (AssetId) -> Unit = { onAction(AssetSelectAction.SelectRecent(it)) }
    val onOpenRecentsSheet: (() -> Unit)? = if (recentsSheetEnabled) { { onAction(AssetSelectAction.OpenRecentsSheet) } } else null
    val onAssetsHeaderClick: (() -> Unit)? = if (assetsHeaderClickable) { { onAction(AssetSelectAction.ShowAllAssets) } } else null
    val listState = rememberLazyListState()
    var isReturnToTop by remember { mutableStateOf(false) }

    var showSelectNetworks by remember { mutableStateOf(false) }
    val longPressedAsset = remember { mutableStateOf<AssetId?>(null) }
    val showTags = query.text.isEmpty()
    var tagsHeightPx by remember { mutableStateOf(0) }

    LaunchedEffect(Unit) {
        snapshotFlow { query.text.toString() }
            .drop(1)
            .collect { isReturnToTop = it.isEmpty() }
    }

    LaunchedEffect(pinned, unpinned) {
        if (isReturnToTop) {
            if (listState.firstVisibleItemIndex > 0 || listState.firstVisibleItemScrollOffset > 0) {
                listState.animateScrollToItem(0)
            }
            isReturnToTop = false
        }
    }

    Scene(
        titleContent = title,
        actions = {
            if (availableChains.isNotEmpty()) {
                IconButton(onClick = { showSelectNetworks = !showSelectNetworks }) {
                    Icon(
                        imageVector = AppIcons.FilterAlt,
                        tint = if (chainsFilter.isEmpty() && !balanceFilter)
                            LocalContentColor.current
                        else
                            MaterialTheme.colorScheme.primary,
                        contentDescription = null,
                    )
                }
            }
            actions()
        },
        onClose = { onAction(AssetSelectAction.Cancel) }
    ) {
        if (searchable) {
            SearchBar(query = query)
        }
        LazyColumn(
            modifier = Modifier.fillMaxWidth(),
            state = listState,
        ) {
            if (showTags) {
                item {
                    Box(modifier = Modifier.onSizeChanged { tagsHeightPx = it.height }) {
                        TabsBar(
                            tabs = tags,
                            selected = selectedTag,
                            onSelect = { onAction(AssetSelectAction.SelectTag(it)) },
                            scrollable = true,
                            equalWidth = false,
                        ) { item ->
                            Text(
                                stringResource(item.labelRes()),
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                                softWrap = false,
                            )
                        }
                    }
                }
            }
            recent(recent, onSelectRecent, onOpenRecentsSheet)
            assets(popular, AssetsGroupType.Popular, onSelect, support, titleBadge, itemTrailing, longPressedAsset, contextActions)
            if (pinned.isNotEmpty() || pinnedPerpetualRows.isNotEmpty()) {
                item { PinnedAssetsHeaderItem(AssetsGroupType.Pined) }
                val pinnedTotal = pinnedPerpetualRows.size + pinned.size
                itemsPositioned(pinnedPerpetualRows, totalCount = pinnedTotal) { position, row ->
                    row(position)
                }
                assetRows(
                    pinned,
                    onSelect,
                    support,
                    titleBadge,
                    itemTrailing,
                    longPressedAsset,
                    contextActions,
                    indexOffset = pinnedPerpetualRows.size,
                    totalCount = pinnedTotal,
                )
            }
            perpetualsContent?.invoke(this)
            if (assetsHeaderRes != null && unpinned.isNotEmpty()) {
                item {
                    SubheaderItem(assetsHeaderRes, onAssetsHeaderClick)
                }
            }
            assets(unpinned, AssetsGroupType.None, onSelect, support, titleBadge, itemTrailing, longPressedAsset, contextActions)
            loading(state)
            notFound(state = state, onAddAsset = { onAction(AssetSelectAction.AddAsset) }, isAddAvailable = isAddAvailable, topOffset = if (showTags) tagsHeightPx else 0)
        }
    }

    if (showSelectNetworks) {
        AssetsFilter(
            availableChains = availableChains,
            chainFilter = chainsFilter,
            balanceFilter = balanceFilter,
            onDismissRequest = { showSelectNetworks = false },
            onChainFilter = { onAction(AssetSelectAction.ChainFilter(it)) },
            onBalanceFilter = { onAction(AssetSelectAction.BalanceFilter(it)) },
            onClearFilters = { onAction(AssetSelectAction.ClearFilters) }
        )
    }
}

private fun LazyListScope.assets(
    items: List<AssetItemUIModel>,
    group: AssetsGroupType,
    onSelect: ((AssetId) -> Unit)?,
    support: ((AssetItemUIModel) -> (@Composable () -> Unit)?)?,
    titleBadge: (AssetItemUIModel) -> String?,
    itemTrailing: (@Composable (AssetItemUIModel) -> Unit)?,
    longPressedAsset: MutableState<AssetId?>,
    contextActions: AssetContextActions,
) {
    if (items.isEmpty()) return

    item { PinnedAssetsHeaderItem(group) }

    assetRows(items, onSelect, support, titleBadge, itemTrailing, longPressedAsset, contextActions)
}

fun LazyListScope.assetRows(
    items: List<AssetItemUIModel>,
    onSelect: ((AssetId) -> Unit)?,
    support: ((AssetItemUIModel) -> (@Composable () -> Unit)?)?,
    titleBadge: (AssetItemUIModel) -> String?,
    itemTrailing: (@Composable (AssetItemUIModel) -> Unit)?,
    longPressedAsset: MutableState<AssetId?>,
    contextActions: AssetContextActions,
    indexOffset: Int = 0,
    totalCount: Int = items.size,
) {
    itemsPositioned(items, indexOffset = indexOffset, totalCount = totalCount) { position, item ->
        AssetSelectRow(
            position = position,
            item = item,
            support = support,
            titleBadge = titleBadge,
            itemTrailing = itemTrailing,
            longPressedAsset = longPressedAsset,
            onSelect = onSelect,
            contextActions = contextActions,
        )
    }
}

@Composable
fun AssetSelectRow(
    position: ListPosition,
    item: AssetItemUIModel,
    support: ((AssetItemUIModel) -> (@Composable () -> Unit)?)?,
    titleBadge: (AssetItemUIModel) -> String?,
    itemTrailing: (@Composable (AssetItemUIModel) -> Unit)?,
    longPressedAsset: MutableState<AssetId?>,
    onSelect: ((AssetId) -> Unit)?,
    contextActions: AssetContextActions,
) {
    AssetContextMenuRow(
        assetId = item.asset.id,
        address = item.owner,
        isPinned = item.metadata?.isPinned == true,
        isBalanceEnabled = item.metadata?.isBalanceEnabled == true,
        longPressed = longPressedAsset,
        actions = contextActions,
        onClick = { onSelect?.invoke(item.asset.id) },
    ) { rowModifier ->
        AssetListItem(
            modifier = rowModifier,
            listPosition = position,
            asset = item,
            support = support?.invoke(item),
            badge = titleBadge.invoke(item),
            trailing = { itemTrailing?.invoke(item) },
        )
    }
}

private fun LazyListScope.notFound(
    state: UIState,
    isAddAvailable: Boolean = false,
    topOffset: Int = 0,
    onAddAsset: (() -> Unit)? = null,
) {
    if (state !is UIState.Empty) {
        return
    }
    item {
        EmptyContentView(
            type = EmptyContentType.SearchAssets(
                onAddCustomToken = if (isAddAvailable) onAddAsset else null,
            ),
            modifier = Modifier
                .animateItem()
                .fillParentMaxSize()
                .offset { IntOffset(0, -topOffset) },
        )
    }
}

private fun LazyListScope.loading(state: UIState) {
    if (state !is UIState.Loading) {
        return
    }
    item {
        Box(
            modifier = Modifier
                .animateItem()
                .fillMaxWidth()
                .defaultPadding(),
        ) {
            CircularProgressIndicator16(Modifier.align(Alignment.Center))
        }
    }
}

private fun LazyListScope.recent(
    items: List<Asset>,
    onSelect: ((AssetId) -> Unit)?,
    onOpenRecentsSheet: (() -> Unit)? = null,
) {
    if (items.isEmpty()) {
        return
    }
    item {
        SubheaderItem(R.string.recent_activity_title, onOpenRecentsSheet)
    }
    item {
        LazyRow(
            modifier = Modifier.padding(top = paddingHalfSmall, start = paddingDefault, bottom = paddingSmall, end = paddingDefault),
            horizontalArrangement = Arrangement.spacedBy(paddingSmall),
        ) {
            items(items) { asset ->
                Row(
                    modifier = Modifier
                        .clip(RoundedCornerShape(16.dp))
                        .background(MaterialTheme.colorScheme.background)
                        .clickable(onClick = { onSelect?.invoke(asset.id) })
                        .padding(paddingSmall),
                    horizontalArrangement = Arrangement.spacedBy(paddingSmall),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    AssetIcon(asset, size = smallIconSize)
                    Text(asset.symbol)
                }
            }
        }
    }
}

@Composable
@Preview
fun PreviewAssetScreenUI() {
    MaterialTheme {
        AssetSelectScene(
            pinned = emptyList<AssetInfoUIModel>().toImmutableList(),
            unpinned = emptyList<AssetInfoUIModel>().toImmutableList(),
            popular = emptyList<AssetInfoUIModel>().toImmutableList(),
            recent = emptyList<Asset>().toImmutableList(),
            state = UIState.Idle,
            title = "Send",
            titleBadge = { it.asset.symbol },
            support = null,
            tags = AssetTag.entries,
            selectedTag = null,
            query = rememberTextFieldState(),
            onAction = {},
        )
    }
}
