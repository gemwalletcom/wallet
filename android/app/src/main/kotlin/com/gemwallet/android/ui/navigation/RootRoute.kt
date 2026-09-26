package com.gemwallet.android.ui.navigation

import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.key
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.model.Session
import com.gemwallet.android.ui.LocalAssetsService
import com.gemwallet.android.ui.LocalDeeplinkService
import com.gemwallet.android.ui.LocalNavigationService
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.StateFlow
import kotlinx.serialization.Serializable

@Serializable
data object WalletRootRoute : NavKey

data class SwapSelection(val itemType: SwapItemType, val assetId: AssetId)

@Composable
fun rememberWalletNavigationState(startDestination: NavKey, currentTab: MutableState<String>, session: StateFlow<Session?>): WalletNavigator {
    val assetsService = LocalAssetsService.current
    val deeplinkService = LocalDeeplinkService.current
    val navigationService = LocalNavigationService.current
    val scope = rememberCoroutineScope()
    return key(startDestination) {
        val backStack = rememberWalletNavBackStack(startDestination)
        remember(backStack, currentTab, deeplinkService, assetsService, navigationService, session, scope) {
            WalletNavigator(
                backStack = backStack,
                currentTab = currentTab,
                deeplinkService = deeplinkService,
                assetsService = assetsService,
                navigationService = navigationService,
                session = session,
                scope = scope,
            )
        }
    }
}
