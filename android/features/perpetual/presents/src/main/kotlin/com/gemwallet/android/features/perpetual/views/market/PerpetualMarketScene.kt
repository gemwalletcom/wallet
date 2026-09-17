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
import com.gemwallet.android.domains.perpetual.values.PerpetualBalance
import com.gemwallet.android.domains.price.values.EquivalentValue
import com.gemwallet.android.features.perpetual.viewmodels.model.PerpetualMarketSceneState
import com.gemwallet.android.features.perpetual.viewmodels.models.PerpetualMarketSectionUIModel
import com.gemwallet.android.features.perpetual.viewmodels.models.PerpetualPositionRowUIModel
import com.gemwallet.android.features.perpetual.views.components.MarketHeadActions
import com.gemwallet.android.features.perpetual.views.components.PerpetualItem
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.SearchBar
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.image.AssetIcon
import com.gemwallet.android.ui.components.list_head.AmountListHead
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

@Composable
internal fun PerpetualMarketScene(
    sceneState: PerpetualMarketSceneState,
    balance: PerpetualBalance,
    canWithdraw: Boolean,
    positions: List<PerpetualPositionRowUIModel>,
    unpinnedPerpetuals: List<PerpetualDataAggregate>,
    pinnedPerpetuals: List<PerpetualDataAggregate>,
    recent: List<Asset> = emptyList(),
    query: TextFieldState,
    sections: List<PerpetualMarketSectionUIModel>,
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
            isRefreshing = sceneState.isRefreshing,
            onRefresh = { onAction(PerpetualMarketAction.Refresh) },
        ) {
            LazyColumn(
                modifier = Modifier.fillMaxSize()
            ) {
                if (!isSearching) {
                    item {
                        AmountListHead(
                            amount = balance.total,
                            equivalent = stringResource(
                                R.string.wallet_available_balance,
                                balance.available
                            ),
                            onClick = { onAction(PerpetualMarketAction.OpenPortfolio) },
                        ) {
                            MarketHeadActions(
                                canWithdraw = canWithdraw,
                                onWithdraw = { onAction(PerpetualMarketAction.Withdraw) },
                                onDeposit = { onAction(PerpetualMarketAction.Deposit) },
                            )
                        }
                    }
                }
                sections.forEach { section ->
                    when (section) {
                        PerpetualMarketSectionUIModel.Recents -> recentPerpetuals(
                            items = recent,
                            onSeeAll = { onAction(PerpetualMarketAction.OpenRecentsSheet) },
                            onSelect = { asset -> onAction(PerpetualMarketAction.OpenRecent(asset)) },
                        )
                        is PerpetualMarketSectionUIModel.Positions -> {
                            section.title?.let { title -> item { SubheaderItem(title) } }
                            itemsPositioned(positions) { position, item ->
                                ListItem(
                                    model = item.model,
                                    listPosition = position,
                                    modifier = Modifier.clickable { onAction(PerpetualMarketAction.OpenPerpetual(item.asset)) },
                                )
                            }
                        }
                        PerpetualMarketSectionUIModel.Pinned -> {
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
                        is PerpetualMarketSectionUIModel.Markets -> {
                            section.title?.let { title -> item { SubheaderItem(title) } }
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
                        PerpetualMarketSectionUIModel.Empty -> item {
                            EmptyContentView(
                                type = EmptyContentType.SearchPerpetuals,
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

private fun LazyListScope.recentPerpetuals(
    items: List<Asset>,
    onSeeAll: () -> Unit,
    onSelect: (Asset) -> Unit,
) {
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
            sceneState = PerpetualMarketSceneState.Idle,
            query = androidx.compose.foundation.text.input.TextFieldState(),
            sections = emptyList(),
            isSearching = false,
            canWithdraw = true,
            balance = object : PerpetualBalance {
                override val deposit: String = "$50,000.00"
                override val available: String = "$45,000.00"
                override val withdrawable: String = "$42,000.00"
                override val total: String = "$137,000.00"
            },
            positions = emptyList(),
            unpinnedPerpetuals = listOf(
                object : PerpetualDataAggregate {
                    override val id: PerpetualId = PerpetualId(PerpetualProvider.Hypercore, "BTC")
                    override val title: String = "BTC/USD"
                    override val showsPrice: Boolean = true
                    override val price = object : EquivalentValue {
                        override val currency = Currency.USD
                        override val value: Double = 95420.50
                        override val changePercentage: Double = 2.5
                    }
                    override val volume: String = "15234567890123"
                    override val asset = Asset(
                        id = AssetId(Chain.Bitcoin),
                        name = "Bitcoin",
                        symbol = "BTC",
                        decimals = 8,
                        type = AssetType.NATIVE
                    )
                    override val isPinned: Boolean = false
                },
                object : PerpetualDataAggregate {
                    override val id: PerpetualId = PerpetualId(PerpetualProvider.Hypercore, "ETH")
                    override val title: String = "ETH/USD"
                    override val showsPrice: Boolean = true
                    override val price = object : EquivalentValue {
                        override val currency = Currency.USD
                        override val value: Double = 3625.75
                        override val changePercentage: Double = 1.8
                    }
                    override val volume: String = "8456789012345"
                    override val asset = Asset(
                        id = AssetId(Chain.Ethereum),
                        name = "Ethereum",
                        symbol = "ETH",
                        decimals = 18,
                        type = AssetType.NATIVE
                    )
                    override val isPinned: Boolean = false
                },
                object : PerpetualDataAggregate {
                    override val id: PerpetualId = PerpetualId(PerpetualProvider.Hypercore, "SOL")
                    override val title: String = "SOL/USD"
                    override val showsPrice: Boolean = true
                    override val price = object : EquivalentValue {
                        override val currency = Currency.USD
                        override val value: Double = 235.40
                        override val changePercentage: Double = -0.5
                    }
                    override val volume: String = "3123847573745"
                    override val asset = Asset(
                        id = AssetId(Chain.Solana),
                        name = "Solana",
                        symbol = "SOL",
                        decimals = 9,
                        type = AssetType.NATIVE
                    )
                    override val isPinned: Boolean = false
                },
                object : PerpetualDataAggregate {
                    override val id: PerpetualId = PerpetualId(PerpetualProvider.Hypercore, "AVAX")
                    override val title: String = "AVAX/USD"
                    override val showsPrice: Boolean = true
                    override val price = object : EquivalentValue {
                        override val currency = Currency.USD
                        override val value: Double = 41.85
                        override val changePercentage: Double = 4.1
                    }
                    override val volume: String = "1234567890123"
                    override val asset = Asset(
                        id = AssetId(Chain.AvalancheC),
                        name = "Avalanche",
                        symbol = "AVAX",
                        decimals = 18,
                        type = AssetType.NATIVE
                    )
                    override val isPinned: Boolean = false
                },
                object : PerpetualDataAggregate {
                    override val id: PerpetualId = PerpetualId(PerpetualProvider.Hypercore, "LINK")
                    override val title: String = "LINK/USD"
                    override val showsPrice: Boolean = true
                    override val price = object : EquivalentValue {
                        override val currency = Currency.USD
                        override val value: Double = 21.45
                        override val changePercentage: Double = 2.7
                    }
                    override val volume: String = "987654321098"
                    override val asset = Asset(
                        id = AssetId(Chain.Ethereum, "0x514910771af9ca656af840dff83e8264ecf986ca"),
                        name = "Chainlink",
                        symbol = "LINK",
                        decimals = 18,
                        type = AssetType.ERC20
                    )
                    override val isPinned: Boolean = false
                }
            ),
            pinnedPerpetuals = listOf(
                object : PerpetualDataAggregate {
                    override val id: PerpetualId = PerpetualId(PerpetualProvider.Hypercore, "BTC")
                    override val title: String = "BTC/USD"
                    override val showsPrice: Boolean = true
                    override val price = object : EquivalentValue {
                        override val currency = Currency.USD
                        override val value: Double = 95420.50
                        override val changePercentage: Double = 2.5
                    }
                    override val volume: String = "15234567890123"
                    override val asset = Asset(
                        id = AssetId(Chain.Bitcoin),
                        name = "Bitcoin",
                        symbol = "BTC",
                        decimals = 8,
                        type = AssetType.NATIVE
                    )
                    override val isPinned: Boolean = false
                },
                object : PerpetualDataAggregate {
                    override val id: PerpetualId = PerpetualId(PerpetualProvider.Hypercore, "ETH")
                    override val title: String = "ETH/USD"
                    override val showsPrice: Boolean = true
                    override val price = object : EquivalentValue {
                        override val currency = Currency.USD
                        override val value: Double = 3625.75
                        override val changePercentage: Double = 1.8
                    }
                    override val volume: String = "8456789012345"
                    override val asset = Asset(
                        id = AssetId(Chain.Ethereum),
                        name = "Ethereum",
                        symbol = "ETH",
                        decimals = 18,
                        type = AssetType.NATIVE
                    )
                    override val isPinned: Boolean = false
                },
            ),
            onAction = {},
        )
    }
}
