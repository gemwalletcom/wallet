package com.gemwallet.android.ui.navigation.routes

import androidx.compose.runtime.Composable
import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.settings.aboutus.presents.AboutUsScreen
import com.gemwallet.android.features.settings.currency.presents.CurrenciesScene
import com.gemwallet.android.features.settings.develop.presents.DevelopScene
import com.gemwallet.android.features.settings.networks.presents.NetworksScreen
import com.gemwallet.android.features.settings.price_alerts.presents.PriceAlertTargetNavScreen
import com.gemwallet.android.features.settings.price_alerts.presents.PriceAlertsNavScreen
import com.gemwallet.android.features.settings.security.presents.SecurityScene
import com.gemwallet.android.features.settings.settings.presents.views.NotificationsScene
import com.gemwallet.android.features.settings.settings.presents.views.PreferencesScene
import com.gemwallet.android.features.settings.settings.presents.views.SupportChatScreen
import com.gemwallet.android.ui.navigation.assetIdArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.wallet.core.primitives.AssetId
import kotlinx.serialization.Serializable

const val settingsRoute = "settings"

@Serializable
data object CurrenciesRoute : NavKey

@Serializable
data object SecurityRoute : NavKey

@Serializable
data object DevelopRoute : NavKey

@Serializable
data object AboutusRoute : NavKey

@Serializable
data object NetworksRoute : NavKey

@Serializable
data object PriceAlertsRoute : NavKey

@Serializable
data class AssetPriceAlertsRoute(val assetId: AssetId) : NavKey

@Serializable
data class AddPriceAlertTargetRoute(val assetId: AssetId) : NavKey

@Serializable
data object SupportRoute : NavKey

@Serializable
data object PreferencesRoute : NavKey

@Serializable
data object NotificationsRoute : NavKey

fun EntryProviderScope<NavKey>.settingsScreen(
    onCurrencies: () -> Unit,
    onNetworks: () -> Unit,
    onPriceAlerts: () -> Unit,
    onAddPriceAlertTarget: (AssetId) -> Unit,
    onPriceAlertTargetComplete: (String) -> Unit,
    onChart: (AssetId) -> Unit,
    toastMessage: (NavKey) -> String?,
    onToastShown: (NavKey) -> Unit,
    onCancel: () -> Unit,
) {
    entry<CurrenciesRoute> {
        CurrenciesScene(onCancel = onCancel)
    }

    entry<SecurityRoute> {
        SecurityScene(onCancel = onCancel)
    }

    entry<DevelopRoute> {
        DevelopScene(onCancel = onCancel)
    }

    entry<AboutusRoute> {
        AboutUsScreen(onCancel = onCancel)
    }

    entry<NetworksRoute> {
        NetworksScreen(onCancel = onCancel)
    }

    entry<PriceAlertsRoute> { key ->
        priceAlertsScreenContent(
            toastMessage = toastMessage(key),
            onToastShown = { onToastShown(key) },
            onChart = onChart,
            onAddPriceAlertTarget = onAddPriceAlertTarget,
            onCancel = onCancel,
        )
    }

    entry<AssetPriceAlertsRoute>(
        metadata = { key -> routeArguments(assetIdArgument(key.assetId)) },
    ) { key ->
        priceAlertsScreenContent(
            toastMessage = toastMessage(key),
            onToastShown = { onToastShown(key) },
            onChart = onChart,
            onAddPriceAlertTarget = onAddPriceAlertTarget,
            onCancel = onCancel,
        )
    }

    entry<AddPriceAlertTargetRoute>(
        metadata = { key -> routeArguments(assetIdArgument(key.assetId)) },
    ) {
        PriceAlertTargetNavScreen(onCancel = onCancel, onComplete = onPriceAlertTargetComplete)
    }

    entry<NotificationsRoute> {
        NotificationsScene(
            onPriceAlerts = onPriceAlerts,
            onCancel = onCancel,
        )
    }

    entry<PreferencesRoute> {
        PreferencesScene(
            onNetworks = onNetworks,
            onCurrencies = onCurrencies,
            onCancel = onCancel,
        )
    }

    entry<SupportRoute> {
        SupportChatScreen(onCancel = onCancel)
    }
}

@Composable
private fun priceAlertsScreenContent(
    toastMessage: String?,
    onToastShown: () -> Unit,
    onChart: (AssetId) -> Unit,
    onAddPriceAlertTarget: (AssetId) -> Unit,
    onCancel: () -> Unit,
) {
    PriceAlertsNavScreen(
        toastMessage = toastMessage,
        onToastShown = onToastShown,
        onChart = onChart,
        onAddPriceAlertTarget = onAddPriceAlertTarget,
        onCancel = onCancel,
    )
}
