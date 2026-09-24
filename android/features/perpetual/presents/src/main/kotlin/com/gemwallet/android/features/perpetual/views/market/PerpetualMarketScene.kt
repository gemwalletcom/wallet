package com.gemwallet.android.features.perpetual.views.market

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
import com.gemwallet.android.features.perpetual.viewmodels.localization.stringRes
import com.gemwallet.android.features.perpetual.viewmodels.models.PerpetualPositionRowUIModel
import com.gemwallet.android.features.perpetual.views.components.PerpetualItem
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.SearchBar
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.image.AssetIcon
import com.gemwallet.android.ui.components.list_head.AmountListHead
import com.gemwallet.android.ui.components.list_head.AssetHeadActions
import com.gemwallet.android.ui.components.list_head.uiModel
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.PinnedAssetsHeaderItem
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
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
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualProvider
import com.wallet.core.primitives.WalletType
import uniffi.gemstone.GemCurrencyStyle
import uniffi.gemstone.GemEmptyStateKind
import uniffi.gemstone.GemPerpetualBalanceHeader
import uniffi.gemstone.GemPerpetualMarketSection
import uniffi.gemstone.PerpetualBalance
import uniffi.gemstone.perpetualBalanceHeader
import uniffi.gemstone.priceRow

@Composable
internal fun PerpetualMarketScene(
    isRefreshing: Boolean,
    balanceHeader: GemPerpetualBalanceHeader?,
    positions: List<PerpetualPositionRowUIModel>,
    unpinnedPerpetuals: List<PerpetualDataAggregate>,
    pinnedPerpetuals: List<PerpetualDataAggregate>,
    recent: List<Asset> = emptyList(),
    query: TextFieldState,
    sections: List<GemPerpetualMarketSection>,
    isSearching: Boolean,
    onAction: (PerpetualMarketAction) -> Unit,
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
                onAction(PerpetualMarketAction.SetSearching(false))
            } else {
                onAction(PerpetualMarketAction.Close)
            }
        },
        actions = {
            if (!isSearching) {
                IconButton(onClick = { onAction(PerpetualMarketAction.SetSearching(true)) }) {
                    Icon(imageVector = AppIcons.Search, contentDescription = "search")
                }
            }
        },
    ) {
        PullToRefreshBox(
            isRefreshing = isRefreshing,
            onRefresh = { onAction(PerpetualMarketAction.Refresh) },
        ) {
            LazyColumn(
                modifier = Modifier.fillMaxSize(),
            ) {
                if (!isSearching && balanceHeader != null) {
                    item {
                        AmountListHead(
                            amount = balanceHeader.total.text(),
                            equivalent = stringResource(
                                R.string.wallet_available_balance,
                                balanceHeader.available.text(),
                            ),
                            onClick = { onAction(PerpetualMarketAction.OpenPortfolio) },
                        ) {
                            AssetHeadActions(
                                balanceHeader.actions.uiModel(
                                    onTransfer = null,
                                    onReceive = null,
                                    onBuy = null,
                                    onSwap = null,
                                    onDeposit = { onAction(PerpetualMarketAction.Deposit) },
                                    onWithdraw = { onAction(PerpetualMarketAction.Withdraw) },
                                ),
                            )
                        }
                    }
                }
                sections.forEach { section ->
                    when (section) {
                        GemPerpetualMarketSection.RECENTS -> recentPerpetuals(
                            items = recent,
                            onSeeAll = { onAction(PerpetualMarketAction.OpenRecentsSheet) },
                            onSelect = { asset -> onAction(PerpetualMarketAction.OpenRecent(asset)) },
                        )

                        GemPerpetualMarketSection.POSITIONS -> {
                            section.stringRes()?.let { title -> item { SubheaderItem(title) } }
                            itemsPositioned(positions) { position, item ->
                                ListItem(
                                    model = item.model,
                                    listPosition = position,
                                    modifier = Modifier.clickable { onAction(PerpetualMarketAction.OpenPerpetual(item.asset)) },
                                )
                            }
                        }

                        GemPerpetualMarketSection.PINNED -> {
                            item {
                                Spacer16()
                                PinnedAssetsHeaderItem(AssetsGroupType.Pinned)
                            }
                            itemsPositioned(pinnedPerpetuals) { position, item ->
                                PerpetualItem(
                                    item = item,
                                    listPosition = position,
                                    longPressState = longPressedAsset,
                                    onTogglePin = { onAction(PerpetualMarketAction.TogglePin(it)) },
                                    onClick = { onAction(PerpetualMarketAction.OpenPerpetual(item.asset)) },
                                )
                            }
                        }

                        GemPerpetualMarketSection.MARKETS -> {
                            section.stringRes()?.let { title -> item { SubheaderItem(title) } }
                            itemsPositioned(unpinnedPerpetuals) { position, item ->
                                PerpetualItem(
                                    item = item,
                                    listPosition = position,
                                    longPressState = longPressedAsset,
                                    onTogglePin = { onAction(PerpetualMarketAction.TogglePin(it)) },
                                    onClick = { onAction(PerpetualMarketAction.OpenPerpetual(item.asset)) },
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
fun PreviewPerpetualMarketScene() {
    WalletTheme {
        PerpetualMarketScene(
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
                object : PerpetualDataAggregate {
                    override val id: PerpetualId = PerpetualId(PerpetualProvider.Hypercore, "BTC")
                    override val title: String = "BTC/USD"
                    override val price = priceRow(price = 95420.50, change = 2.5, currency = Currency.USD.toGem(), style = GemCurrencyStyle.SHORT)
                    override val volume: String = "15234567890123"
                    override val asset = Asset(
                        id = AssetId(Chain.Bitcoin),
                        name = "Bitcoin",
                        symbol = "BTC",
                        decimals = 8,
                        type = AssetType.NATIVE,
                    )
                    override val isPinned: Boolean = false
                },
                object : PerpetualDataAggregate {
                    override val id: PerpetualId = PerpetualId(PerpetualProvider.Hypercore, "ETH")
                    override val title: String = "ETH/USD"
                    override val price = priceRow(price = 3625.75, change = 1.8, currency = Currency.USD.toGem(), style = GemCurrencyStyle.SHORT)
                    override val volume: String = "8456789012345"
                    override val asset = Asset(
                        id = AssetId(Chain.Ethereum),
                        name = "Ethereum",
                        symbol = "ETH",
                        decimals = 18,
                        type = AssetType.NATIVE,
                    )
                    override val isPinned: Boolean = false
                },
                object : PerpetualDataAggregate {
                    override val id: PerpetualId = PerpetualId(PerpetualProvider.Hypercore, "SOL")
                    override val title: String = "SOL/USD"
                    override val price = priceRow(price = 235.40, change = -0.5, currency = Currency.USD.toGem(), style = GemCurrencyStyle.SHORT)
                    override val volume: String = "3123847573745"
                    override val asset = Asset(
                        id = AssetId(Chain.Solana),
                        name = "Solana",
                        symbol = "SOL",
                        decimals = 9,
                        type = AssetType.NATIVE,
                    )
                    override val isPinned: Boolean = false
                },
                object : PerpetualDataAggregate {
                    override val id: PerpetualId = PerpetualId(PerpetualProvider.Hypercore, "AVAX")
                    override val title: String = "AVAX/USD"
                    override val price = priceRow(price = 41.85, change = 4.1, currency = Currency.USD.toGem(), style = GemCurrencyStyle.SHORT)
                    override val volume: String = "1234567890123"
                    override val asset = Asset(
                        id = AssetId(Chain.AvalancheC),
                        name = "Avalanche",
                        symbol = "AVAX",
                        decimals = 18,
                        type = AssetType.NATIVE,
                    )
                    override val isPinned: Boolean = false
                },
                object : PerpetualDataAggregate {
                    override val id: PerpetualId = PerpetualId(PerpetualProvider.Hypercore, "LINK")
                    override val title: String = "LINK/USD"
                    override val price = priceRow(price = 21.45, change = 2.7, currency = Currency.USD.toGem(), style = GemCurrencyStyle.SHORT)
                    override val volume: String = "987654321098"
                    override val asset = Asset(
                        id = AssetId(Chain.Ethereum, "0x514910771af9ca656af840dff83e8264ecf986ca"),
                        name = "Chainlink",
                        symbol = "LINK",
                        decimals = 18,
                        type = AssetType.ERC20,
                    )
                    override val isPinned: Boolean = false
                },
            ),
            pinnedPerpetuals = listOf(
                object : PerpetualDataAggregate {
                    override val id: PerpetualId = PerpetualId(PerpetualProvider.Hypercore, "BTC")
                    override val title: String = "BTC/USD"
                    override val price = priceRow(price = 95420.50, change = 2.5, currency = Currency.USD.toGem(), style = GemCurrencyStyle.SHORT)
                    override val volume: String = "15234567890123"
                    override val asset = Asset(
                        id = AssetId(Chain.Bitcoin),
                        name = "Bitcoin",
                        symbol = "BTC",
                        decimals = 8,
                        type = AssetType.NATIVE,
                    )
                    override val isPinned: Boolean = false
                },
                object : PerpetualDataAggregate {
                    override val id: PerpetualId = PerpetualId(PerpetualProvider.Hypercore, "ETH")
                    override val title: String = "ETH/USD"
                    override val price = priceRow(price = 3625.75, change = 1.8, currency = Currency.USD.toGem(), style = GemCurrencyStyle.SHORT)
                    override val volume: String = "8456789012345"
                    override val asset = Asset(
                        id = AssetId(Chain.Ethereum),
                        name = "Ethereum",
                        symbol = "ETH",
                        decimals = 18,
                        type = AssetType.NATIVE,
                    )
                    override val isPinned: Boolean = false
                },
            ),
            onAction = {},
        )
    }
}
