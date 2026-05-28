package com.gemwallet.android.features.main.views

import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedContent
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material3.Badge
import androidx.compose.material3.BadgedBox
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationBarItemDefaults
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveableStateHolder
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.activities.presents.list.TransactionsNavScreen
import com.gemwallet.android.features.assets.viewmodels.AssetsViewModel
import com.gemwallet.android.features.assets.views.AssetsScreen
import com.gemwallet.android.features.main.models.BottomNavItem
import com.gemwallet.android.features.main.viewmodels.MainScreenViewModel
import com.gemwallet.android.features.nft.presents.NftListNavScreen
import com.gemwallet.android.features.settings.settings.presents.views.SettingsScene
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.animation.NavigationAnimation
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.navigation.WalletNavigator
import com.gemwallet.android.ui.navigation.WalletRootRoute
import com.gemwallet.android.ui.navigation.routes.assetsRoute
import com.gemwallet.android.ui.navigation.routes.nftRoute
import com.gemwallet.android.ui.navigation.routes.settingsRoute
import com.gemwallet.android.ui.navigation.routes.transactionsRoute
import com.gemwallet.android.ui.theme.alpha10
import kotlinx.coroutines.launch

@Composable
fun MainScreen(
    navigator: WalletNavigator,
    currentTab: MutableState<String>,
    onWalletContentReady: () -> Unit = {},
    viewModel: MainScreenViewModel = hiltViewModel()
) {
    val pendingCount by viewModel.pendingTxCount.collectAsStateWithLifecycle()
    val collectionsAvailable by viewModel.collectionsAvailable.collectAsStateWithLifecycle()
    val assetsViewModel: AssetsViewModel = hiltViewModel()
    val isRootRouteActive = navigator.backStack.lastOrNull() == WalletRootRoute

    BackHandler(isRootRouteActive && currentTab.value != assetsRoute) {
        currentTab.value = assetsRoute
    }
    val assetsListState = rememberLazyListState()
    val activitiesListState = rememberLazyListState()
    val nftListState = rememberLazyGridState()
    val settingsScrollState = rememberScrollState()
    val tabStateHolder = rememberSaveableStateHolder()
    val coroutineScope = rememberCoroutineScope()
    val scrollTabToTop: (String) -> Unit = { route ->
        coroutineScope.launch {
            when (route) {
                assetsRoute -> assetsListState.animateScrollToItem(0)
                transactionsRoute -> activitiesListState.animateScrollToItem(0)
                nftRoute -> nftListState.animateScrollToItem(0)
                settingsRoute -> settingsScrollState.animateScrollTo(0)
            }
        }
    }

    LaunchedEffect(collectionsAvailable, currentTab.value) {
        if (!collectionsAvailable && currentTab.value == nftRoute) {
            currentTab.value = assetsRoute
        }
    }

    val navItems = listOfNotNull(
        BottomNavItem(
            label = stringResource(R.string.common_wallet),
            icon = AppIcons.Wallet,
            route = assetsRoute,
            testTag = "mainTab",
        ),
        if (collectionsAvailable) BottomNavItem(
            label = stringResource(R.string.nft_collections),
            icon = AppIcons.EmojiEvents,
            route = nftRoute,
            testTag = "nftTab",
        ) else null,
        BottomNavItem(
            label = stringResource(R.string.activity_title),
            icon = AppIcons.ElectricBolt,
            route = transactionsRoute,
            badge = pendingCount,
            testTag = "activitiesTab",
        ),
        BottomNavItem(
            label = stringResource(R.string.settings_title),
            icon = AppIcons.Settings,
            route = settingsRoute,
            testTag = "settingsTab",
        ),
    )
    Scaffold(
        containerColor = MaterialTheme.colorScheme.surface,
        bottomBar = {
            Column {
                HorizontalDivider(thickness = 0.5.dp)
                NavigationBar(
                    containerColor = MaterialTheme.colorScheme.surface,
                    contentColor = MaterialTheme.colorScheme.primary,
                ) {
                    navItems.forEach { item ->
                        NavigationBarItem(
                            modifier = Modifier.testTag(item.testTag),
                            selected = item.route == currentTab.value,
                            onClick = {
                                if (item.route == currentTab.value) {
                                    scrollTabToTop(item.route)
                                } else {
                                    currentTab.value = item.route
                                }
                            },
                            icon = {
                                val modifier = Modifier.size(24.dp)
                                if (item.route == assetsRoute) {
                                    Icon(
                                        modifier = modifier,
                                        painter = painterResource(R.drawable.wallets),
                                        contentDescription = item.label,
                                    )
                                } else {
                                    BadgedBox(
                                        badge = {
                                            if (!item.badge.isNullOrEmpty()) {
                                                Badge(
                                                    modifier = Modifier.offset(x = 6.dp, y = 0.dp)
                                                ) {
                                                    Text(text = item.badge)
                                                }
                                            }
                                        }
                                    ) {
                                        Icon(
                                            modifier = modifier,
                                            imageVector = item.icon,
                                            contentDescription = item.label,
                                        )

                                    }
                                }
                            },
                            label = { Text(item.label, maxLines = 1, overflow = TextOverflow.MiddleEllipsis) },
                            colors = NavigationBarItemDefaults.colors().copy(
                                selectedIconColor = MaterialTheme.colorScheme.primary,
                                selectedTextColor = MaterialTheme.colorScheme.onSurface,
                                unselectedIconColor = MaterialTheme.colorScheme.secondary,
                                unselectedTextColor = MaterialTheme.colorScheme.onSurface,
                                selectedIndicatorColor = MaterialTheme.colorScheme.primary.copy(alpha = alpha10),
                            )
                        )
                    }
                }
            }
        }
    ) {
        Box(modifier = Modifier.padding(bottom = it.calculateBottomPadding())) {
            AnimatedContent(
                targetState = currentTab.value,
                transitionSpec = { NavigationAnimation.tabContentTransition() },
                label = "MainTabContent",
            ) { tab ->
                tabStateHolder.SaveableStateProvider(tab) {
                    when (tab) {
                        assetsRoute -> AssetsScreen(
                            onShowWallets = navigator::openWallets,
                            onManage = navigator::openAssetsManage,
                            onSearch = navigator::openAssetsSearch,
                            onSendClick = navigator::openRecipient,
                            onReceiveClick = navigator::openReceive,
                            onBuyClick = navigator::openBuy,
                            onSwapClick = navigator::openSwap,
                            onPerpetuals = navigator::openPerpetuals,
                            onPerpetualDetails = navigator::openPerpetualDetails,
                            onAssetClick = navigator::openAsset,
                            onContentReady = onWalletContentReady,
                            listState = assetsListState,
                            viewModel = assetsViewModel,
                        )
                        transactionsRoute -> TransactionsNavScreen(
                            listState = activitiesListState,
                            onTransaction = navigator::openTransaction,
                            onBuy = navigator::openBuy,
                            onReceive = navigator::openReceive,
                        )
                        nftRoute -> NftListNavScreen(
                            listState = nftListState,
                            cancelAction = null,
                            collectionAction = navigator::openNftCollection,
                            assetAction = navigator::openNftAsset,
                            onReceive = navigator::openReceiveNftChains,
                            onUnverifiedClick = navigator::openNftUnverifiedCollections,
                        )
                        else -> SettingsScene(
                            scrollState = settingsScrollState,
                            onSecurity = navigator::openSecurity,
                            onBridges = navigator::openBridgeConnections,
                            onDevelop = navigator::openDevelop,
                            onWallets = navigator::openWallets,
                            onNotifications = navigator::openNotifications,
                            onSupport = navigator::openSupport,
                            onAboutUs = navigator::openAboutUs,
                            onReferral = { navigator.openReferral() },
                            onPreferences = navigator::openPreferences,
                        )
                    }
                }
            }
        }
    }
}
