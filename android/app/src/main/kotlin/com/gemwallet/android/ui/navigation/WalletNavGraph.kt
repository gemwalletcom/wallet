package com.gemwallet.android.ui.navigation

import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.ContentTransform
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.testTagsAsResourceId
import androidx.navigation3.runtime.NavEntry
import androidx.navigation3.runtime.NavKey
import androidx.navigation3.runtime.entryProvider
import androidx.navigation3.runtime.rememberDecoratedNavEntries
import androidx.navigation3.runtime.rememberSaveableStateHolderNavEntryDecorator
import androidx.navigation3.scene.Scene
import androidx.navigation3.ui.NavDisplay
import com.gemwallet.android.features.assets.presents.select.assetsManageScreen
import com.gemwallet.android.features.assets.viewmodels.asset.models.AssetAction
import com.gemwallet.android.features.contacts.presents.ContactsAction
import com.gemwallet.android.features.main.views.MainScreen
import com.gemwallet.android.features.onboarding.presents.create_wallet.createWalletScreen
import com.gemwallet.android.features.onboarding.presents.import_wallet.importWalletScreen
import com.gemwallet.android.features.onboarding.presents.terms.acceptTermsScreen
import com.gemwallet.android.features.transactions.presents.transaction.TransactionAction
import com.gemwallet.android.features.wallet_tab.presents.WalletSearchAction
import com.gemwallet.android.ui.components.animation.navigationSlideTransition
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.navigation.routes.SettingsAction
import com.gemwallet.android.ui.navigation.routes.addAssetScreen
import com.gemwallet.android.ui.navigation.routes.addressDetailsScreen
import com.gemwallet.android.ui.navigation.routes.amount
import com.gemwallet.android.ui.navigation.routes.assetScreen
import com.gemwallet.android.ui.navigation.routes.chartScreen
import com.gemwallet.android.ui.navigation.routes.collectionsScreen
import com.gemwallet.android.ui.navigation.routes.confirmTransfer
import com.gemwallet.android.ui.navigation.routes.connectionsScreen
import com.gemwallet.android.ui.navigation.routes.contactsScreen
import com.gemwallet.android.ui.navigation.routes.fiatScreen
import com.gemwallet.android.ui.navigation.routes.networkAssetsScreen
import com.gemwallet.android.ui.navigation.routes.perpetualsScreen
import com.gemwallet.android.ui.navigation.routes.portfolioScreen
import com.gemwallet.android.ui.navigation.routes.receiveScreen
import com.gemwallet.android.ui.navigation.routes.recipient
import com.gemwallet.android.ui.navigation.routes.referral
import com.gemwallet.android.ui.navigation.routes.settingsScreen
import com.gemwallet.android.ui.navigation.routes.stake
import com.gemwallet.android.ui.navigation.routes.swap
import com.gemwallet.android.ui.navigation.routes.swapSelect
import com.gemwallet.android.ui.navigation.routes.transactionScreen
import com.gemwallet.android.ui.navigation.routes.walletConnectorRequest
import com.gemwallet.android.ui.navigation.routes.walletDetailScreen
import com.gemwallet.android.ui.navigation.routes.walletSearchScreen
import com.gemwallet.android.ui.navigation.routes.walletsScreen
import com.wallet.core.primitives.PortfolioType

@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun WalletNavGraph(
    modifier: Modifier = Modifier,
    navigator: WalletNavigator,
    onboard: @Composable () -> Unit,
    onAcceptTerms: () -> Unit,
    onPayment: (String) -> Unit,
    onWalletContentReady: () -> Unit = {},
    walletConnectorRequest: @Composable (String) -> Unit = {},
) {
    val onCancel: () -> Unit = navigator::pop
    val currentOnWalletContentReady by rememberUpdatedState(onWalletContentReady)
    val currentOnPayment by rememberUpdatedState(onPayment)

    val entryProvider = remember(navigator, onboard, onAcceptTerms, walletConnectorRequest) {
        entryProvider<NavKey> {
            entry<WalletRootRoute> {
                MainScreen(
                    navigator = navigator,
                    currentTab = navigator.currentTab,
                    onWalletContentReady = { currentOnWalletContentReady() },
                )
            }

            entry<OnboardingRoute> {
                onboard()
            }

            assetsManageScreen(
                onAddAsset = navigator::openAddAsset,
                onAssetClick = navigator::openAsset,
                onCancel = onCancel,
            )

            walletSearchScreen(
                onAction = { action ->
                    when (action) {
                        WalletSearchAction.AddAsset -> navigator.openAddAsset()
                        WalletSearchAction.Cancel -> onCancel()
                        WalletSearchAction.OpenPerpetuals -> navigator.openPerpetuals()
                        WalletSearchAction.OpenCollections -> navigator.openCollections()
                        is WalletSearchAction.OpenAsset -> navigator.openAsset(action.asset.id)
                        is WalletSearchAction.OpenPerpetual -> navigator.openPerpetual(action.asset.id)
                        is WalletSearchAction.OpenRecent -> navigator.openRecent(action.asset)
                        is WalletSearchAction.OpenNftCollection -> navigator.openCollection(action.collectionId)
                        is WalletSearchAction.OpenNftAsset -> navigator.openCollectible(action.assetId)
                        is WalletSearchAction.ShowAllAssets -> navigator.openAssetsResults(action.query)
                        is WalletSearchAction.OpenList -> navigator.openAssetsResultsList(action.listId, action.title)
                        else -> Unit
                    }
                },
            )

            assetScreen(
                onAction = { action ->
                    when (action) {
                        AssetAction.Close -> onCancel()
                        is AssetAction.Transfer -> navigator.openRecipient(action.assetId)
                        is AssetAction.Receive -> navigator.openReceive(action.assetId)
                        is AssetAction.Buy -> navigator.openBuy(action.assetId)
                        is AssetAction.Swap -> navigator.openSwap(action.fromAssetId, action.toAssetId)
                        is AssetAction.OpenTransaction -> navigator.openTransaction(action.transactionId)
                        is AssetAction.OpenChart -> navigator.openChart(action.assetId)
                        is AssetAction.OpenNetwork -> navigator.openAsset(action.assetId)
                        is AssetAction.OpenNetworkAssets -> navigator.openNetworkAssets(action.chain)
                        is AssetAction.Stake -> navigator.openStake(action.assetId)
                        is AssetAction.Earn -> navigator.openEarn(action.assetId)
                        AssetAction.OpenPerpetuals -> navigator.openPerpetuals()
                        is AssetAction.OpenPriceAlerts -> navigator.openPriceAlerts(action.assetId)
                        is AssetAction.Confirm -> navigator.openConfirmTransfer(action.input)
                    }
                },
            )
            networkAssetsScreen(
                onSelectAsset = navigator::openAsset,
                onManageAssets = { navigator.openAssetsManage(it) },
                onCancel = onCancel,
            )

            chartScreen(
                onPriceAlerts = navigator::openPriceAlerts,
                onAddPriceAlertTarget = navigator::openAddPriceAlertTarget,
                onOpenAddress = navigator::openAddress,
                routeMessage = navigator::routeMessage,
                onRouteMessageShown = navigator::clearRouteMessage,
                onCancel = onCancel,
            )

            portfolioScreen(
                onCancel = onCancel,
            )

            swap(
                navigator = navigator,
                onConfirm = navigator::openConfirmTransfer,
                onSelect = navigator::openSwapSelect,
                onCancel = onCancel,
            )
            swapSelect(navigator = navigator, onCancel = onCancel)

            recipient(
                navigator = navigator,
                cancelAction = onCancel,
                amountAction = navigator::openAmount,
                confirmAction = navigator::openConfirmTransfer,
            )

            amount(
                onCancel = onCancel,
                onConfirm = navigator::openConfirmTransfer,
                onBuy = { navigator.openBuy(it) },
            )

            confirmTransfer(
                navigator = navigator,
                finishAction = { _, warning -> navigator.popConfirmFlow(warning) },
                onGetAsset = navigator::openGetAsset,
                cancelAction = onCancel,
            )

            collectionsScreen(
                cancelAction = onCancel,
                collectionIdAction = navigator::openCollection,
                assetIdAction = navigator::openCollectible,
                onRecipient = navigator::openNftRecipient,
                onReceive = navigator::openReceiveCollection,
                onUnverified = navigator::openUnverifiedCollections,
                onOpenAddress = navigator::openAddress,
            )

            fiatScreen(
                cancelAction = onCancel,
                onBuy = navigator::openBuy,
                onFiatTransactions = navigator::openFiatTransactions,
            )

            receiveScreen(
                onCancel = onCancel,
                onReceive = navigator::openReceive,
            )

            walletsScreen(
                onCreateWallet = navigator::openCreateWalletRules,
                onImportWallet = navigator::openImportWallet,
                onEditWallet = navigator::openWalletDetail,
                onSelectWallet = navigator::resetToWallet,
                onBoard = navigator::resetToOnboarding,
                onCancel = onCancel,
            )

            walletDetailScreen(
                onCancel = onCancel,
                onBoard = navigator::resetToOnboarding,
                onSelectImage = { navigator.openWalletImage(it) },
                onSecurityReminder = navigator::openWalletSecurityReminder,
                onSecurityReminderAccepted = navigator::finishWalletSecurityReminder,
            )

            stake(
                onAmount = navigator::openAmount,
                onConfirm = navigator::openConfirmTransfer,
                onDelegation = navigator::openDelegation,
                onOpenAddress = navigator::openAddress,
                onCancel = onCancel,
            )

            addAssetScreen(
                onCancel = onCancel,
                onFinish = navigator::resetToWallet,
            )

            transactionScreen(
                onAction = {
                    when (it) {
                        TransactionAction.Close -> onCancel()
                        is TransactionAction.OpenAsset -> navigator.openAsset(it.assetId)
                        is TransactionAction.OpenNft -> navigator.openCollectible(it.assetId)
                        is TransactionAction.OpenPerpetual -> navigator.openPerpetual(it.assetId)
                        is TransactionAction.OpenSwap -> navigator.openSwap(it.fromAssetId, it.toAssetId)
                        is TransactionAction.OpenAddress -> navigator.openAddress(it.chainAddress)
                    }
                },
            )

            connectionsScreen(
                onConnection = navigator::openConnection,
                onCancel = onCancel,
            )

            settingsScreen(
                onAction = { action ->
                    when (action) {
                        SettingsAction.Currencies -> navigator.openCurrencies()
                        SettingsAction.Contacts -> navigator.openContacts()
                        SettingsAction.Networks -> navigator.openNetworks()
                        SettingsAction.PriceAlerts -> navigator.openPriceAlerts()
                        is SettingsAction.AddPriceAlertTarget -> navigator.openAddPriceAlertTarget(action.assetId)
                        is SettingsAction.PriceAlertTargetComplete -> navigator.popWithToast(action.message)
                        is SettingsAction.Chart -> navigator.openChart(action.assetId)
                        SettingsAction.InAppNotifications -> navigator.openInAppNotifications()
                        SettingsAction.DeveloperPayments -> navigator.openDeveloperPayments()
                        is SettingsAction.Payment -> currentOnPayment(action.payload)
                        is SettingsAction.OpenNotification -> navigator.openUrlAction(action.action)
                        SettingsAction.Cancel -> onCancel()
                    }
                },
                onOpenUrl = navigator::openNotificationUrl,
                routeMessage = navigator::routeMessage,
                onRouteMessageShown = navigator::clearRouteMessage,
            )

            contactsScreen(
                onAction = { action ->
                    when (action) {
                        is ContactsAction.OpenContact -> navigator.openContact(action.contactId)
                        ContactsAction.AddContact -> navigator.openAddContact()
                        ContactsAction.Cancel -> onCancel()
                    }
                },
            )

            acceptTermsScreen(
                onCancel = onCancel,
                onAccept = { destination ->
                    onAcceptTerms()
                    navigator.finishAcceptTerms(destination)
                },
            )

            createWalletScreen(
                onCreateWallet = navigator::openCreateWallet,
                onCancel = onCancel,
                onCreated = navigator::resetToWallet,
            )

            importWalletScreen(
                onCancel = onCancel,
                onImported = navigator::resetToWallet,
                onSelectType = navigator::openImportWallet,
            )

            perpetualsScreen(
                onOpenPerpetual = navigator::openPerpetual,
                onOpenPortfolio = { navigator.openPortfolio(PortfolioType.Perpetuals) },
                amountAction = AmountTransactionAction(navigator::openAmount),
                confirmAction = ConfirmTransactionAction(navigator::openConfirmTransfer),
                onCancel = onCancel,
                onTransaction = navigator::openTransaction,
                onGetAsset = navigator::openGetAsset,
            )

            referral(onClose = onCancel)

            addressDetailsScreen(onCancel = onCancel)

            walletConnectorRequest(content = walletConnectorRequest)
        }
    }
    val entries = rememberWalletNavEntries(navigator.backStack, entryProvider)
    val decoratedEntries = rememberDecoratedNavEntries(
        entries = entries,
        entryDecorators = listOf(
            rememberSaveableStateHolderNavEntryDecorator(),
            rememberRouteArgumentsViewModelStoreNavEntryDecorator(),
        ),
    )

    NavDisplay(
        entries = decoratedEntries,
        modifier = modifier.semantics { testTagsAsResourceId = true },
        onBack = { navigator.pop() },
        transitionSpec = slideLeftTransition,
        popTransitionSpec = slideRightTransition,
        predictivePopTransitionSpec = { slideRightTransition() },
    )
}

@Composable
private fun rememberWalletNavEntries(backStack: List<NavKey>, entryProvider: (NavKey) -> NavEntry<NavKey>): List<NavEntry<NavKey>> {
    val keys = backStack.toList()
    return remember(keys, entryProvider) {
        val occurrences = mutableMapOf<Any, Int>()
        keys.map { key ->
            val entry = entryProvider(key)
            val occurrence = occurrences.getOrDefault(entry.contentKey, 0)
            occurrences[entry.contentKey] = occurrence + 1
            entry.withOccurrenceContentKey(key, occurrence)
        }
    }
}

private typealias WalletNavTransition = AnimatedContentTransitionScope<Scene<NavKey>>.() -> ContentTransform

private val slideLeftTransition: WalletNavTransition = {
    navigationSlideTransition(AnimatedContentTransitionScope.SlideDirection.Left)
}

private val slideRightTransition: WalletNavTransition = {
    navigationSlideTransition(AnimatedContentTransitionScope.SlideDirection.Right)
}
