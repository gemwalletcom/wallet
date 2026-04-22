package com.gemwallet.android.ui.navigation.routes

import androidx.navigation.NavController
import androidx.navigation.NavGraphBuilder
import androidx.navigation.NavOptions
import androidx.navigation.compose.composable
import androidx.navigation.navOptions
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.asset_select.presents.views.SelectReceiveScreen
import com.gemwallet.android.features.receive.presents.ReceiveNftChainsScreen
import com.gemwallet.android.features.receive.presents.ReceiveScreen
import com.wallet.core.primitives.AssetId
import kotlinx.serialization.Serializable

@Serializable
class ReceiveRoute(val assetId: String)

@Serializable
class ReceiveSelectRoute

@Serializable
class ReceiveNftChainsRoute

fun NavController.navigateToReceiveScreen(assetId: AssetId? = null, navOptions: NavOptions? = null) {
    if (assetId == null) {
        navigate(ReceiveSelectRoute(), navOptions ?: navOptions { launchSingleTop = true })
    } else {
        navigate(ReceiveRoute(assetId.toIdentifier()), navOptions ?: navOptions { launchSingleTop = true })
    }

}

fun NavController.navigateToReceiveNftChains(navOptions: NavOptions? = null) {
    navigate(ReceiveNftChainsRoute(), navOptions ?: navOptions { launchSingleTop = true })
}

fun NavGraphBuilder.receiveScreen(
    onCancel: () -> Unit,
    onReceive: (AssetId) -> Unit,
) {
    composable<ReceiveRoute> {
        ReceiveScreen(onCancel = onCancel)
    }

    composable<ReceiveSelectRoute> {
        SelectReceiveScreen(
            onCancel = onCancel,
            onSelect = { onReceive(it) }
        )
    }

    composable<ReceiveNftChainsRoute> {
        ReceiveNftChainsScreen(
            onCancel = onCancel,
            onSelect = { onReceive(AssetId(it)) },
        )
    }
}
