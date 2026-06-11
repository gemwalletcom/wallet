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
import com.gemwallet.android.cases.wallet.WalletImportResult
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.features.activities.presents.details.TransactionDetailsAction
import com.gemwallet.android.features.asset_select.presents.navigation.assetsManageScreen
import com.gemwallet.android.features.assets.views.WalletSearchAction
import com.gemwallet.android.features.create_wallet.navigation.createWalletScreen
import com.gemwallet.android.features.import_wallet.navigation.importWalletScreen
import com.gemwallet.android.features.onboarding.OnboardingRoute
import com.gemwallet.android.features.onboarding.acceptTermsScreen
import com.gemwallet.android.features.main.views.MainScreen
import com.gemwallet.android.features.setup_wallet.navigation.setupWalletScreen
import com.gemwallet.android.features.wallet.presents.WalletImageSource
import com.gemwallet.android.ui.components.animation.navigationSlideTransition
import com.gemwallet.android.ui.navigation.routes.addAssetScreen
import com.gemwallet.android.ui.navigation.routes.amount
import com.gemwallet.android.ui.navigation.routes.assetChartScreen
import com.gemwallet.android.ui.navigation.routes.assetScreen
import com.gemwallet.android.ui.navigation.routes.bridgesScreen
import com.gemwallet.android.ui.navigation.routes.confirm
import com.gemwallet.android.ui.navigation.routes.fiatScreen
import com.gemwallet.android.ui.navigation.routes.nftCollection
import com.gemwallet.android.ui.navigation.routes.perpetualScreen
import com.gemwallet.android.ui.navigation.routes.receiveScreen
import com.gemwallet.android.ui.navigation.routes.recipientInput
import com.gemwallet.android.ui.navigation.routes.referral
import com.gemwallet.android.features.settings.contacts.presents.ContactsAction
import com.gemwallet.android.ui.navigation.routes.contactsScreen
import com.gemwallet.android.ui.navigation.routes.SettingsAction
import com.gemwallet.android.ui.navigation.routes.settingsScreen
import com.gemwallet.android.ui.navigation.routes.stake
import com.gemwallet.android.ui.navigation.routes.swap
import com.gemwallet.android.ui.navigation.routes.swapSelect
import com.gemwallet.android.ui.navigation.routes.transactionDetailsScreen
import com.gemwallet.android.ui.navigation.routes.walletScreen
import com.gemwallet.android.ui.navigation.routes.walletSearchScreen
import com.gemwallet.android.ui.navigation.routes.walletsScreen
import com.wallet.core.primitives.WalletId

@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun WalletNavGraph(
    modifier: Modifier = Modifier,
    navigator: WalletNavigator,
    onboard: @Composable () -> Unit,
    onAcceptTerms: () -> Unit,
    onWalletContentReady: () -> Unit = {},
) {
    val onCancel: () -> Unit = navigator::pop
    val currentOnWalletContentReady by rememberUpdatedState(onWalletContentReady)

    val entryProvider = remember(navigator, onboard, onAcceptTerms) {
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
                        is WalletSearchAction.OpenAsset -> navigator.openAsset(action.assetId)
                        is WalletSearchAction.OpenPerpetual -> navigator.openPerpetualDetails(action.assetId)
                        is WalletSearchAction.ShowAllAssets -> navigator.openAssetsResults(action.query, action.tag)
                        else -> Unit
                    }
                },
            )

            assetScreen(
                onCancel = onCancel,
                onTransfer = navigator::openRecipient,
                onReceive = navigator::openReceive,
                onBuy = navigator::openBuy,
                onSwap = navigator::openSwap,
                onTransaction = navigator::openTransaction,
                onChart = navigator::openAssetChart,
                openNetwork = navigator::openAsset,
                onStake = navigator::openStake,
                onConfirm = navigator::openConfirm,
                onPriceAlerts = navigator::openPriceAlerts,
            )

            assetChartScreen(
                onPriceAlerts = navigator::openPriceAlerts,
                onAddPriceAlertTarget = navigator::openAddPriceAlertTarget,
                toastMessage = navigator::toastMessage,
                onToastShown = navigator::clearToastMessage,
                onCancel = onCancel,
            )

            swap(
                navigator = navigator,
                onConfirm = navigator::openConfirm,
                onSelect = navigator::openSwapSelect,
                onCancel = onCancel,
            )
            swapSelect(navigator = navigator, onCancel = onCancel)

            recipientInput(
                cancelAction = onCancel,
                recipientAction = navigator::openRecipient,
                amountAction = navigator::openAmount,
                confirmAction = navigator::openConfirm,
            )

            amount(
                onCancel = onCancel,
                onConfirm = navigator::openConfirm,
            )

            confirm(
                finishAction = { _ -> navigator.popConfirmFlow() },
                onBuy = navigator::openBuy,
                cancelAction = onCancel,
            )

            nftCollection(
                cancelAction = onCancel,
                collectionIdAction = navigator::openNftCollection,
                assetIdAction = navigator::openNftAsset,
                onRecipient = navigator::openNftRecipient,
                onReceive = navigator::openReceiveNftChains,
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
                onEditWallet = navigator::openWallet,
                onSelectWallet = navigator::resetToWallet,
                onBoard = navigator::resetToOnboarding,
                onCancel = onCancel,
            )

            walletScreen(
                onCancel = onCancel,
                onBoard = navigator::resetToOnboarding,
                onSelectImage = { navigator.openWalletImage(it) },
                onSecurityReminder = navigator::openWalletSecurityReminder,
                onSecurityReminderAccepted = navigator::finishWalletSecurityReminder,
            )

            stake(
                onAmount = navigator::openAmount,
                onConfirm = navigator::openConfirm,
                onDelegation = navigator::openDelegation,
                onCancel = onCancel,
            )

            addAssetScreen(
                onCancel = onCancel,
                onFinish = navigator::resetToWallet,
            )

            transactionDetailsScreen(
                onAction = {
                    when (it) {
                        TransactionDetailsAction.Close -> onCancel()
                        is TransactionDetailsAction.OpenAsset -> navigator.openAsset(it.assetId)
                        is TransactionDetailsAction.OpenNft -> navigator.openNftAsset(it.assetId)
                        is TransactionDetailsAction.OpenPerpetual -> navigator.openPerpetualDetails(it.assetId)
                        is TransactionDetailsAction.OpenSwap -> navigator.openSwap(it.fromAssetId, it.toAssetId)
                    }
                },
            )

            bridgesScreen(
                onConnection = navigator::openBridgeConnectionDetails,
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
                        is SettingsAction.Chart -> navigator.openAssetChart(action.assetId)
                        SettingsAction.InAppNotifications -> navigator.openInAppNotifications()
                        is SettingsAction.OpenNotificationUrl -> navigator.openNotificationUrl(action.url)
                        SettingsAction.Cancel -> onCancel()
                    }
                },
                toastMessage = navigator::toastMessage,
                onToastShown = navigator::clearToastMessage,
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
                onCreated = { walletId ->
                    if (walletId != null) {
                        navigator.openSetupWallet(walletId)
                    } else {
                        navigator.resetToWallet()
                    }
                },
            )

            importWalletScreen(
                onCancel = onCancel,
                onImported = { result ->
                    when (result) {
                        is WalletImportResult.New -> navigator.openSetupWallet(result.wallet.id)
                        is WalletImportResult.Existing -> navigator.resetToWallet()
                    }
                },
                onSelectType = navigator::openImportWallet,
            )

            setupWalletScreen(
                onComplete = navigator::resetToWallet,
                onSelectImage = { navigator.openWalletImage(it, WalletImageSource.Onboarding) },
            )

            perpetualScreen(
                onOpenPerpetualDetails = navigator::openPerpetualDetails,
                amountAction = AmountTransactionAction(navigator::openAmount),
                confirmAction = ConfirmTransactionAction(navigator::openConfirm),
                onCancel = onCancel,
                onTransaction = navigator::openTransaction,
            )

            referral(onClose = onCancel)
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
private fun rememberWalletNavEntries(
    backStack: List<NavKey>,
    entryProvider: (NavKey) -> NavEntry<NavKey>,
): List<NavEntry<NavKey>> {
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
