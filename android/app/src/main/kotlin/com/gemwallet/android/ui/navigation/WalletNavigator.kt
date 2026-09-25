package com.gemwallet.android.ui.navigation

import android.util.Log
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateMapOf
import androidx.navigation3.runtime.NavBackStack
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.domains.confirm.pack
import com.gemwallet.android.domains.search.WalletSearchTag
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.domains.wallet.WalletSecretInput
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.asset_select.presents.navigation.AssetsManageRoute
import com.gemwallet.android.features.confirm.viewmodels.models.AcquireAssetAction
import com.gemwallet.android.features.create_wallet.navigation.CreateWalletAlertRoute
import com.gemwallet.android.features.create_wallet.navigation.CreateWalletRoute
import com.gemwallet.android.features.import_wallet.navigation.ImportChainWalletRoute
import com.gemwallet.android.features.import_wallet.navigation.ImportMulticoinWalletRoute
import com.gemwallet.android.features.import_wallet.navigation.ImportSelectTypeRoute
import com.gemwallet.android.features.onboarding.AcceptTermsDestination
import com.gemwallet.android.features.onboarding.AcceptTermsRoute
import com.gemwallet.android.features.onboarding.OnboardingRoute
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.ImportType
import com.gemwallet.android.routes
import com.gemwallet.android.ui.models.navigation.RouteMessage
import com.gemwallet.android.ui.navigation.routes.AboutusRoute
import com.gemwallet.android.ui.navigation.routes.AddAssetRoute
import com.gemwallet.android.ui.navigation.routes.AddContactRoute
import com.gemwallet.android.ui.navigation.routes.AddPriceAlertTargetRoute
import com.gemwallet.android.ui.navigation.routes.AddressDetailsRoute
import com.gemwallet.android.ui.navigation.routes.AmountRoute
import com.gemwallet.android.ui.navigation.routes.AssetChartRoute
import com.gemwallet.android.ui.navigation.routes.AssetPriceAlertsRoute
import com.gemwallet.android.ui.navigation.routes.AssetRoute
import com.gemwallet.android.ui.navigation.routes.AssetsResultsRoute
import com.gemwallet.android.ui.navigation.routes.BridgeConnectionDetailsRoute
import com.gemwallet.android.ui.navigation.routes.BridgeConnectionsRoute
import com.gemwallet.android.ui.navigation.routes.ConfirmRoute
import com.gemwallet.android.ui.navigation.routes.ContactsRoute
import com.gemwallet.android.ui.navigation.routes.CurrenciesRoute
import com.gemwallet.android.ui.navigation.routes.DelegationRoute
import com.gemwallet.android.ui.navigation.routes.DevelopPaymentsRoute
import com.gemwallet.android.ui.navigation.routes.DevelopRoute
import com.gemwallet.android.ui.navigation.routes.EarnRoute
import com.gemwallet.android.ui.navigation.routes.EditContactRoute
import com.gemwallet.android.ui.navigation.routes.FiatInputRoute
import com.gemwallet.android.ui.navigation.routes.FiatSelectRoute
import com.gemwallet.android.ui.navigation.routes.FiatTransactionsRoute
import com.gemwallet.android.ui.navigation.routes.InAppNotificationsRoute
import com.gemwallet.android.ui.navigation.routes.NetworkAssetsRoute
import com.gemwallet.android.ui.navigation.routes.NetworksRoute
import com.gemwallet.android.ui.navigation.routes.NftAssetRoute
import com.gemwallet.android.ui.navigation.routes.NftCollectionRoute
import com.gemwallet.android.ui.navigation.routes.NftListRoute
import com.gemwallet.android.ui.navigation.routes.NftUnverifiedCollectionsRoute
import com.gemwallet.android.ui.navigation.routes.NotificationsRoute
import com.gemwallet.android.ui.navigation.routes.PaymentSelectRoute
import com.gemwallet.android.ui.navigation.routes.PaymentVerificationRoute
import com.gemwallet.android.ui.navigation.routes.PerpetualPositionRoute
import com.gemwallet.android.ui.navigation.routes.PerpetualRoute
import com.gemwallet.android.ui.navigation.routes.PortfolioChartRoute
import com.gemwallet.android.ui.navigation.routes.PreferencesRoute
import com.gemwallet.android.ui.navigation.routes.PriceAlertsRoute
import com.gemwallet.android.ui.navigation.routes.ReceiveCollectionRoute
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
import com.gemwallet.android.ui.navigation.routes.TransactionDetailsRoute
import com.gemwallet.android.ui.navigation.routes.WalletConnectRequestRoute
import com.gemwallet.android.ui.navigation.routes.WalletDetailsRoute
import com.gemwallet.android.ui.navigation.routes.WalletImageRoute
import com.gemwallet.android.ui.navigation.routes.WalletPhraseRoute
import com.gemwallet.android.ui.navigation.routes.WalletSearchRoute
import com.gemwallet.android.ui.navigation.routes.WalletSecurityReminderRoute
import com.gemwallet.android.ui.navigation.routes.WalletsRoute
import com.gemwallet.android.ui.navigation.routes.assetsRoute
import com.gemwallet.android.ui.navigation.routes.settingsRoute
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChainAddress
import com.wallet.core.primitives.FiatQuoteType
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.NFTAssetId
import com.wallet.core.primitives.PortfolioType
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemAssetsServiceInterface
import uniffi.gemstone.GemDeeplinkServiceInterface
import uniffi.gemstone.GemNavigationServiceInterface
import uniffi.gemstone.GemNavigationTab
import uniffi.gemstone.GemPaymentRecipient
import uniffi.gemstone.UrlAction
class WalletNavigator(
    val backStack: NavBackStack<NavKey>,
    val currentTab: MutableState<String>,
    private val deeplinkService: GemDeeplinkServiceInterface,
    private val assetsService: GemAssetsServiceInterface,
    private val navigationService: GemNavigationServiceInterface,
    private val scope: CoroutineScope,
) {
    private val routeMessages = mutableStateMapOf<NavKey, RouteMessage>()
    private val swapSelections = mutableStateMapOf<NavKey, SwapSelection>()
    private val paymentSelections = mutableStateMapOf<NavKey, AssetId>()

    private fun push(route: NavKey): Boolean {
        if (backStack.lastOrNull() == route) return true
        return backStack.add(route)
    }

    private fun <T> popWithResult(results: MutableMap<NavKey, T>, value: T) {
        val target = backStack.getOrNull(backStack.lastIndex - 1) ?: return
        results[target] = value
        pop()
    }

    private fun openAssetRoute(route: AssetRoute) = scope.launch {
        runCatchingCancellable { withContext(Dispatchers.IO) { assetsService.openAsset(route.assetId.toIdentifier()) } }
            .onSuccess { asset -> if (asset != null) push(route) }
            .onFailure { Log.e(TAG, "opening an asset failed", it) }
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

    fun showWalletConnectRequest(key: String?) {
        val route = key?.let(::WalletConnectRequestRoute)
        if (route != null && backStack.contains(route)) return
        backStack.removeAll { it is WalletConnectRequestRoute }
        if (route != null) {
            push(route)
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

    fun routeMessage(route: NavKey): RouteMessage? = routeMessages[route]

    fun clearRouteMessage(route: NavKey) {
        routeMessages.remove(route)
    }

    fun popWithToast(message: String) = popWithResult(routeMessages, RouteMessage.Toast(message))

    fun swapSelection(route: NavKey): SwapSelection? = swapSelections[route]

    fun paymentSelection(route: NavKey): AssetId? = paymentSelections[route]

    fun clearPaymentSelection(route: NavKey) {
        paymentSelections.remove(route)
    }

    fun openPaymentSelect(assetIds: List<AssetId>) = push(PaymentSelectRoute(assetIds))

    fun finishPaymentSelect(assetId: AssetId) = popWithResult(paymentSelections, assetId)

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
        push(importType.toImportRoute())
    }
    fun openWallet(walletId: WalletId) = push(WalletDetailsRoute(walletId))
    fun openWalletImage(walletId: WalletId) = push(WalletImageRoute(walletId))
    fun openWalletSecurityReminder(input: WalletSecretInput) = push(WalletSecurityReminderRoute(input))
    fun finishWalletSecurityReminder(input: WalletSecretInput) = replaceTop(WalletPhraseRoute(input))
    fun openAddAsset() = push(AddAssetRoute)
    fun openAsset(assetId: AssetId) = openAssetRoute(AssetRoute(assetId))
    fun openNetworkAssets(chain: Chain) = push(NetworkAssetsRoute(chain))
    fun openAssetChart(assetId: AssetId) = push(AssetChartRoute(assetId))
    fun openPortfolioChart(type: PortfolioType = PortfolioType.Wallet) = push(PortfolioChartRoute(type))
    fun openTransaction(transactionId: TransactionId) = push(TransactionDetailsRoute(transactionId))
    fun openAddress(chainAddress: ChainAddress) = push(AddressDetailsRoute(chainAddress))
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
        val action = runCatching { deeplinkService.urlAction(url) }.getOrNull() ?: return false
        return openUrlAction(action)
    }

    fun openUrlAction(action: UrlAction): Boolean {
        val deeplink = (action as? UrlAction.Deeplink)?.deeplink ?: return false
        val origin = backStack.lastOrNull()
        scope.launch {
            runCatchingCancellable { withContext(Dispatchers.IO) { navigationService.openDeeplink(deeplink) } }
                .onSuccess { target ->
                    selectTab(target.tab())
                    target.routes().forEach(::push)
                }
                .onFailure { error ->
                    Log.e(TAG, "opening a deep link failed", error)
                    origin?.let { routeMessages[it] = RouteMessage.Error(error.errorText()) }
                }
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
    fun openEarn(assetId: AssetId) = push(EarnRoute(assetId))

    fun openStake(assetId: AssetId) = push(StakeRoute(assetId))
    fun openDelegation(validatorId: String, delegationId: String) = push(DelegationRoute(validatorId, delegationId))
    fun openReceive() = push(ReceiveSelectRoute)
    fun openReceive(assetId: AssetId) = push(ReceiveRoute(assetId))
    fun openReceiveCollection() = push(ReceiveCollectionRoute)
    fun openRecipient(payment: GemPaymentRecipient? = null, chains: List<Chain> = emptyList()) = push(SendSelectRoute(payment, chains))
    fun openRecipient(assetId: AssetId, payment: GemPaymentRecipient? = null) = push(RecipientInputRoute(assetId, payment = payment))
    fun openNftRecipient(nft: NFTAsset) = push(RecipientInputRoute(AssetId(nft.chain), nft = nft))
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
        swapSelections[SwapRoute] = SwapSelection(itemType = SwapItemType.Receive, assetId = assetId)
        push(SwapRoute)
    }
    fun openSwapSelect(itemType: SwapItemType, payAssetId: AssetId?, receiveAssetId: AssetId?) {
        push(SwapSelectRoute(itemType, payAssetId, receiveAssetId))
    }
    fun finishSwapSelect(itemType: SwapItemType, assetId: AssetId) = popWithResult(swapSelections, SwapSelection(itemType = itemType, assetId = assetId))
    private fun clearSwapSelections() = swapSelections.clear()
    fun openBuy() = push(FiatSelectRoute)
    fun openBuy(assetId: AssetId) = openBuy(assetId, amount = null)
    fun openBuy(assetId: AssetId, amount: Int?) = push(FiatInputRoute(assetId, amount, FiatQuoteType.Buy))
    fun openAcquireAsset(action: AcquireAssetAction, assetId: AssetId) {
        when (action) {
            is AcquireAssetAction.Buy -> openBuy(assetId, amount = action.amount)
            is AcquireAssetAction.Swap -> action.payAssetId?.let { openSwap(from = it, to = assetId) } ?: openSwapTo(assetId)
            AcquireAssetAction.Receive -> openReceive(assetId)
        }
    }
    fun openFiatTransactions() = push(FiatTransactionsRoute)
    fun openConfirm(input: ConfirmTransferInput) {
        val pack = input.pack() ?: return
        push(ConfirmRoute(pack))
    }
    fun replaceWithConfirm(input: ConfirmTransferInput) {
        val pack = input.pack() ?: return
        replaceTop(ConfirmRoute(pack))
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
            },
        )
    }

    internal fun openPendingNavigation(routes: List<NavKey>, tab: GemNavigationTab? = null): Boolean {
        if (routes.isEmpty()) return false
        if (backStack.firstOrNull() != WalletRootRoute) return false
        resetToWallet()
        selectTab(tab)
        routes.forEach(::push)
        return true
    }

    private fun selectTab(tab: GemNavigationTab?) {
        currentTab.value = when (tab) {
            GemNavigationTab.WALLET -> assetsRoute
            GemNavigationTab.SETTINGS -> settingsRoute
            null -> return
        }
    }

    fun popConfirmFlow(toast: String? = null) {
        val index = backStack.indexOfLast { !it.isConfirmFlowSegmentRoute() }
        toast?.let { message -> backStack.getOrNull(index)?.let { target -> routeMessages[target] = RouteMessage.Toast(message) } }
        popFrom(index + 1)
    }

    private fun popFrom(index: Int) {
        while (backStack.lastIndex >= index && backStack.size > 1) {
            backStack.removeLastOrNull()
        }
    }

    private fun clearTransientState() {
        clearSwapSelections()
        paymentSelections.clear()
        routeMessages.clear()
    }

    private companion object {
        const val TAG = "WalletNavigator"
    }
}

internal fun NavKey.isConfirmFlowSegmentRoute(): Boolean = when (this) {
    SwapRoute -> true

    is SendSelectRoute,
    is AmountRoute,
    is ConfirmRoute,
    is DelegationRoute,
    is RecipientInputRoute,
    is EarnRoute,
    is StakeRoute,
    is SwapPairRoute,
    is SwapSelectRoute,
    is PaymentSelectRoute,
    is PaymentVerificationRoute,
    -> true

    else -> false
}

private fun ImportType.toImportRoute(): NavKey = when (val chain = chain) {
    null -> ImportMulticoinWalletRoute
    else -> ImportChainWalletRoute(kind, chain)
}
