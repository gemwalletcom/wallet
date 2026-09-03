package com.gemwallet.android.ui.navigation

import com.gemwallet.android.ui.LocalAssetsService
import com.gemwallet.android.ui.LocalDeeplinkService
import com.gemwallet.android.ui.LocalTransferService
import uniffi.gemstone.GemAssetsServiceInterface
import uniffi.gemstone.GemTransferService
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.remember
import androidx.navigation3.runtime.NavBackStack
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.asset_select.presents.navigation.AssetsManageRoute
import com.gemwallet.android.features.confirm.presents.AcquireAssetAction
import com.gemwallet.android.ui.navigation.routes.AssetsResultsRoute
import com.gemwallet.android.ui.navigation.routes.WalletSearchRoute
import com.gemwallet.android.features.create_wallet.navigation.CreateWalletAlertRoute
import com.gemwallet.android.features.create_wallet.navigation.CreateWalletRoute
import com.gemwallet.android.features.import_wallet.navigation.ImportChainWalletRoute
import com.gemwallet.android.features.import_wallet.navigation.ImportMulticoinWalletRoute
import com.gemwallet.android.features.import_wallet.navigation.ImportSelectTypeRoute
import com.gemwallet.android.features.onboarding.AcceptTermsDestination
import com.gemwallet.android.features.onboarding.AcceptTermsRoute
import com.gemwallet.android.features.onboarding.OnboardingRoute
import com.gemwallet.android.features.setup_wallet.navigation.SetupWalletRoute
import com.gemwallet.android.features.wallet.presents.WalletImageSource
import com.gemwallet.android.domains.search.WalletSearchTag
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.domains.confirm.pack
import uniffi.gemstone.GemTransferData
import com.gemwallet.android.model.ImportType
import com.gemwallet.android.model.PaymentRecipient
import com.gemwallet.android.toRoute
import com.gemwallet.android.ui.navigation.routes.AboutusRoute
import com.gemwallet.android.ui.navigation.routes.AddAssetRoute
import com.gemwallet.android.ui.navigation.routes.NetworkAssetsRoute
import com.gemwallet.android.ui.navigation.routes.AddPriceAlertTargetRoute
import com.gemwallet.android.ui.navigation.routes.AssetPriceAlertsRoute
import com.gemwallet.android.ui.navigation.routes.AmountRoute
import com.gemwallet.android.ui.navigation.routes.AssetChartRoute
import com.gemwallet.android.ui.navigation.routes.AssetRoute
import com.gemwallet.android.ui.navigation.routes.assetsRoute
import com.gemwallet.android.ui.navigation.routes.BridgeConnectionDetailsRoute
import com.gemwallet.android.ui.navigation.routes.BridgeConnectionsRoute
import com.gemwallet.android.ui.navigation.routes.AddContactRoute
import com.gemwallet.android.ui.navigation.routes.ConfirmRoute
import com.gemwallet.android.ui.navigation.routes.ContactsRoute
import com.gemwallet.android.ui.navigation.routes.CurrenciesRoute
import com.gemwallet.android.ui.navigation.routes.EditContactRoute
import com.gemwallet.android.ui.navigation.routes.DelegationRoute
import com.gemwallet.android.ui.navigation.routes.DevelopRoute
import com.gemwallet.android.ui.navigation.routes.DevelopPaymentsRoute
import com.gemwallet.android.ui.navigation.routes.FiatInputRoute
import com.gemwallet.android.ui.navigation.routes.FiatSelectRoute
import com.gemwallet.android.ui.navigation.routes.FiatTransactionsRoute
import com.gemwallet.android.ui.navigation.routes.InAppNotificationsRoute
import com.gemwallet.android.ui.navigation.routes.NetworksRoute
import com.gemwallet.android.ui.navigation.routes.NftAssetRoute
import com.gemwallet.android.ui.navigation.routes.NftCollectionRoute
import com.gemwallet.android.ui.navigation.routes.NftListRoute
import com.gemwallet.android.ui.navigation.routes.NftUnverifiedCollectionsRoute
import com.gemwallet.android.ui.navigation.routes.NotificationsRoute
import com.gemwallet.android.ui.navigation.routes.PerpetualPositionRoute
import com.gemwallet.android.ui.navigation.routes.PerpetualRoute
import com.gemwallet.android.ui.navigation.routes.PreferencesRoute
import com.gemwallet.android.ui.navigation.routes.PriceAlertsRoute
import com.gemwallet.android.ui.navigation.routes.ReceiveNftChainsRoute
import com.gemwallet.android.ui.navigation.routes.ReceiveRoute
import com.gemwallet.android.ui.navigation.routes.ReceiveSelectRoute
import com.gemwallet.android.ui.navigation.routes.RecipientInputRoute
import com.gemwallet.android.ui.navigation.routes.ReferralRoute
import com.gemwallet.android.ui.navigation.routes.SecurityRoute
import com.gemwallet.android.ui.navigation.routes.SendSelectRoute
import com.gemwallet.android.ui.navigation.routes.StakeRoute
import com.gemwallet.android.ui.navigation.routes.SupportRoute
import com.gemwallet.android.ui.navigation.routes.SwapPairRoute
import com.gemwallet.android.ui.navigation.routes.SwapRoute
import com.gemwallet.android.ui.navigation.routes.SwapSelectRoute
import com.gemwallet.android.ui.navigation.routes.PortfolioChartRoute
import com.gemwallet.android.ui.navigation.routes.TransactionDetailsRoute
import com.gemwallet.android.ui.navigation.routes.WalletDetailsRoute
import com.gemwallet.android.ui.navigation.routes.WalletImageRoute
import com.gemwallet.android.ui.navigation.routes.WalletPhraseRoute
import com.gemwallet.android.ui.navigation.routes.WalletSecurityReminderRoute
import com.gemwallet.android.ui.navigation.routes.WalletsRoute
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.FiatQuoteType
import com.wallet.core.primitives.NFTAssetId
import com.wallet.core.primitives.PortfolioType
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletType
import kotlinx.serialization.Serializable
import uniffi.gemstone.UrlAction
import uniffi.gemstone.GemDeeplinkService

@Serializable
data object WalletRootRoute : NavKey

data class SwapSelection(
    val itemType: SwapItemType,
    val payAssetId: AssetId?,
    val receiveAssetId: AssetId?,
)

@Composable
fun rememberWalletNavigationState(
    startDestination: NavKey,
    currentTab: MutableState<String>,
): WalletNavigator {
    val transferService = LocalTransferService.current
    val deeplinkService = LocalDeeplinkService.current
    val assetsService = LocalAssetsService.current
    val scope = rememberCoroutineScope()
    return key(startDestination) {
        val backStack = rememberWalletNavBackStack(startDestination)
        remember(backStack, currentTab, transferService, deeplinkService, assetsService, scope) {
            WalletNavigator(
                backStack = backStack,
                currentTab = currentTab,
                transferService = transferService,
                deeplinkService = deeplinkService,
                assetsService = assetsService,
                scope = scope,
            )
        }
    }
}

class WalletNavigator(
    val backStack: NavBackStack<NavKey>,
    val currentTab: MutableState<String>,
    private val transferService: GemTransferService,
    private val deeplinkService: GemDeeplinkService,
    private val assetsService: GemAssetsServiceInterface,
    private val scope: CoroutineScope,
) {
    private val toastMessages = mutableStateMapOf<NavKey, String>()
    private val swapSelections = mutableStateMapOf<NavKey, SwapSelection>()

    private fun push(route: NavKey): Boolean {
        if (backStack.lastOrNull() == route) return true
        return backStack.add(route)
    }

    private fun openAssetRoute(route: AssetRoute) {
        scope.launch {
            if (assetsService.openAsset(route.assetId.toIdentifier()) != null) {
                push(route)
            }
        }
    }

    private fun replaceTop(route: NavKey) {
        if (backStack.isNotEmpty()) {
            backStack.removeLastOrNull()
        }
        backStack.add(route)
    }

    fun pop() {
        if (backStack.size > 1) {
            backStack.removeLastOrNull()
        }
    }

    fun resetToWallet() {
        resetTo(WalletRootRoute)
    }

    fun resetToOnboarding() {
        resetTo(OnboardingRoute)
    }

    private fun resetTo(route: NavKey) {
        clearTransientState()
        currentTab.value = assetsRoute
        backStack.clear()
        backStack.add(route)
    }

    fun toastMessage(route: NavKey): String? {
        return toastMessages[route]
    }

    fun clearToastMessage(route: NavKey) {
        toastMessages.remove(route)
    }

    fun popWithToast(message: String) {
        val target = backStack.getOrNull(backStack.lastIndex - 1) ?: return
        toastMessages[target] = message
        pop()
    }

    fun swapSelection(route: NavKey): SwapSelection? {
        return swapSelections[route]
    }

    fun clearSwapSelection(route: NavKey) {
        swapSelections.remove(route)
    }

    fun openWallets() = push(WalletsRoute)
    fun openAcceptTerms(destination: AcceptTermsDestination) = push(AcceptTermsRoute(destination))
    fun openAssetsManage(chain: Chain? = null) = push(AssetsManageRoute(chain))
    fun openAssetsSearch() = push(WalletSearchRoute)
    fun openAssetsResults(query: String) = push(AssetsResultsRoute(query, WalletSearchTag.All))
    fun openAssetsResultsList(listId: String, title: String) = push(AssetsResultsRoute(query = "", scope = WalletSearchTag.List(listId), title = title))
    fun openCreateWalletRules() = push(CreateWalletAlertRoute)
    fun openCreateWallet() = push(CreateWalletRoute)
    fun openImportWallet() = push(ImportSelectTypeRoute)
    fun openImportWallet(importType: ImportType) {
        importType.toImportRoute()?.let(::push)
    }
    fun openWallet(walletId: WalletId) = push(WalletDetailsRoute(walletId))
    fun openWalletImage(walletId: WalletId, source: WalletImageSource = WalletImageSource.Wallet) = push(WalletImageRoute(walletId, source))
    fun openWalletSecurityReminder(walletId: WalletId, type: WalletType) = push(WalletSecurityReminderRoute(walletId, type))
    fun finishWalletSecurityReminder(walletId: WalletId, type: WalletType) = replaceTop(WalletPhraseRoute(walletId, type))
    fun openSetupWallet(walletId: WalletId) = replaceTop(SetupWalletRoute(walletId))
    fun openAddAsset() = push(AddAssetRoute)
    fun openAsset(assetId: AssetId) = openAssetRoute(AssetRoute(assetId))
    fun openNetworkAssets(chain: Chain) = push(NetworkAssetsRoute(chain))
    fun openAssetChart(assetId: AssetId) = push(AssetChartRoute(assetId))
    fun openPortfolioChart(type: PortfolioType = PortfolioType.Wallet) = push(PortfolioChartRoute(type))
    fun openTransaction(transactionId: TransactionId) = push(TransactionDetailsRoute(transactionId))
    fun openBridgeConnections() = push(BridgeConnectionsRoute)
    fun openBridgeConnectionDetails(connectionId: String) = push(BridgeConnectionDetailsRoute(connectionId))
    fun openCurrencies() = push(CurrenciesRoute)
    fun openContacts() = push(ContactsRoute)
    fun openAddContact() = push(AddContactRoute)
    fun openContact(contactId: String) = push(EditContactRoute(contactId))
    fun openSecurity() = push(SecurityRoute)
    fun openDevelop() = push(DevelopRoute)
    fun openDeveloperPayments() = push(DevelopPaymentsRoute)
    fun openInAppNotifications() = push(InAppNotificationsRoute)
    fun openNotificationUrl(url: String): Boolean {
        val action = runCatching { deeplinkService.urlAction(url) }.getOrNull() as? UrlAction.Deeplink ?: return false
        when (val route = action.deeplink.toRoute() ?: return false) {
            is AssetRoute -> openAssetRoute(route)
            else -> push(route)
        }
        return true
    }
    fun openAboutUs() = push(AboutusRoute)
    fun openNetworks() = push(NetworksRoute)
    fun openNotifications() = push(NotificationsRoute)
    fun openPreferences() = push(PreferencesRoute)
    fun openSupport() = push(SupportRoute)
    fun openReferral(code: String? = null) = push(ReferralRoute(code))
    fun openPriceAlerts() = push(PriceAlertsRoute)
    fun openPriceAlerts(assetId: AssetId) = push(AssetPriceAlertsRoute(assetId))
    fun openAddPriceAlertTarget(assetId: AssetId) = push(AddPriceAlertTargetRoute(assetId))
    fun openPerpetuals() = push(PerpetualRoute)
    fun openPerpetualDetails(assetId: AssetId) = push(PerpetualPositionRoute(assetId))
    fun openStake(assetId: AssetId) = push(StakeRoute(assetId))
    fun openDelegation(validatorId: String, delegationId: String) = push(DelegationRoute(validatorId, delegationId))
    fun openReceive() = push(ReceiveSelectRoute)
    fun openReceive(assetId: AssetId) = push(ReceiveRoute(assetId))
    fun openReceiveNftChains() = push(ReceiveNftChainsRoute)
    fun openRecipient(payment: PaymentRecipient? = null, chains: List<Chain> = emptyList()) = push(SendSelectRoute(payment, chains))
    fun openRecipient(assetId: AssetId, payment: PaymentRecipient? = null) = push(RecipientInputRoute(assetId, nftAssetId = null, payment = payment))
    fun openNftRecipient(assetId: AssetId, nftAssetId: NFTAssetId) = push(RecipientInputRoute(assetId, nftAssetId.toIdentifier()))
    fun openAmount(params: AmountParams) {
        val pack = params.pack() ?: return
        push(AmountRoute(pack))
    }
    fun openSwap() {
        clearSwapSelections()
        push(SwapRoute)
    }
    fun openSwap(from: AssetId, to: AssetId? = null) {
        clearSwapSelections()
        push(SwapPairRoute(from, to))
    }
    fun openSwapTo(assetId: AssetId) {
        clearSwapSelections()
        swapSelections[SwapRoute] = SwapSelection(
            itemType = SwapItemType.Receive,
            payAssetId = null,
            receiveAssetId = assetId,
        )
        push(SwapRoute)
    }
    fun openSwapSelect(itemType: SwapItemType, payAssetId: AssetId?, receiveAssetId: AssetId?) {
        push(SwapSelectRoute(itemType, payAssetId, receiveAssetId))
    }
    fun finishSwapSelect(itemType: SwapItemType, payAssetId: AssetId?, receiveAssetId: AssetId?) {
        val target = backStack.getOrNull(backStack.lastIndex - 1) ?: return
        swapSelections[target] = SwapSelection(
            itemType = itemType,
            payAssetId = payAssetId,
            receiveAssetId = receiveAssetId,
        )
        pop()
    }
    private fun clearSwapSelections() = swapSelections.clear()
    fun openBuy() = push(FiatSelectRoute)
    fun openBuy(assetId: AssetId) = openBuy(assetId, amount = null)
    fun openBuy(assetId: AssetId, amount: Int?) = push(FiatInputRoute(assetId, amount, FiatQuoteType.Buy))
    fun openAcquireAsset(action: AcquireAssetAction, assetId: AssetId) {
        when (action) {
            is AcquireAssetAction.Buy -> openBuy(assetId, amount = action.amount)
            AcquireAssetAction.Swap -> openSwapTo(assetId)
            AcquireAssetAction.Receive -> openReceive(assetId)
        }
    }
    fun openFiatTransactions() = push(FiatTransactionsRoute)
    fun openConfirm(transfer: GemTransferData) {
        val pack = transferService.pack(transfer) ?: return
        push(ConfirmRoute(pack))
    }
    fun openNftList() = push(NftListRoute)
    fun openNftCollection(nftCollectionId: String) = push(NftCollectionRoute(nftCollectionId))
    fun openNftUnverifiedCollections() = push(NftUnverifiedCollectionsRoute)
    fun openNftAsset(nftAssetId: NFTAssetId) = push(NftAssetRoute(nftAssetId.toIdentifier()))

    fun finishAcceptTerms(destination: AcceptTermsDestination) {
        replaceTop(
            when (destination) {
                AcceptTermsDestination.Create -> CreateWalletAlertRoute
                AcceptTermsDestination.Import -> ImportSelectTypeRoute
            }
        )
    }

    internal fun openPendingNavigation(routes: List<NavKey>, confirmed: Boolean = false): Boolean {
        if (routes.isEmpty()) return false
        if (!canOpenPendingNavigation()) return false
        if (!confirmed && needsPendingNavigationConfirmation()) return false
        resetToWallet()
        routes.forEach(::push)
        return true
    }

    internal fun needsPendingNavigationConfirmation(): Boolean {
        return canOpenPendingNavigation() && backStack.lastOrNull()?.isPendingNavigationProtectedRoute() == true
    }

    private fun canOpenPendingNavigation(): Boolean {
        return backStack.firstOrNull() == WalletRootRoute
    }

    fun popConfirmFlow() {
        popFrom(backStack.indexOfLast { !it.isConfirmFlowSegmentRoute() } + 1)
    }

    private fun popFrom(index: Int) {
        while (backStack.lastIndex >= index && backStack.size > 1) {
            backStack.removeLastOrNull()
        }
    }

    private fun clearTransientState() {
        clearSwapSelections()
        toastMessages.clear()
    }
}

internal fun NavKey.isConfirmFlowSegmentRoute(): Boolean {
    return when (this) {
        SwapRoute -> true
        is SendSelectRoute,
        is AmountRoute,
        is ConfirmRoute,
        is DelegationRoute,
        is RecipientInputRoute,
        is StakeRoute,
        is SwapPairRoute,
        is SwapSelectRoute -> true
        else -> false
    }
}

internal fun NavKey.isPendingNavigationProtectedRoute(): Boolean {
    return isConfirmFlowSegmentRoute() ||
        this is WalletSecurityReminderRoute ||
        this is WalletPhraseRoute
}

private fun ImportType.toImportRoute(): NavKey? {
    return when (walletType) {
        WalletType.Multicoin -> ImportMulticoinWalletRoute
        WalletType.Single,
        WalletType.PrivateKey,
        WalletType.View -> chain?.let { ImportChainWalletRoute(walletType, it) }
    }
}
