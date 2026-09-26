package com.gemwallet.android.ui.navigation

import android.util.Log
import androidx.compose.runtime.mutableStateOf
import androidx.navigation3.runtime.NavBackStack
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.domains.wallet.WalletSecretInput
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.onboarding.presents.create_wallet.CreateWalletAlertRoute
import com.gemwallet.android.features.onboarding.presents.create_wallet.CreateWalletRoute
import com.gemwallet.android.features.onboarding.presents.import_wallet.ImportChainWalletRoute
import com.gemwallet.android.features.onboarding.presents.import_wallet.ImportMulticoinWalletRoute
import com.gemwallet.android.features.onboarding.presents.import_wallet.ImportSelectTypeRoute
import com.gemwallet.android.features.onboarding.presents.terms.AcceptTermsDestination
import com.gemwallet.android.features.onboarding.presents.terms.AcceptTermsRoute
import com.gemwallet.android.model.ImportType
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockNftAsset
import com.gemwallet.android.ui.models.navigation.RouteMessage
import com.gemwallet.android.ui.navigation.routes.AddPriceAlertTargetRoute
import com.gemwallet.android.ui.navigation.routes.AmountRoute
import com.gemwallet.android.ui.navigation.routes.AssetPriceAlertsRoute
import com.gemwallet.android.ui.navigation.routes.AssetRoute
import com.gemwallet.android.ui.navigation.routes.ChartRoute
import com.gemwallet.android.ui.navigation.routes.ConfirmTransferRoute
import com.gemwallet.android.ui.navigation.routes.DelegationRoute
import com.gemwallet.android.ui.navigation.routes.ExportWalletRoute
import com.gemwallet.android.ui.navigation.routes.FiatInputRoute
import com.gemwallet.android.ui.navigation.routes.FiatSelectRoute
import com.gemwallet.android.ui.navigation.routes.NftAssetRoute
import com.gemwallet.android.ui.navigation.routes.NftCollectionRoute
import com.gemwallet.android.ui.navigation.routes.PriceAlertsRoute
import com.gemwallet.android.ui.navigation.routes.ReceiveRoute
import com.gemwallet.android.ui.navigation.routes.ReceiveSelectRoute
import com.gemwallet.android.ui.navigation.routes.RecipientRoute
import com.gemwallet.android.ui.navigation.routes.ReferralRoute
import com.gemwallet.android.ui.navigation.routes.SecurityReminderRoute
import com.gemwallet.android.ui.navigation.routes.SendSelectRoute
import com.gemwallet.android.ui.navigation.routes.StakeRoute
import com.gemwallet.android.ui.navigation.routes.SupportRoute
import com.gemwallet.android.ui.navigation.routes.SwapPairRoute
import com.gemwallet.android.ui.navigation.routes.SwapRoute
import com.gemwallet.android.ui.navigation.routes.SwapSelectRoute
import com.gemwallet.android.ui.navigation.routes.WalletConnectRequestRoute
import com.gemwallet.android.ui.navigation.routes.WalletDetailRoute
import com.gemwallet.android.ui.navigation.routes.WalletsRoute
import com.gemwallet.android.ui.navigation.routes.assetsRoute
import com.gemwallet.android.ui.navigation.routes.settingsRoute
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NFTAssetId
import com.wallet.core.primitives.WalletId
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.job
import kotlinx.coroutines.joinAll
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemAssetsServiceInterface
import uniffi.gemstone.GemDeeplinkService
import uniffi.gemstone.GemNavigationServiceInterface
import uniffi.gemstone.GemNavigationTab
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemWalletImportKind
import uniffi.gemstone.GemWalletSecretKind

class WalletNavigatorTest {

    @Before
    fun setUp() {
        mockkStatic(Log::class)
        every { Log.e(any(), any(), any()) } returns 0
    }

    @After
    fun tearDown() {
        unmockkStatic(Log::class)
    }

    @Test
    fun openNotificationUrl_reportsTheFailureToTheScreenTheLinkWasTappedOn() = runTest {
        val error = GemServiceException.NotFound("asset not found")
        val navigationService = mockk<GemNavigationServiceInterface> {
            coEvery { openDeeplink(any()) } throws error
        }
        val navigator = navigatorWith(WalletRootRoute, SupportRoute, navigationService = navigationService, scope = this)

        val handled = navigator.openNotificationUrl("https://gemwallet.com/tokens/ethereum/0x1111111111111111111111111111111111111111/buy")
        coroutineContext.job.children.toList().joinAll()

        assertTrue(handled)
        assertEquals(listOf(WalletRootRoute, SupportRoute), navigator.backStack.toList())
        assertEquals(RouteMessage.Error(error.errorText()), navigator.routeMessage(SupportRoute))
        assertNull(navigator.routeMessage(WalletRootRoute))

        navigator.clearRouteMessage(SupportRoute)

        assertNull(navigator.routeMessage(SupportRoute))
    }

    @Test
    fun openAsset_leavesTheStackAloneWhenCoreFailsToOpenIt() = runTest {
        val assetId = mockAssetId(Chain.Tron)
        val assetsService = mockk<GemAssetsServiceInterface> {
            coEvery { openAsset(assetId.toIdentifier()) } throws GemServiceException.Api("offline")
        }
        val navigator = navigatorWith(WalletRootRoute, assetsService = assetsService, scope = this)

        navigator.openAsset(assetId).join()

        assertEquals(listOf(WalletRootRoute), navigator.backStack.toList())
    }

    @Test
    fun openAsset_pushesOnlyAssetsCoreOpens() = runTest {
        val blocked = mockAssetId(Chain.Tempo)
        val opened = mockAssetId(Chain.Tron)
        val callingThreads = mutableListOf<String>()
        val assetsService = mockk<GemAssetsServiceInterface> {
            coEvery { openAsset(blocked.toIdentifier()) } answers {
                callingThreads += Thread.currentThread().name
                null
            }
            coEvery { openAsset(opened.toIdentifier()) } answers {
                callingThreads += Thread.currentThread().name
                mockAsset(id = mockAssetId(chain = Chain.Tron)).toGem()
            }
        }
        val navigator = navigatorWith(WalletRootRoute, assetsService = assetsService, scope = this)

        navigator.openAsset(blocked).join()
        navigator.openAsset(opened).join()

        assertEquals(listOf(WalletRootRoute, AssetRoute(opened)), navigator.backStack.toList())
        assertTrue(
            "openAsset reads the current wallet through a synchronous store callback that blocks on Room; " +
                "calling it on the navigator scope lands on main. Got $callingThreads",
            callingThreads.size == 2 && callingThreads.all { it.startsWith("DefaultDispatcher-worker") },
        )
    }

    @Test
    fun finishAcceptTerms_replacesCreateTermsRoute() {
        val navigator = navigatorWith(OnboardingRoute, AcceptTermsRoute(AcceptTermsDestination.Create))

        navigator.finishAcceptTerms(AcceptTermsDestination.Create)

        assertEquals(listOf(OnboardingRoute, CreateWalletAlertRoute), navigator.backStack.toList())
    }

    @Test
    fun finishAcceptTerms_replacesImportTermsRoute() {
        val navigator = navigatorWith(OnboardingRoute, AcceptTermsRoute(AcceptTermsDestination.Import))

        navigator.finishAcceptTerms(AcceptTermsDestination.Import)

        assertEquals(listOf(OnboardingRoute, ImportSelectTypeRoute), navigator.backStack.toList())
    }

    @Test
    fun resetToWallet_landsOnTheWalletFromCreateOrImport() {
        val createNavigator = navigatorWith(OnboardingRoute, CreateWalletRoute)
        val importNavigator = navigatorWith(OnboardingRoute, ImportMulticoinWalletRoute)

        createNavigator.resetToWallet()
        importNavigator.resetToWallet()

        assertEquals(listOf(WalletRootRoute), createNavigator.backStack.toList())
        assertEquals(listOf(WalletRootRoute), importNavigator.backStack.toList())
    }

    @Test
    fun openPendingNavigation_resetsWalletStackAndOpensRoute() {
        val navigator = navigatorWith(WalletRootRoute, WalletsRoute)
        val route = AssetRoute(mockAssetId(Chain.Solana))

        val opened = navigator.openPendingNavigation(listOf(route))

        assertTrue(opened)
        assertEquals(listOf(WalletRootRoute, route), navigator.backStack.toList())
    }

    @Test
    fun openPendingNavigation_selectsTheTabCoreNamed() {
        val navigator = navigatorWith(WalletRootRoute)

        navigator.openPendingNavigation(listOf(ReferralRoute(code = null)), GemNavigationTab.SETTINGS)

        assertEquals(settingsRoute, navigator.currentTab.value)
    }

    @Test
    fun openPendingNavigation_waitsWhenWalletRootIsUnavailable() {
        val navigator = navigatorWith(OnboardingRoute)

        val opened = navigator.openPendingNavigation(listOf(AssetRoute(mockAssetId(Chain.Solana))))

        assertFalse(opened)
        assertEquals(listOf(OnboardingRoute), navigator.backStack.toList())
    }

    @Test
    fun openPendingNavigation_resetsActiveTransactionFlow() {
        val navigator = navigatorWith(
            WalletRootRoute,
            AssetRoute(mockAssetId(Chain.Solana)),
            AmountRoute("amount"),
        )
        val route = AssetRoute(mockAssetId(Chain.Ethereum))

        val opened = navigator.openPendingNavigation(listOf(route))

        assertTrue(opened)
        assertEquals(listOf(WalletRootRoute, route), navigator.backStack.toList())
    }

    @Test
    fun openPendingNavigation_resetsSecretPhraseFlow() {
        val walletId = WalletId("wallet-1")
        val navigator = navigatorWith(
            WalletRootRoute,
            WalletDetailRoute(walletId),
            ExportWalletRoute(WalletSecretInput(walletId, GemWalletSecretKind.PHRASE)),
        )
        val route = AssetRoute(mockAssetId(Chain.Solana))

        val opened = navigator.openPendingNavigation(listOf(route))

        assertTrue(opened)
        assertEquals(listOf(WalletRootRoute, route), navigator.backStack.toList())
    }

    @Test
    fun finishWalletSecurityReminder_replacesReminderWithPhraseRoute() {
        val walletId = WalletId("wallet-1")
        val navigator = navigatorWith(
            WalletRootRoute,
            WalletDetailRoute(walletId),
            SecurityReminderRoute(WalletSecretInput(walletId, GemWalletSecretKind.PHRASE)),
        )

        navigator.finishWalletSecurityReminder(WalletSecretInput(walletId, GemWalletSecretKind.PHRASE))

        assertEquals(
            listOf(
                WalletRootRoute,
                WalletDetailRoute(walletId),
                ExportWalletRoute(WalletSecretInput(walletId, GemWalletSecretKind.PHRASE)),
            ),
            navigator.backStack.toList(),
        )
    }

    @Test
    fun dropNonRestorableRoutes_keepsTheLiveRootWhenItDiffersFromTheStartDestination() {
        val assetId = mockAssetId(Chain.Solana)

        val restored = listOf<NavKey>(
            WalletRootRoute,
            AssetRoute(assetId),
        ).dropNonRestorableRoutes(OnboardingRoute)

        assertEquals(listOf(WalletRootRoute, AssetRoute(assetId)), restored)
    }

    @Test
    fun dropNonRestorableRoutes_keepsOnboardingRootAgainstAWalletStartDestination() {
        val restored = listOf<NavKey>(OnboardingRoute).dropNonRestorableRoutes(WalletRootRoute)

        assertEquals(listOf(OnboardingRoute), restored)
    }

    @Test
    fun dropNonRestorableRoutes_fallsBackToStartDestinationWhenTheRootIsNotARoot() {
        val assetId = mockAssetId(Chain.Solana)

        val restored = listOf<NavKey>(
            AssetRoute(assetId),
            AmountRoute("amount"),
        ).dropNonRestorableRoutes(WalletRootRoute)

        assertEquals(listOf(WalletRootRoute), restored)
    }

    @Test
    fun dropNonRestorableRoutes_removesSensitiveAndInFlightRoutes() {
        val assetId = mockAssetId(Chain.Solana)
        val walletId = WalletId("wallet-1")

        val restored = listOf<NavKey>(
            WalletRootRoute,
            AssetRoute(assetId),
            SecurityReminderRoute(WalletSecretInput(walletId, GemWalletSecretKind.PHRASE)),
            ExportWalletRoute(WalletSecretInput(walletId, GemWalletSecretKind.PHRASE)),
            CreateWalletRoute,
            RecipientRoute(assetId),
            AmountRoute("amount"),
            AmountRoute("perpetual"),
            ConfirmTransferRoute("confirm"),
        ).dropNonRestorableRoutes(WalletRootRoute)

        assertEquals(listOf(WalletRootRoute, AssetRoute(assetId)), restored)
    }

    @Test
    fun openImportWallet_usesValidTypedRoutes() {
        val navigator = navigatorWith(OnboardingRoute)

        navigator.openImportWallet()
        navigator.openImportWallet(ImportType(GemWalletImportKind.PHRASE))
        navigator.openImportWallet(ImportType(GemWalletImportKind.PRIVATE_KEY, Chain.Solana))

        assertEquals(
            listOf(
                OnboardingRoute,
                ImportSelectTypeRoute,
                ImportMulticoinWalletRoute,
                ImportChainWalletRoute(GemWalletImportKind.PRIVATE_KEY, Chain.Solana),
            ),
            navigator.backStack.toList(),
        )
    }

    @Test
    fun openRecipient_usesExplicitRoutes() {
        val navigator = navigatorWith(WalletRootRoute)
        val assetId = mockAssetId(Chain.Ethereum)

        navigator.openRecipient()
        navigator.openRecipient(assetId)
        val nft = mockNftAsset(chain = Chain.Ethereum)
        navigator.openNftRecipient(nft)

        assertEquals(
            listOf(
                WalletRootRoute,
                SendSelectRoute(),
                RecipientRoute(assetId),
                RecipientRoute(assetId, nft = nft),
            ),
            navigator.backStack.toList(),
        )
    }

    @Test
    fun openAssetActions_useExplicitRoutes() {
        val navigator = navigatorWith(WalletRootRoute)
        val assetId = mockAssetId(Chain.Ethereum)

        navigator.openReceive()
        navigator.openReceive(assetId)
        navigator.openBuy()
        navigator.openBuy(assetId)

        assertEquals(
            listOf(
                WalletRootRoute,
                ReceiveSelectRoute,
                ReceiveRoute(assetId),
                FiatSelectRoute,
                FiatInputRoute(assetId),
            ),
            navigator.backStack.toList(),
        )
    }

    @Test
    fun openPriceAlerts_usesExplicitRoutes() {
        val navigator = navigatorWith(WalletRootRoute)
        val assetId = mockAssetId(Chain.Ethereum)

        navigator.openPriceAlerts()
        navigator.openPriceAlerts(assetId)

        assertEquals(
            listOf(
                WalletRootRoute,
                PriceAlertsRoute,
                AssetPriceAlertsRoute(assetId),
            ),
            navigator.backStack.toList(),
        )
    }

    @Test
    fun openNft_usesExplicitRoutes() {
        val navigator = navigatorWith(WalletRootRoute)

        navigator.openNftCollection("ethereum_0xcollection")
        navigator.openNftAsset(NFTAssetId(Chain.Ethereum, "0xcollection", "1"))

        assertEquals(
            listOf(
                WalletRootRoute,
                NftCollectionRoute("ethereum_0xcollection"),
                NftAssetRoute("ethereum_0xcollection::1"),
            ),
            navigator.backStack.toList(),
        )
    }

    @Test
    fun openSwap_usesExplicitRoutes() {
        val navigator = navigatorWith(WalletRootRoute)
        val payAssetId = mockAssetId(Chain.Solana)
        val receiveAssetId = mockAssetId(Chain.Ethereum)

        navigator.openSwap()
        navigator.openSwap(payAssetId)
        navigator.openSwap(payAssetId, receiveAssetId)

        assertEquals(
            listOf(
                WalletRootRoute,
                SwapRoute,
                SwapPairRoute(payAssetId, to = null),
                SwapPairRoute(payAssetId, receiveAssetId),
            ),
            navigator.backStack.toList(),
        )
    }

    @Test
    fun openSwapTo_opensSwapWithReceiveAssetSelection() {
        val navigator = navigatorWith(WalletRootRoute)
        val receiveAssetId = mockAssetId(Chain.Tron)

        navigator.openSwapTo(receiveAssetId)

        assertEquals(listOf(WalletRootRoute, SwapRoute), navigator.backStack.toList())
        assertEquals(
            SwapSelection(itemType = SwapItemType.Receive, assetId = receiveAssetId),
            navigator.swapSelection(SwapRoute),
        )
    }

    @Test
    fun finishSwapSelect_popsSelectorAndStoresSelectionForTargetRoute() {
        val route = SwapPairRoute(mockAssetId(Chain.Bitcoin), to = null)
        val selectedPayAssetId = mockAssetId(Chain.Solana)
        val navigator = navigatorWith(
            WalletRootRoute,
            route,
            SwapSelectRoute(SwapItemType.Pay, payAssetId = null, receiveAssetId = null),
        )

        navigator.finishSwapSelect(itemType = SwapItemType.Pay, assetId = selectedPayAssetId)

        assertEquals(listOf(WalletRootRoute, route), navigator.backStack.toList())
        assertEquals(
            SwapSelection(SwapItemType.Pay, assetId = selectedPayAssetId),
            navigator.swapSelection(route),
        )
        assertNull(navigator.swapSelection(SwapRoute))

        navigator.clearSwapSelection(route)

        assertNull(navigator.swapSelection(route))
    }

    @Test
    fun popConfirmFlow_popsTransferFlowToAsset() {
        val assetId = mockAssetId(Chain.Solana)
        val navigator = navigatorWith(
            WalletRootRoute,
            WalletsRoute,
            AssetRoute(assetId),
            RecipientRoute(assetId),
            AmountRoute("amount"),
            ConfirmTransferRoute("confirm"),
        )

        navigator.popConfirmFlow()

        assertEquals(
            listOf(
                WalletRootRoute,
                WalletsRoute,
                AssetRoute(assetId),
            ),
            navigator.backStack.toList(),
        )
    }

    @Test
    fun popConfirmFlow_popsStakeFlowToLaunchingAsset() {
        val previousAssetId = mockAssetId(Chain.Ethereum)
        val stakeAssetId = mockAssetId(Chain.Solana)
        val navigator = navigatorWith(
            WalletRootRoute,
            AssetRoute(previousAssetId),
            AssetRoute(stakeAssetId),
            StakeRoute(stakeAssetId),
            AmountRoute("amount"),
            ConfirmTransferRoute("confirm"),
        )

        navigator.popConfirmFlow()

        assertEquals(
            listOf(
                WalletRootRoute,
                AssetRoute(previousAssetId),
                AssetRoute(stakeAssetId),
            ),
            navigator.backStack.toList(),
        )
    }

    @Test
    fun popConfirmFlow_popsDelegationStakeFlowToLaunchingAsset() {
        val assetId = mockAssetId(Chain.Solana)
        val navigator = navigatorWith(
            WalletRootRoute,
            AssetRoute(assetId),
            StakeRoute(assetId),
            DelegationRoute(validatorId = "validator", delegationId = "delegation"),
            AmountRoute("amount"),
            ConfirmTransferRoute("confirm"),
        )

        navigator.popConfirmFlow()

        assertEquals(
            listOf(
                WalletRootRoute,
                AssetRoute(assetId),
            ),
            navigator.backStack.toList(),
        )
    }

    @Test
    fun popConfirmFlow_popsSwapFlowToAsset() {
        val assetId = mockAssetId(Chain.Ethereum)
        val navigator = navigatorWith(
            WalletRootRoute,
            WalletsRoute,
            AssetRoute(assetId),
            SwapPairRoute(assetId, to = null),
            ConfirmTransferRoute("confirm"),
        )

        navigator.popConfirmFlow()

        assertEquals(
            listOf(
                WalletRootRoute,
                WalletsRoute,
                AssetRoute(assetId),
            ),
            navigator.backStack.toList(),
        )
    }

    @Test
    fun popConfirmFlow_popsToRootWhenNoAssetUnderneath() {
        val navigator = navigatorWith(
            WalletRootRoute,
            ConfirmTransferRoute("confirm"),
        )

        navigator.popConfirmFlow()

        assertEquals(listOf(WalletRootRoute), navigator.backStack.toList())
    }

    @Test
    fun popWithToast_scopesMessageToPreviousRoute() {
        val assetId = mockAssetId(Chain.Solana)
        val target = AssetPriceAlertsRoute(assetId)
        val otherTarget = ChartRoute(assetId)
        val navigator = navigatorWith(
            WalletRootRoute,
            target,
            AddPriceAlertTargetRoute(assetId),
        )

        navigator.popWithToast("Created")

        assertEquals(listOf(WalletRootRoute, target), navigator.backStack.toList())
        assertEquals(RouteMessage.Toast("Created"), navigator.routeMessage(target))
        assertNull(navigator.routeMessage(otherTarget))

        navigator.clearRouteMessage(target)

        assertNull(navigator.routeMessage(target))
    }

    @Test
    fun showWalletConnectRequest_pushesOneRouteAndReplacesItForTheNextRequest() {
        val navigator = navigatorWith(WalletRootRoute)

        navigator.showWalletConnectRequest("request/topic/1")
        navigator.showWalletConnectRequest("request/topic/1")
        assertEquals(listOf(WalletRootRoute, WalletConnectRequestRoute("request/topic/1")), navigator.backStack.toList())

        navigator.showWalletConnectRequest("request/topic/2")
        assertEquals(listOf(WalletRootRoute, WalletConnectRequestRoute("request/topic/2")), navigator.backStack.toList())
    }

    @Test
    fun showWalletConnectRequest_removesTheRouteUnderneathAScreenItOpened() {
        val navigator = navigatorWith(WalletRootRoute, WalletConnectRequestRoute("request/topic/1"), ReceiveRoute(mockAssetId(Chain.Tron)))

        navigator.showWalletConnectRequest(null)

        assertEquals(listOf(WalletRootRoute, ReceiveRoute(mockAssetId(Chain.Tron))), navigator.backStack.toList())
    }

    private fun navigatorWith(
        vararg routes: NavKey,
        assetsService: GemAssetsServiceInterface = mockk(),
        navigationService: GemNavigationServiceInterface = mockk(relaxed = true),
        scope: CoroutineScope = CoroutineScope(Dispatchers.Unconfined),
    ): WalletNavigator = WalletNavigator(
        backStack = NavBackStack(*routes),
        currentTab = mutableStateOf(assetsRoute),
        deeplinkService = GemDeeplinkService(),
        assetsService = assetsService,
        navigationService = navigationService,
        scope = scope,
    )
}
