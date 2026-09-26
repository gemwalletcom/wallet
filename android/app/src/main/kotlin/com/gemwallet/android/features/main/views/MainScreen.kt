package com.gemwallet.android.features.main.views

import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedContent
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.consumeWindowInsets
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
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
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.saveable.rememberSaveableStateHolder
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.MessageToast
import com.gemwallet.android.features.main.models.BottomNavItem
import com.gemwallet.android.features.main.viewmodels.MainScreenViewModel
import com.gemwallet.android.features.settings.presents.SettingsScreen
import com.gemwallet.android.features.transactions.presents.list.TransactionsScreen
import com.gemwallet.android.features.wallet_tab.presents.WalletAction
import com.gemwallet.android.features.wallet_tab.presents.WalletScreen
import com.gemwallet.android.features.wallet_tab.viewmodels.WalletViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.ConnectionStatusBannerHost
import com.gemwallet.android.ui.components.LocalConnectionBannerHandled
import com.gemwallet.android.ui.components.animation.NavigationAnimation
import com.gemwallet.android.ui.components.screen.text
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.actions.SettingsAction
import com.gemwallet.android.ui.navigation.WalletNavigator
import com.gemwallet.android.ui.navigation.WalletRootRoute
import com.gemwallet.android.ui.navigation.routes.SettingsRoute
import com.gemwallet.android.ui.navigation.routes.TransactionsRoute
import com.gemwallet.android.ui.navigation.routes.WalletRoute
import com.gemwallet.android.ui.theme.alpha10
import com.gemwallet.android.ui.theme.hairlineThickness
import com.gemwallet.android.ui.theme.smallIconSize
import com.gemwallet.android.ui.theme.space0
import com.gemwallet.android.ui.theme.space6
import kotlinx.coroutines.launch

@Composable
fun MainScreen(navigator: WalletNavigator, currentTab: MutableState<String>, onWalletContentReady: () -> Unit = {}, viewModel: MainScreenViewModel = hiltViewModel()) {
    val pendingCount by viewModel.pendingTxCount.collectAsStateWithLifecycle()
    val walletViewModel: WalletViewModel = hiltViewModel()
    val isRootRouteActive = navigator.backStack.lastOrNull() == WalletRootRoute
    var isPresentingScanner by rememberSaveable { mutableStateOf(false) }

    ScanReceiveModal(
        isVisible = isPresentingScanner,
        onDismissRequest = { isPresentingScanner = false },
        onScan = { code ->
            isPresentingScanner = false
            viewModel.onScan(code)
        },
    )

    MessageToast(
        message = navigator.routeMessage(WalletRootRoute)?.text(LocalContext.current),
        onShown = { navigator.clearRouteMessage(WalletRootRoute) },
    )

    BackHandler(isRootRouteActive && currentTab.value != WalletRoute) {
        currentTab.value = WalletRoute
    }
    val assetsListState = rememberLazyListState()
    val activitiesListState = rememberLazyListState()
    val settingsScrollState = rememberScrollState()
    val tabStateHolder = rememberSaveableStateHolder()
    val coroutineScope = rememberCoroutineScope()
    val scrollTabToTop: (String) -> Unit = { route ->
        coroutineScope.launch {
            when (route) {
                WalletRoute -> assetsListState.animateScrollToItem(0)
                TransactionsRoute -> activitiesListState.animateScrollToItem(0)
                SettingsRoute -> settingsScrollState.animateScrollTo(0)
            }
        }
    }

    val navItems = listOfNotNull(
        BottomNavItem(
            label = stringResource(R.string.common_wallet),
            icon = AppIcons.Wallet,
            route = WalletRoute,
            testTag = "mainTab",
        ),
        BottomNavItem(
            label = stringResource(R.string.activity_title),
            icon = AppIcons.ElectricBolt,
            route = TransactionsRoute,
            badge = pendingCount,
            testTag = "activitiesTab",
        ),
        BottomNavItem(
            label = stringResource(R.string.settings_title),
            icon = AppIcons.Settings,
            route = SettingsRoute,
            testTag = "settingsTab",
        ),
    )
    Scaffold(
        containerColor = MaterialTheme.colorScheme.surface,
        bottomBar = {
            Column {
                HorizontalDivider(thickness = hairlineThickness)
                ConnectionStatusBannerHost()
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
                                val modifier = Modifier.size(smallIconSize)
                                if (item.route == WalletRoute) {
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
                                                    modifier = Modifier.offset(x = space6, y = space0),
                                                ) {
                                                    Text(text = item.badge)
                                                }
                                            }
                                        },
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
                            ),
                        )
                    }
                }
            }
        },
    ) {
        val bottomPadding = it.calculateBottomPadding()
        Box(
            modifier = Modifier
                .padding(bottom = bottomPadding)
                .consumeWindowInsets(PaddingValues(bottom = bottomPadding)),
        ) {
            CompositionLocalProvider(LocalConnectionBannerHandled provides true) {
                AnimatedContent(
                    targetState = currentTab.value,
                    transitionSpec = { NavigationAnimation.tabContentTransition() },
                    label = "MainTabContent",
                ) { tab ->
                    tabStateHolder.SaveableStateProvider(tab) {
                        when (tab) {
                            WalletRoute -> WalletScreen(
                                onAction = { action ->
                                    when (action) {
                                        WalletAction.ShowWallets -> navigator.openWallets()
                                        WalletAction.Manage -> navigator.openAssetsManage()
                                        WalletAction.Scan -> isPresentingScanner = true
                                        WalletAction.Search -> navigator.openAssetsSearch()
                                        WalletAction.Send -> navigator.openRecipient()
                                        WalletAction.Receive -> navigator.openReceive()
                                        WalletAction.Buy -> navigator.openBuy()
                                        WalletAction.Swap -> navigator.openSwap()
                                        WalletAction.Portfolio -> navigator.openPortfolio()
                                        WalletAction.Perpetuals -> navigator.openPerpetuals()
                                        is WalletAction.OpenPerpetual -> navigator.openPerpetual(action.assetId)
                                        is WalletAction.OpenAsset -> navigator.openAsset(action.assetId)
                                        WalletAction.OpenCollections -> navigator.openCollections()
                                        is WalletAction.OpenNftCollection -> navigator.openCollection(action.collectionId)
                                        is WalletAction.OpenNftAsset -> navigator.openCollectible(action.assetId)
                                    }
                                },
                                onContentReady = onWalletContentReady,
                                listState = assetsListState,
                                viewModel = walletViewModel,
                            )

                            TransactionsRoute -> TransactionsScreen(
                                listState = activitiesListState,
                                onTransaction = navigator::openTransaction,
                                onBuy = navigator::openBuy,
                                onReceive = navigator::openReceive,
                            )

                            else -> SettingsScreen(
                                scrollState = settingsScrollState,
                                onAction = { action ->
                                    when (action) {
                                        SettingsAction.Wallets -> navigator.openWallets()
                                        SettingsAction.Security -> navigator.openSecurity()
                                        SettingsAction.Notifications -> navigator.openNotifications()
                                        SettingsAction.Preferences -> navigator.openPreferences()
                                        SettingsAction.Connections -> navigator.openConnections()
                                        SettingsAction.Support -> navigator.openSupport()
                                        SettingsAction.Rewards -> navigator.openReferral()
                                        SettingsAction.AboutUs -> navigator.openAboutUs()
                                        SettingsAction.Developer -> navigator.openDeveloper()
                                    }
                                },
                            )
                        }
                    }
                }
            }
        }
    }
}
