package com.gemwallet.android.ui.navigation

import androidx.compose.runtime.mutableStateOf
import androidx.navigation3.runtime.NavBackStack
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.create_wallet.navigation.CreateWalletAlertRoute
import com.gemwallet.android.features.create_wallet.navigation.CreateWalletRoute
import com.gemwallet.android.features.import_wallet.navigation.ImportChainWalletRoute
import com.gemwallet.android.features.import_wallet.navigation.ImportMulticoinWalletRoute
import com.gemwallet.android.features.import_wallet.navigation.ImportSelectTypeRoute
import com.gemwallet.android.features.onboarding.AcceptTermsDestination
import com.gemwallet.android.features.onboarding.AcceptTermsRoute
import com.gemwallet.android.features.onboarding.OnboardingRoute
import com.gemwallet.android.features.setup_wallet.navigation.SetupWalletRoute
import com.gemwallet.android.features.swap.viewmodels.models.SwapItemType
import com.gemwallet.android.model.ImportType
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockWalletId
import com.gemwallet.android.ui.navigation.routes.AddPriceAlertTargetRoute
import com.gemwallet.android.ui.navigation.routes.AmountRoute
import com.gemwallet.android.ui.navigation.routes.AssetChartRoute
import com.gemwallet.android.ui.navigation.routes.AssetPriceAlertsRoute
import com.gemwallet.android.ui.navigation.routes.AssetRoute
import com.gemwallet.android.ui.navigation.routes.assetsRoute
import com.gemwallet.android.ui.navigation.routes.ConfirmRoute
import com.gemwallet.android.ui.navigation.routes.FiatInputRoute
import com.gemwallet.android.ui.navigation.routes.FiatSelectRoute
import com.gemwallet.android.ui.navigation.routes.NftAssetRoute
import com.gemwallet.android.ui.navigation.routes.NftCollectionRoute
import com.gemwallet.android.ui.navigation.routes.PerpetualAmountRoute
import com.gemwallet.android.ui.navigation.routes.PriceAlertsRoute
import com.gemwallet.android.ui.navigation.routes.RecipientInputRoute
import com.gemwallet.android.ui.navigation.routes.ReceiveRoute
import com.gemwallet.android.ui.navigation.routes.ReceiveSelectRoute
import com.gemwallet.android.ui.navigation.routes.SendSelectRoute
import com.gemwallet.android.ui.navigation.routes.StakeRoute
import com.gemwallet.android.ui.navigation.routes.SwapPairRoute
import com.gemwallet.android.ui.navigation.routes.SwapRoute
import com.gemwallet.android.ui.navigation.routes.SwapSelectRoute
import com.gemwallet.android.ui.navigation.routes.WalletsRoute
import com.gemwallet.android.ui.navigation.routes.WalletDetailsRoute
import com.gemwallet.android.ui.navigation.routes.WalletPhraseRoute
import com.gemwallet.android.ui.navigation.routes.WalletSecurityReminderRoute
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletType
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class WalletNavigatorTest {

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
    fun openSetupWallet_replacesCreateOrImportRoute() {
        val walletId = mockWalletId("wallet-1")
        val createNavigator = navigatorWith(OnboardingRoute, CreateWalletRoute)
        val importNavigator = navigatorWith(OnboardingRoute, ImportMulticoinWalletRoute)

        createNavigator.openSetupWallet(walletId)
        importNavigator.openSetupWallet(walletId)

        assertEquals(listOf(OnboardingRoute, SetupWalletRoute(walletId)), createNavigator.backStack.toList())
        assertEquals(listOf(OnboardingRoute, SetupWalletRoute(walletId)), importNavigator.backStack.toList())
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
    fun openPendingNavigation_waitsWhenWalletRootIsUnavailable() {
        val navigator = navigatorWith(OnboardingRoute)

        val opened = navigator.openPendingNavigation(listOf(AssetRoute(mockAssetId(Chain.Solana))))

        assertFalse(opened)
        assertEquals(listOf(OnboardingRoute), navigator.backStack.toList())
    }

    @Test
    fun openPendingNavigation_requiresConfirmationDuringActiveTransactionFlow() {
        val assetId = mockAssetId(Chain.Solana)
        val navigator = navigatorWith(
            WalletRootRoute,
            AssetRoute(assetId),
            AmountRoute("amount"),
        )

        val opened = navigator.openPendingNavigation(listOf(AssetRoute(mockAssetId(Chain.Ethereum))))

        assertFalse(opened)
        assertTrue(navigator.needsPendingNavigationConfirmation())
        assertEquals(
            listOf(
                WalletRootRoute,
                AssetRoute(assetId),
                AmountRoute("amount"),
            ),
            navigator.backStack.toList(),
        )
    }

    @Test
    fun openPendingNavigation_requiresConfirmationDuringSecretPhraseFlow() {
        val walletId = mockWalletId("wallet-1")
        val navigator = navigatorWith(
            WalletRootRoute,
            WalletDetailsRoute(walletId),
            WalletPhraseRoute(walletId, WalletType.Multicoin),
        )

        val opened = navigator.openPendingNavigation(listOf(AssetRoute(mockAssetId(Chain.Solana))))

        assertFalse(opened)
        assertTrue(navigator.needsPendingNavigationConfirmation())
        assertEquals(
            listOf(
                WalletRootRoute,
                WalletDetailsRoute(walletId),
                WalletPhraseRoute(walletId, WalletType.Multicoin),
            ),
            navigator.backStack.toList(),
        )
    }

    @Test
    fun finishWalletSecurityReminder_replacesReminderWithPhraseRoute() {
        val walletId = mockWalletId("wallet-1")
        val navigator = navigatorWith(
            WalletRootRoute,
            WalletDetailsRoute(walletId),
            WalletSecurityReminderRoute(walletId, WalletType.Multicoin),
        )

        navigator.finishWalletSecurityReminder(walletId, WalletType.Multicoin)

        assertEquals(
            listOf(
                WalletRootRoute,
                WalletDetailsRoute(walletId),
                WalletPhraseRoute(walletId, WalletType.Multicoin),
            ),
            navigator.backStack.toList(),
        )
    }

    @Test
    fun confirmedPendingNavigation_resetsActiveFlow() {
        val navigator = navigatorWith(
            WalletRootRoute,
            AmountRoute("amount"),
        )
        val route = AssetRoute(mockAssetId(Chain.Solana))

        val opened = navigator.openPendingNavigation(listOf(route), confirmed = true)

        assertTrue(opened)
        assertEquals(listOf(WalletRootRoute, route), navigator.backStack.toList())
    }

    @Test
    fun dropNonRestorableRoutes_removesSensitiveAndInFlightRoutes() {
        val assetId = mockAssetId(Chain.Solana)
        val walletId = mockWalletId("wallet-1")

        val restored = listOf<NavKey>(
            WalletRootRoute,
            AssetRoute(assetId),
            WalletSecurityReminderRoute(walletId, WalletType.Multicoin),
            WalletPhraseRoute(walletId, WalletType.Multicoin),
            RecipientInputRoute(assetId, nftAssetId = null),
            AmountRoute("amount"),
            PerpetualAmountRoute("perpetual"),
            ConfirmRoute("confirm"),
        ).dropNonRestorableRoutes(WalletRootRoute)

        assertEquals(listOf(WalletRootRoute, AssetRoute(assetId)), restored)
    }

    @Test
    fun openImportWallet_usesValidTypedRoutes() {
        val navigator = navigatorWith(OnboardingRoute)

        navigator.openImportWallet()
        navigator.openImportWallet(ImportType(WalletType.Multicoin))
        navigator.openImportWallet(ImportType(WalletType.PrivateKey, Chain.Solana))
        navigator.openImportWallet(ImportType(WalletType.Single))

        assertEquals(
            listOf(
                OnboardingRoute,
                ImportSelectTypeRoute,
                ImportMulticoinWalletRoute,
                ImportChainWalletRoute(WalletType.PrivateKey, Chain.Solana),
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
        navigator.openNftRecipient(assetId, "nft-1")

        assertEquals(
            listOf(
                WalletRootRoute,
                SendSelectRoute,
                RecipientInputRoute(assetId, nftAssetId = null),
                RecipientInputRoute(assetId, nftAssetId = "nft-1"),
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
        navigator.openNftAsset("ethereum_0xcollection::1")

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
    fun finishSwapSelect_popsSelectorAndStoresSelectionForTargetRoute() {
        val route = SwapPairRoute(mockAssetId(Chain.Bitcoin), to = null)
        val selectedPayAssetId = mockAssetId(Chain.Solana)
        val selectedReceiveAssetId = mockAssetId(Chain.Ethereum)
        val navigator = navigatorWith(
            WalletRootRoute,
            route,
            SwapSelectRoute(SwapItemType.Pay, payAssetId = null, receiveAssetId = null),
        )

        navigator.finishSwapSelect(
            itemType = SwapItemType.Pay,
            payAssetId = selectedPayAssetId,
            receiveAssetId = selectedReceiveAssetId,
        )

        assertEquals(listOf(WalletRootRoute, route), navigator.backStack.toList())
        assertEquals(
            SwapSelection(
                SwapItemType.Pay,
                payAssetId = selectedPayAssetId,
                receiveAssetId = selectedReceiveAssetId,
            ),
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
            RecipientInputRoute(assetId, nftAssetId = null),
            AmountRoute("amount"),
            ConfirmRoute("confirm"),
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
            ConfirmRoute("confirm"),
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
    fun popConfirmFlow_popsSwapFlowToAsset() {
        val assetId = mockAssetId(Chain.Ethereum)
        val navigator = navigatorWith(
            WalletRootRoute,
            WalletsRoute,
            AssetRoute(assetId),
            SwapPairRoute(assetId, to = null),
            ConfirmRoute("confirm"),
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
            ConfirmRoute("confirm"),
        )

        navigator.popConfirmFlow()

        assertEquals(listOf(WalletRootRoute), navigator.backStack.toList())
    }

    @Test
    fun popWithToast_scopesMessageToPreviousRoute() {
        val assetId = mockAssetId(Chain.Solana)
        val target = AssetPriceAlertsRoute(assetId)
        val otherTarget = AssetChartRoute(assetId)
        val navigator = navigatorWith(
            WalletRootRoute,
            target,
            AddPriceAlertTargetRoute(assetId),
        )

        navigator.popWithToast("Created")

        assertEquals(listOf(WalletRootRoute, target), navigator.backStack.toList())
        assertEquals("Created", navigator.toastMessage(target))
        assertNull(navigator.toastMessage(otherTarget))

        navigator.clearToastMessage(target)

        assertNull(navigator.toastMessage(target))
    }

    private fun navigatorWith(vararg routes: NavKey): WalletNavigator {
        return WalletNavigator(
            backStack = NavBackStack(*routes),
            currentTab = mutableStateOf(assetsRoute),
        )
    }
}
