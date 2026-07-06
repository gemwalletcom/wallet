@file:OptIn(ExperimentalMaterial3Api::class)

package com.gemwallet.android.features.assets.views

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.material3.pulltorefresh.PullToRefreshDefaults.Indicator
import androidx.compose.material3.pulltorefresh.rememberPullToRefreshState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.AppUrl
import com.gemwallet.android.domains.wallet.aggregates.WalletSummaryAggregate
import com.gemwallet.android.features.assets.viewmodels.AssetsViewModel
import com.gemwallet.android.features.assets.viewmodels.WalletContentType
import com.gemwallet.android.features.assets.viewmodels.WalletHeaderViewModel
import com.gemwallet.android.features.assets.views.components.AssetsHead
import com.gemwallet.android.features.nft.presents.NftListAction
import com.gemwallet.android.features.nft.presents.walletNftItems
import com.gemwallet.android.features.nft.viewmodels.NftListViewModels
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.TabsBar
import com.gemwallet.android.ui.components.TabsWidth
import com.gemwallet.android.ui.components.list_item.AssetContextActions
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.BannerEvent
import uniffi.gemstone.DocsUrl

@Composable
fun WalletScreen(
    onAction: (WalletAction) -> Unit,
    onContentReady: () -> Unit = {},
    listState: LazyListState = rememberLazyListState(),
    headerViewModel: WalletHeaderViewModel = hiltViewModel(),
    assetsViewModel: AssetsViewModel = hiltViewModel(),
    nftViewModel: NftListViewModels = hiltViewModel(),
) {
    val walletSummary by headerViewModel.walletSummary.collectAsStateWithLifecycle()
    val availableContentTypes by headerViewModel.availableContentTypes.collectAsStateWithLifecycle()

    val importing by assetsViewModel.importInProgress.collectAsStateWithLifecycle()
    val pinnedAssets by assetsViewModel.pinnedAssets.collectAsStateWithLifecycle()
    val unpinnedAssets by assetsViewModel.unpinnedAssets.collectAsStateWithLifecycle()
    val assetsRefreshing by assetsViewModel.isRefreshing.collectAsStateWithLifecycle()
    val showWelcomeBanner by assetsViewModel.showWelcomeBanner.collectAsStateWithLifecycle()
    val currentWalletId by assetsViewModel.currentWalletId.collectAsStateWithLifecycle()

    val nftItems by nftViewModel.collections.collectAsStateWithLifecycle()
    val nftRefreshing by nftViewModel.isRefreshing.collectAsStateWithLifecycle()
    val nftError by nftViewModel.error.collectAsStateWithLifecycle()
    val unverifiedCount by nftViewModel.unverifiedCount.collectAsStateWithLifecycle()
    val nftWalletId by nftViewModel.walletId.collectAsStateWithLifecycle()

    var selected by rememberSaveable { mutableStateOf(WalletContentType.Assets) }
    LaunchedEffect(availableContentTypes) {
        if (selected !in availableContentTypes) {
            selected = WalletContentType.Assets
        }
    }
    LaunchedEffect(selected) {
        listState.scrollToItem(0)
    }
    LaunchedEffect(nftWalletId) {
        nftViewModel.syncIfNeeded()
    }

    var previousWalletId by rememberSaveable { mutableStateOf<String?>(null) }
    val walletId = currentWalletId?.id
    val walletChanged = walletId != null && previousWalletId != null && previousWalletId != walletId
    if (walletChanged) {
        selected = WalletContentType.Assets
    }
    LaunchedEffect(walletId) {
        if (walletChanged) {
            listState.scrollToItem(0)
        }
        if (walletId != null) previousWalletId = walletId
    }

    val currentOnContentReady by rememberUpdatedState(onContentReady)
    LaunchedEffect(walletSummary != null) {
        if (walletSummary != null) currentOnContentReady()
    }

    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current
    val longPressedAsset = remember { mutableStateOf<AssetId?>(null) }
    val assetActions = remember(assetsViewModel) {
        AssetContextActions(
            onTogglePin = assetsViewModel::togglePin,
            onHide = assetsViewModel::hideAsset,
        )
    }

    val isRefreshing = when (selected) {
        WalletContentType.Assets -> assetsRefreshing
        WalletContentType.Collections -> nftRefreshing
        WalletContentType.Defi -> false
    }
    val onRefresh: () -> Unit = {
        when (selected) {
            WalletContentType.Assets -> assetsViewModel.onRefresh()
            WalletContentType.Collections -> nftViewModel.refresh()
            WalletContentType.Defi -> Unit
        }
    }

    Scaffold(
        modifier = Modifier.fillMaxSize(),
        topBar = {
            AssetsTopBar(
                walletSummary = walletSummary,
                onShowWallets = { onAction(WalletAction.ShowWallets) },
                onSearch = { onAction(WalletAction.Search) },
            )
        },
        containerColor = MaterialTheme.colorScheme.surface,
    ) { padding ->
        val pullToRefreshState = rememberPullToRefreshState()
        PullToRefreshBox(
            modifier = Modifier.padding(top = padding.calculateTopPadding()),
            isRefreshing = isRefreshing,
            onRefresh = onRefresh,
            state = pullToRefreshState,
            indicator = {
                Indicator(
                    modifier = Modifier.align(Alignment.TopCenter),
                    isRefreshing = isRefreshing,
                    state = pullToRefreshState,
                    containerColor = MaterialTheme.colorScheme.background,
                )
            },
        ) {
            LazyColumn(
                modifier = Modifier
                    .fillMaxSize()
                    .testTag("wallet_content"),
                state = listState,
            ) {
                val header: @Composable () -> Unit = {
                    WalletHeader(
                        walletSummary = walletSummary,
                        availableContentTypes = availableContentTypes,
                        selected = selected,
                        onSelect = { selected = it },
                        onAction = onAction,
                        onHideBalances = headerViewModel::hideBalances,
                    )
                }
                when (selected) {
                    WalletContentType.Assets -> {
                        item { header() }
                        walletAssetsItems(
                            importing = importing,
                            showWelcomeBanner = showWelcomeBanner,
                            pinnedAssets = pinnedAssets,
                            unpinnedAssets = unpinnedAssets,
                            longPressState = longPressedAsset,
                            assetActions = assetActions,
                            onAction = onAction,
                            onBanner = { banner ->
                                when (banner.event) {
                                    BannerEvent.AccountBlockedMultiSignature ->
                                        uriHandler.open(context, AppUrl.docs(DocsUrl.TronMultiSignature))
                                    else -> {}
                                }
                            },
                            onCloseWelcome = assetsViewModel::onHideWelcomeBanner,
                        )
                    }
                    WalletContentType.Collections -> walletNftItems(
                        items = nftItems,
                        error = nftError,
                        unverifiedCount = unverifiedCount,
                        header = header,
                        onAction = { action ->
                            when (action) {
                                is NftListAction.OpenCollection -> onAction(WalletAction.OpenNftCollection(action.collectionId))
                                is NftListAction.OpenAsset -> onAction(WalletAction.OpenNftAsset(action.assetId))
                                NftListAction.OpenUnverified -> onAction(WalletAction.NftUnverified)
                                NftListAction.Receive -> onAction(WalletAction.NftReceive)
                                NftListAction.Refresh -> nftViewModel.refresh()
                                NftListAction.Close -> Unit
                            }
                        },
                    )
                    WalletContentType.Defi -> item {
                        Column(modifier = Modifier.fillParentMaxSize()) {
                            header()
                            Box(
                                modifier = Modifier
                                    .weight(1f)
                                    .fillMaxWidth(),
                                contentAlignment = Alignment.Center,
                            ) {
                                WalletDefiSection()
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun WalletHeader(
    walletSummary: WalletSummaryAggregate?,
    availableContentTypes: List<WalletContentType>,
    selected: WalletContentType,
    onSelect: (WalletContentType) -> Unit,
    onAction: (WalletAction) -> Unit,
    onHideBalances: () -> Unit,
) {
    Column {
        AssetsHead(
            walletSummary = walletSummary,
            onSendClick = { onAction(WalletAction.Send) },
            onReceiveClick = {
                onAction(if (selected == WalletContentType.Collections) WalletAction.NftReceive else WalletAction.Receive)
            },
            onBuyClick = { onAction(WalletAction.Buy) },
            onSwapClick = { onAction(WalletAction.Swap) },
            onHideBalances = onHideBalances,
        )
        if (availableContentTypes.size > 1) {
            Box(modifier = Modifier.padding(bottom = paddingDefault)) {
                TabsBar(
                    tabs = availableContentTypes,
                    selected = selected,
                    onSelect = onSelect,
                    width = TabsWidth.Fill,
                ) { type ->
                    Text(contentTypeTitle(type))
                }
            }
        }
    }
}

@Composable
private fun contentTypeTitle(type: WalletContentType): String = when (type) {
    WalletContentType.Assets -> stringResource(R.string.assets_title)
    WalletContentType.Collections -> stringResource(R.string.nft_collections)
    WalletContentType.Defi -> "DeFi"
}
