@file:OptIn(ExperimentalMaterial3ExpressiveApi::class)

package com.gemwallet.android.features.assets.views

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.CircularWavyProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
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
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.AppUrl
import com.gemwallet.android.features.assets.viewmodels.AssetsViewModel
import com.gemwallet.android.features.assets.views.components.AssetsHead
import com.gemwallet.android.features.assets.views.components.AssetsListFooter
import com.gemwallet.android.features.assets.views.components.assets
import com.gemwallet.android.features.banner.views.BannersScene
import com.gemwallet.android.features.banner.views.WelcomeBanner
import com.gemwallet.android.features.perpetual.views.PerpetualsPreviewSection
import com.gemwallet.android.features.update_app.presents.InAppUpdateBanner
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.AssetContextActions
import com.gemwallet.android.ui.models.AssetsGroupType
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.BannerEvent
import uniffi.gemstone.DocsUrl

private const val AssetsHeadItemKey = "assets_head"
private const val WelcomeBannerItemKey = "welcome_banner"
private const val InAppUpdateBannerItemKey = "in_app_update_banner"
private const val BannersItemKey = "banners"
private const val ImportingItemKey = "importing"
private const val FooterItemKey = "footer"
private const val PerpetualsSectionItemKey = "perpetuals_section"

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AssetsScreen(
    onAction: (AssetsAction) -> Unit,
    onContentReady: () -> Unit = {},
    listState: LazyListState = rememberLazyListState(),
    viewModel: AssetsViewModel = hiltViewModel(),
) {
    val importing by viewModel.importInProgress.collectAsStateWithLifecycle()
    val pinnedAssets by viewModel.pinnedAssets.collectAsStateWithLifecycle()
    val unpinnedAssets by viewModel.unpinnedAssets.collectAsStateWithLifecycle()
    val walletSummary by viewModel.walletSummary.collectAsStateWithLifecycle()
    val isRefreshing by viewModel.isRefreshing.collectAsStateWithLifecycle()
    val showWelcomeBanner by viewModel.showWelcomeBanner.collectAsStateWithLifecycle()

    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current

    val currentOnContentReady by rememberUpdatedState(onContentReady)
    LaunchedEffect(walletSummary != null) {
        if (walletSummary != null) currentOnContentReady()
    }

    val currentWalletId by viewModel.currentWalletId.collectAsStateWithLifecycle()
    var previousWalletId by rememberSaveable { mutableStateOf<String?>(null) }
    LaunchedEffect(currentWalletId) {
        val walletId = currentWalletId?.id ?: return@LaunchedEffect
        if (previousWalletId != null && previousWalletId != walletId) {
            listState.scrollToItem(0)
        }
        previousWalletId = walletId
    }

    Scaffold(
        modifier = Modifier.fillMaxSize(),
        topBar = { AssetsTopBar(walletSummary, { onAction(AssetsAction.ShowWallets) }, { onAction(AssetsAction.Search) }) },
        containerColor = MaterialTheme.colorScheme.surface,
    ) {
        val pullToRefreshState = rememberPullToRefreshState()
        PullToRefreshBox(
            modifier = Modifier.padding(top = it.calculateTopPadding()),
            isRefreshing = isRefreshing,
            onRefresh = viewModel::onRefresh,
            state = pullToRefreshState,
            indicator = {
                Indicator(
                    modifier = Modifier.align(Alignment.TopCenter),
                    isRefreshing = isRefreshing,
                    state = pullToRefreshState,
                    containerColor = MaterialTheme.colorScheme.background
                )
            }
        ) {
            val longPressedAsset = remember { mutableStateOf<AssetId?>(null) }
            val assetActions = remember(viewModel) {
                AssetContextActions(
                    onTogglePin = viewModel::togglePin,
                    onHide = viewModel::hideAsset,
                )
            }
            LazyColumn(
                modifier = Modifier
                    .fillMaxHeight()
                    .testTag("assets_list"),
                state = listState
            ) {
                item(key = AssetsHeadItemKey) {
                    AssetsHead(
                        walletSummary = walletSummary,
                        onSendClick = { onAction(AssetsAction.Send) },
                        onReceiveClick = { onAction(AssetsAction.Receive) },
                        onBuyClick = { onAction(AssetsAction.Buy) },
                        onSwapClick = { onAction(AssetsAction.Swap) },
                        onHideBalances = viewModel::hideBalances
                    )
                }
                if (showWelcomeBanner) {
                    item(key = WelcomeBannerItemKey) {
                        WelcomeBanner(
                            onBuy = { onAction(AssetsAction.Buy) },
                            onReceive = { onAction(AssetsAction.Receive) },
                            onClose = viewModel::onHideWelcomeBanner
                        )
                    }
                }
                item(key = InAppUpdateBannerItemKey) {
                    InAppUpdateBanner()
                }
                item(key = BannersItemKey) {
                    BannersScene(
                        asset = null,
                        onClick = { banner ->
                            when (banner.event) {
                                BannerEvent.AccountBlockedMultiSignature ->
                                    uriHandler.open(context, AppUrl.docs(DocsUrl.TronMultiSignature))
                                else -> {}
                            }
                        },
                        false
                    )
                }
                if (importing) {
                    item(key = ImportingItemKey) {
                        Row(
                            modifier = Modifier.padding(paddingDefault),
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(paddingSmall)
                        ) {
                            Text(
                                text = "${stringResource(R.string.common_loading)}…"
                            )
                            CircularWavyProgressIndicator(
                                modifier = Modifier.size(paddingDefault),
                                stroke = Stroke(
                                    width = with(LocalDensity.current) { 2.dp.toPx() },
                                    cap = StrokeCap.Round,
                                ),
                                trackStroke = Stroke(
                                    width = with(LocalDensity.current) { 2.dp.toPx() },
                                    cap = StrokeCap.Round,
                                )
                            )
                        }
                    }
                }
                item(key = PerpetualsSectionItemKey) {
                    PerpetualsPreviewSection(
                        onOpenPerpetuals = { onAction(AssetsAction.Perpetuals) },
                        onOpenPerpetualDetails = { onAction(AssetsAction.OpenPerpetualDetails(it)) },
                    )
                }
                assets(
                    items = pinnedAssets,
                    longPressState = longPressedAsset,
                    group = AssetsGroupType.Pined,
                    onAssetClick = { onAction(AssetsAction.OpenAsset(it)) },
                    actions = assetActions,
                )
                assets(
                    items = unpinnedAssets,
                    longPressState = longPressedAsset,
                    group = AssetsGroupType.None,
                    onAssetClick = { onAction(AssetsAction.OpenAsset(it)) },
                    actions = assetActions,
                )
                item(key = FooterItemKey) { AssetsListFooter { onAction(AssetsAction.Manage) } }
            }
        }
    }
}
