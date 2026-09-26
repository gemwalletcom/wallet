package com.gemwallet.android.features.perpetuals.presents.perpetuals

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.clearText
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualDataAggregate
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.perpetuals.presents.components.PerpetualListItem
import com.gemwallet.android.features.perpetuals.presents.components.previewPerpetual
import com.gemwallet.android.features.perpetuals.viewmodels.models.PerpetualPositionRowUIModel
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.SearchBar
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.image.AssetIcon
import com.gemwallet.android.ui.components.list_head.AssetHeadActions
import com.gemwallet.android.ui.components.list_head.ValueListHead
import com.gemwallet.android.ui.components.list_item.AssetListItem
import com.gemwallet.android.ui.components.list_item.PinnedAssetsHeaderItem
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.models.AssetsGroupType
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.smallIconSize
import com.gemwallet.android.ui.theme.space0
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.WalletType
import uniffi.gemstone.GemEmptyStateKind
import uniffi.gemstone.GemHeaderButtonTap
import uniffi.gemstone.GemPerpetualMarketSection
import uniffi.gemstone.GemValueHeader
import uniffi.gemstone.PerpetualBalance
import uniffi.gemstone.perpetualBalanceHeader

@Composable
internal fun PerpetualsScene(
    isRefreshing: Boolean,
    balanceHeader: GemValueHeader?,
    positions: List<PerpetualPositionRowUIModel>,
    unpinnedPerpetuals: List<PerpetualDataAggregate>,
    pinnedPerpetuals: List<PerpetualDataAggregate>,
    recent: List<Asset> = emptyList(),
    query: TextFieldState,
    sections: List<GemPerpetualMarketSection>,
    isSearching: Boolean,
    onAction: (PerpetualsAction) -> Unit,
) {
    val longPressedAsset = remember { mutableStateOf<PerpetualId?>(null) }

    Scene(
        titleContent = {
            if (isSearching) {
                SearchBar(
                    query = query,
                    modifier = Modifier.listItem(com.gemwallet.android.ui.models.ListPosition.Single, paddingHorizontal = space0),
                )
            } else {
                Text(stringResource(R.string.perpetuals_title))
            }
        },
        onClose = {
            if (isSearching) {
                query.clearText()
                onAction(PerpetualsAction.SetSearching(false))
            } else {
                onAction(PerpetualsAction.Close)
            }
        },
        actions = {
            if (!isSearching) {
                IconButton(onClick = { onAction(PerpetualsAction.SetSearching(true)) }) {
                    Icon(imageVector = AppIcons.Search, contentDescription = "search")
                }
            }
        },
    ) {
        PullToRefreshBox(
            isRefreshing = isRefreshing,
            onRefresh = { onAction(PerpetualsAction.Refresh) },
        ) {
            LazyColumn(
                modifier = Modifier.fillMaxSize(),
            ) {
                if (!isSearching && balanceHeader != null) {
                    item {
                        ValueListHead(
                            header = balanceHeader,
                            onClick = { onAction(PerpetualsAction.OpenPortfolio) },
                        ) {
                            AssetHeadActions(balanceHeader.actions ?: return@ValueListHead) { tap ->
                                when (tap) {
                                    is GemHeaderButtonTap.Deposit -> onAction(PerpetualsAction.Deposit(tap.asset.toPrimitives().id))

                                    is GemHeaderButtonTap.Withdraw -> onAction(PerpetualsAction.Withdraw(tap.asset.toPrimitives().id))

                                    is GemHeaderButtonTap.Send, is GemHeaderButtonTap.Receive, is GemHeaderButtonTap.Buy, is GemHeaderButtonTap.Swap,
                                    GemHeaderButtonTap.SendCollectible, GemHeaderButtonTap.CollectibleMenu,
                                    -> Unit
                                }
                            }
                        }
                    }
                }
                sections.forEach { section ->
                    when (section) {
                        GemPerpetualMarketSection.RECENTS -> recentPerpetuals(
                            items = recent,
                            onSeeAll = { onAction(PerpetualsAction.OpenRecentsSheet) },
                            onSelect = { asset -> onAction(PerpetualsAction.OpenRecent(asset)) },
                        )

                        GemPerpetualMarketSection.POSITIONS -> {
                            section.stringRes()?.let { title -> item { SubheaderItem(title) } }
                            itemsPositioned(positions) { position, item ->
                                AssetListItem(
                                    row = item.row,
                                    listPosition = position,
                                    modifier = Modifier.clickable { onAction(PerpetualsAction.OpenPerpetual(item.asset)) },
                                )
                            }
                        }

                        GemPerpetualMarketSection.PINNED -> {
                            item {
                                Spacer16()
                                PinnedAssetsHeaderItem(AssetsGroupType.Pinned)
                            }
                            itemsPositioned(pinnedPerpetuals) { position, item ->
                                PerpetualListItem(
                                    item = item,
                                    listPosition = position,
                                    longPressState = longPressedAsset,
                                    onTogglePin = { onAction(PerpetualsAction.TogglePin(it)) },
                                    onClick = { onAction(PerpetualsAction.OpenPerpetual(item.asset)) },
                                )
                            }
                        }

                        GemPerpetualMarketSection.MARKETS -> {
                            section.stringRes()?.let { title -> item { SubheaderItem(title) } }
                            itemsPositioned(unpinnedPerpetuals) { position, item ->
                                PerpetualListItem(
                                    item = item,
                                    listPosition = position,
                                    longPressState = longPressedAsset,
                                    onTogglePin = { onAction(PerpetualsAction.TogglePin(it)) },
                                    onClick = { onAction(PerpetualsAction.OpenPerpetual(item.asset)) },
                                )
                            }
                        }

                        GemPerpetualMarketSection.EMPTY -> item {
                            EmptyContentView(
                                type = EmptyContentType(GemEmptyStateKind.SEARCH_PERPETUALS),
                                modifier = Modifier
                                    .animateItem()
                                    .fillParentMaxSize(),
                            )
                        }
                    }
                }
            }
        }
    }
}

private fun LazyListScope.recentPerpetuals(items: List<Asset>, onSeeAll: () -> Unit, onSelect: (Asset) -> Unit) {
    if (items.isEmpty()) {
        return
    }
    item { SubheaderItem(R.string.recent_activity_title, onClick = onSeeAll) }
    item {
        LazyRow(
            modifier = Modifier.padding(
                top = paddingHalfSmall,
                start = paddingDefault,
                bottom = paddingSmall,
                end = paddingDefault,
            ),
            horizontalArrangement = Arrangement.spacedBy(paddingSmall),
        ) {
            items(items) { asset ->
                Row(
                    modifier = Modifier
                        .clip(RoundedCornerShape(paddingDefault))
                        .background(MaterialTheme.colorScheme.background)
                        .clickable { onSelect(asset) }
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
fun PreviewPerpetualsScene() {
    WalletTheme {
        PerpetualsScene(
            isRefreshing = false,
            query = androidx.compose.foundation.text.input.TextFieldState(),
            sections = emptyList(),
            isSearching = false,
            balanceHeader = perpetualBalanceHeader(
                PerpetualBalance(available = 45_000.0, reserved = 92_000.0, withdrawable = 42_000.0),
                WalletType.Multicoin.toGem(),
            ),
            positions = emptyList(),
            unpinnedPerpetuals = listOf(
                previewPerpetual(
                    asset = Asset(
                        id = AssetId(Chain.Bitcoin),
                        name = "Bitcoin",
                        symbol = "BTC",
                        decimals = 8,
                        type = AssetType.NATIVE,
                    ),
                    title = "BTC/USD",
                    price = 95420.50,
                    change = 2.5,
                    volume = "15234567890123",
                ),
                previewPerpetual(
                    asset = Asset(
                        id = AssetId(Chain.Ethereum),
                        name = "Ethereum",
                        symbol = "ETH",
                        decimals = 18,
                        type = AssetType.NATIVE,
                    ),
                    title = "ETH/USD",
                    price = 3625.75,
                    change = 1.8,
                    volume = "8456789012345",
                ),
                previewPerpetual(
                    asset = Asset(
                        id = AssetId(Chain.Solana),
                        name = "Solana",
                        symbol = "SOL",
                        decimals = 9,
                        type = AssetType.NATIVE,
                    ),
                    title = "SOL/USD",
                    price = 235.40,
                    change = -0.5,
                    volume = "3123847573745",
                ),
                previewPerpetual(
                    asset = Asset(
                        id = AssetId(Chain.AvalancheC),
                        name = "Avalanche",
                        symbol = "AVAX",
                        decimals = 18,
                        type = AssetType.NATIVE,
                    ),
                    title = "AVAX/USD",
                    price = 41.85,
                    change = 4.1,
                    volume = "1234567890123",
                ),
                previewPerpetual(
                    asset = Asset(
                        id = AssetId(Chain.Ethereum, "0x514910771af9ca656af840dff83e8264ecf986ca"),
                        name = "Chainlink",
                        symbol = "LINK",
                        decimals = 18,
                        type = AssetType.ERC20,
                    ),
                    title = "LINK/USD",
                    price = 21.45,
                    change = 2.7,
                    volume = "987654321098",
                ),
            ),
            pinnedPerpetuals = listOf(
                previewPerpetual(
                    asset = Asset(
                        id = AssetId(Chain.Bitcoin),
                        name = "Bitcoin",
                        symbol = "BTC",
                        decimals = 8,
                        type = AssetType.NATIVE,
                    ),
                    title = "BTC/USD",
                    price = 95420.50,
                    change = 2.5,
                    volume = "15234567890123",
                ),
                previewPerpetual(
                    asset = Asset(
                        id = AssetId(Chain.Ethereum),
                        name = "Ethereum",
                        symbol = "ETH",
                        decimals = 18,
                        type = AssetType.NATIVE,
                    ),
                    title = "ETH/USD",
                    price = 3625.75,
                    change = 1.8,
                    volume = "8456789012345",
                ),
            ),
            onAction = {},
        )
    }
}
