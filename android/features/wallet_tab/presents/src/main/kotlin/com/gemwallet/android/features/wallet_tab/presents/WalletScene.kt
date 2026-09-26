@file:OptIn(ExperimentalMaterial3ExpressiveApi::class)

package com.gemwallet.android.features.wallet_tab.presents

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.material3.CircularWavyProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.wallet.aggregates.WalletSummary
import com.gemwallet.android.features.assets.presents.banner.Banner
import com.gemwallet.android.features.nft.presents.CollectionsPreviewAction
import com.gemwallet.android.features.nft.presents.CollectionsPreviewSection
import com.gemwallet.android.features.perpetuals.presents.PerpetualsPreviewSection
import com.gemwallet.android.features.wallet_tab.presents.components.AssetsListFooter
import com.gemwallet.android.features.wallet_tab.presents.components.WalletHeader
import com.gemwallet.android.features.wallet_tab.presents.components.assets
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.AssetContextActions
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.SnackbarHost
import com.gemwallet.android.ui.models.AssetsGroupType
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.space2
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemBannerDestination
import uniffi.gemstone.GemBannerKey
import uniffi.gemstone.GemBannerRow

private const val WalletHeaderItemKey = "assets_head"
private const val InAppUpdateBannerItemKey = "in_app_update_banner"
private const val BannersItemKey = "banners"
private const val ImportingItemKey = "importing"
private const val FooterItemKey = "footer"
private const val PerpetualsSectionItemKey = "perpetuals_section"
private const val CollectionsSectionItemKey = "collections_section"
private const val AssetsListTag = "assets_list"

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun WalletScene(
    walletSummary: WalletSummary?,
    importing: Boolean,
    pinnedAssets: List<AssetInfoDataAggregate>,
    unpinnedAssets: List<AssetInfoDataAggregate>,
    bannerRow: GemBannerRow?,
    isRefreshing: Boolean,
    collectionsAvailable: Boolean,
    snackbar: SnackbarHostState,
    listState: LazyListState,
    assetActions: AssetContextActions,
    onRefresh: () -> Unit,
    onHideBalances: () -> Unit,
    onCloseBanner: (GemBannerKey) -> Unit,
    onAction: (WalletAction) -> Unit,
) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current

    Scaffold(
        modifier = Modifier.fillMaxSize(),
        topBar = {
            WalletTopBar(
                walletSummary = walletSummary,
                onShowWallets = { onAction(WalletAction.ShowWallets) },
                onSearch = { onAction(WalletAction.Search) },
                onScan = { onAction(WalletAction.Scan) },
            )
        },
        snackbarHost = { SnackbarHost(snackbar) },
        contentWindowInsets = WindowInsets(0, 0, 0, 0),
        containerColor = MaterialTheme.colorScheme.surface,
    ) {
        PullToRefreshBox(
            modifier = Modifier.padding(top = it.calculateTopPadding()),
            isRefreshing = isRefreshing,
            onRefresh = onRefresh,
        ) {
            val longPressedAsset = remember { mutableStateOf<AssetId?>(null) }
            LazyColumn(
                modifier = Modifier
                    .fillMaxHeight()
                    .testTag(AssetsListTag),
                state = listState,
            ) {
                item(key = WalletHeaderItemKey) {
                    WalletHeader(
                        walletSummary = walletSummary,
                        onSendClick = { onAction(WalletAction.Send) },
                        onReceiveClick = { onAction(WalletAction.Receive) },
                        onBuyClick = { onAction(WalletAction.Buy) },
                        onSwapClick = { onAction(WalletAction.Swap) },
                        onPortfolio = { onAction(WalletAction.Portfolio) },
                        onHideBalances = onHideBalances,
                    )
                }
                item(key = InAppUpdateBannerItemKey) {
                    InAppUpdateBanner()
                }
                bannerRow?.let { banner ->
                    item(key = BannersItemKey) {
                        Banner(
                            banner = banner,
                            onSelect = { destination ->
                                when (destination) {
                                    is GemBannerDestination.Url -> uriHandler.open(context, destination.url)

                                    GemBannerDestination.Stake,
                                    GemBannerDestination.Perpetuals,
                                    is GemBannerDestination.ActivateAsset,
                                    -> Unit
                                }
                            },
                            onClose = onCloseBanner,
                            onBuy = { onAction(WalletAction.Buy) },
                            onReceive = { onAction(WalletAction.Receive) },
                        )
                    }
                }
                if (importing) {
                    item(key = ImportingItemKey) {
                        Row(
                            modifier = Modifier.padding(paddingDefault),
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(paddingSmall),
                        ) {
                            Text(
                                text = "${stringResource(R.string.common_loading)}…",
                            )
                            CircularWavyProgressIndicator(
                                modifier = Modifier.size(paddingDefault),
                                stroke = Stroke(
                                    width = with(LocalDensity.current) { space2.toPx() },
                                    cap = StrokeCap.Round,
                                ),
                                trackStroke = Stroke(
                                    width = with(LocalDensity.current) { space2.toPx() },
                                    cap = StrokeCap.Round,
                                ),
                            )
                        }
                    }
                }
                if (walletSummary?.state?.showsPerpetuals == true) {
                    item(key = PerpetualsSectionItemKey) {
                        PerpetualsPreviewSection(
                            onOpenPerpetuals = { onAction(WalletAction.Perpetuals) },
                            onOpenPerpetual = { onAction(WalletAction.OpenPerpetual(it)) },
                        )
                    }
                }
                assets(
                    items = pinnedAssets,
                    longPressState = longPressedAsset,
                    group = AssetsGroupType.Pinned,
                    onAssetClick = { onAction(WalletAction.OpenAsset(it)) },
                    actions = assetActions,
                )
                assets(
                    items = unpinnedAssets,
                    longPressState = longPressedAsset,
                    group = AssetsGroupType.None,
                    onAssetClick = { onAction(WalletAction.OpenAsset(it)) },
                    actions = assetActions,
                )
                if (collectionsAvailable) {
                    item(key = CollectionsSectionItemKey) {
                        CollectionsPreviewSection(
                            onAction = { action ->
                                when (action) {
                                    CollectionsPreviewAction.OpenCollections -> onAction(WalletAction.OpenCollections)
                                    is CollectionsPreviewAction.OpenCollection -> onAction(WalletAction.OpenNftCollection(action.collectionId))
                                    is CollectionsPreviewAction.OpenAsset -> onAction(WalletAction.OpenNftAsset(action.assetId))
                                }
                            },
                        )
                    }
                }
                item(key = FooterItemKey) { AssetsListFooter { onAction(WalletAction.Manage) } }
            }
        }
    }
}
