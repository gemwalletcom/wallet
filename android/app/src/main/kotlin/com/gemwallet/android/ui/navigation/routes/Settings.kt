package com.gemwallet.android.ui.navigation.routes

import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.platform.UriHandler
import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.in_app_notifications.presents.InAppNotificationsAction
import com.gemwallet.android.features.in_app_notifications.presents.InAppNotificationsScreen
import com.gemwallet.android.features.price_alerts.presents.AssetPriceAlertsScreen
import com.gemwallet.android.features.price_alerts.presents.PriceAlertsScreen
import com.gemwallet.android.features.price_alerts.presents.SetPriceAlertScreen
import com.gemwallet.android.features.settings.presents.NotificationsScreen
import com.gemwallet.android.features.settings.presents.PreferencesScreen
import com.gemwallet.android.features.settings.presents.about_us.AboutUsScreen
import com.gemwallet.android.features.settings.presents.chain_settings.ChainListSettingsScreen
import com.gemwallet.android.features.settings.presents.chain_settings.ChainSettingsScreen
import com.gemwallet.android.features.settings.presents.chain_settings.ServiceStatusScreen
import com.gemwallet.android.features.settings.presents.currency.CurrencyScreen
import com.gemwallet.android.features.settings.presents.developer.DeveloperPaymentsScene
import com.gemwallet.android.features.settings.presents.developer.DeveloperScreen
import com.gemwallet.android.features.settings.presents.security.SecurityScreen
import com.gemwallet.android.features.settings.viewmodels.models.PreferencesAction
import com.gemwallet.android.features.support.presents.SupportChatScreen
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.WalletNavigator
import com.gemwallet.android.ui.navigation.assetIdArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.gemwallet.android.ui.open
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import kotlinx.serialization.Serializable
import uniffi.gemstone.GemNotificationDestination

const val SettingsRoute = "settings"

@Serializable
data object CurrencyRoute : NavKey

@Serializable
data object SecurityRoute : NavKey

@Serializable
data object DeveloperRoute : NavKey

@Serializable
data object DeveloperPaymentsRoute : NavKey

@Serializable
data object InAppNotificationsRoute : NavKey

@Serializable
data object AboutUsRoute : NavKey

@Serializable
data object ChainListSettingsRoute : NavKey

@Serializable
data class ChainSettingsRoute(val chain: Chain) : NavKey

@Serializable
data object ServiceStatusRoute : NavKey

@Serializable
data object PriceAlertsRoute : NavKey

@Serializable
data class AssetPriceAlertsRoute(val assetId: AssetId) : NavKey

@Serializable
data class SetPriceAlertRoute(val assetId: AssetId) : NavKey

@Serializable
data object SupportRoute : NavKey

@Serializable
data object PreferencesRoute : NavKey

@Serializable
data object NotificationsRoute : NavKey

fun EntryProviderScope<NavKey>.settingsScreen(navigator: WalletNavigator, onPayment: (String) -> Unit, onCancel: () -> Unit) {
    entry<CurrencyRoute> {
        CurrencyScreen(onCancel = onCancel)
    }

    entry<SecurityRoute> {
        SecurityScreen(onCancel = onCancel)
    }

    entry<DeveloperRoute> {
        DeveloperScreen(
            onInAppNotifications = navigator::openInAppNotifications,
            onPayments = navigator::openDeveloperPayments,
            onCancel = onCancel,
        )
    }

    entry<DeveloperPaymentsRoute> {
        DeveloperPaymentsScene(
            onSelect = onPayment,
            onCancel = onCancel,
        )
    }

    entry<InAppNotificationsRoute> { key ->
        val context = LocalContext.current
        val uriHandler = LocalUriHandler.current
        InAppNotificationsScreen(
            message = navigator.routeMessage(key),
            onMessageShown = { navigator.clearRouteMessage(key) },
            onAction = { action ->
                when (action) {
                    InAppNotificationsAction.Cancel -> onCancel()

                    is InAppNotificationsAction.Open -> when (val destination = action.destination) {
                        is GemNotificationDestination.InApp -> navigator.openUrlAction(destination.action)
                        is GemNotificationDestination.Web -> uriHandler.open(context, destination.url)
                    }
                }
            },
        )
    }

    entry<AboutUsRoute> {
        AboutUsScreen(onCancel = onCancel)
    }

    entry<ChainListSettingsRoute> {
        ChainListSettingsScreen(
            onSelectChain = navigator::openChainSettings,
            onShowStatus = navigator::openServiceStatus,
            onCancel = onCancel,
        )
    }

    entry<ChainSettingsRoute>(
        metadata = { key -> routeArguments(RouteArgument.Chain to key.chain.string) },
    ) {
        ChainSettingsScreen(onCancel = onCancel)
    }

    entry<ServiceStatusRoute> {
        ServiceStatusScreen(onCancel = onCancel)
    }

    entry<PriceAlertsRoute> { key ->
        PriceAlertsScreen(
            message = navigator.routeMessage(key),
            onMessageShown = { navigator.clearRouteMessage(key) },
            onChart = navigator::openChart,
            onCancel = onCancel,
        )
    }

    entry<AssetPriceAlertsRoute>(
        metadata = { key -> routeArguments(assetIdArgument(key.assetId)) },
    ) { key ->
        AssetPriceAlertsScreen(
            message = navigator.routeMessage(key),
            onMessageShown = { navigator.clearRouteMessage(key) },
            onSetPriceAlert = navigator::openSetPriceAlert,
            onCancel = onCancel,
        )
    }

    entry<SetPriceAlertRoute>(
        metadata = { key -> routeArguments(assetIdArgument(key.assetId)) },
    ) {
        SetPriceAlertScreen(
            onCancel = onCancel,
            onComplete = navigator::popWithToast,
        )
    }

    entry<NotificationsRoute> {
        NotificationsScreen(
            onPriceAlerts = navigator::openPriceAlerts,
            onCancel = onCancel,
        )
    }

    entry<PreferencesRoute> {
        PreferencesScreen(
            onAction = { action ->
                when (action) {
                    PreferencesAction.Currencies -> navigator.openCurrency()
                    PreferencesAction.Networks -> navigator.openChainListSettings()
                    PreferencesAction.Contacts -> navigator.openContacts()
                    PreferencesAction.Cancel -> onCancel()
                }
            },
        )
    }

    entry<SupportRoute> { key ->
        val context = LocalContext.current
        val defaultUriHandler = LocalUriHandler.current
        val currentOnOpenUrl by rememberUpdatedState(navigator::openNotificationUrl)
        val uriHandler = remember(defaultUriHandler, context) {
            object : UriHandler {
                override fun openUri(uri: String) {
                    if (!currentOnOpenUrl(uri)) {
                        defaultUriHandler.open(context, uri)
                    }
                }
            }
        }
        CompositionLocalProvider(LocalUriHandler provides uriHandler) {
            SupportChatScreen(
                message = navigator.routeMessage(key),
                onMessageShown = { navigator.clearRouteMessage(key) },
                onCancel = onCancel,
            )
        }
    }
}
