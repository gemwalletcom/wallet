package com.gemwallet.android.ui.navigation.routes

import androidx.compose.runtime.Composable
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
import com.gemwallet.android.features.price_alerts.presents.PriceAlertsScreen
import com.gemwallet.android.features.price_alerts.presents.SetPriceAlertScreen
import com.gemwallet.android.features.settings.presents.NotificationsScreen
import com.gemwallet.android.features.settings.presents.PreferencesScreen
import com.gemwallet.android.features.settings.presents.about_us.AboutUsScreen
import com.gemwallet.android.features.settings.presents.chain_settings.ChainSettingsScreen
import com.gemwallet.android.features.settings.presents.currency.CurrencyScreen
import com.gemwallet.android.features.settings.presents.developer.DeveloperPaymentsScene
import com.gemwallet.android.features.settings.presents.developer.DeveloperScreen
import com.gemwallet.android.features.settings.presents.security.SecurityScreen
import com.gemwallet.android.features.support.presents.SupportChatScreen
import com.gemwallet.android.ui.models.actions.PreferencesAction
import com.gemwallet.android.ui.models.navigation.RouteMessage
import com.gemwallet.android.ui.navigation.assetIdArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.gemwallet.android.ui.open
import com.wallet.core.primitives.AssetId
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
data object ChainSettingsRoute : NavKey

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

fun EntryProviderScope<NavKey>.settingsScreen(onAction: (SettingsAction) -> Unit, onOpenUrl: (String) -> Boolean, routeMessage: (NavKey) -> RouteMessage?, onRouteMessageShown: (NavKey) -> Unit) {
    val onCancel = { onAction(SettingsAction.Cancel) }

    entry<CurrencyRoute> {
        CurrencyScreen(onCancel = onCancel)
    }

    entry<SecurityRoute> {
        SecurityScreen(onCancel = onCancel)
    }

    entry<DeveloperRoute> {
        DeveloperScreen(
            onInAppNotifications = { onAction(SettingsAction.InAppNotifications) },
            onPayments = { onAction(SettingsAction.DeveloperPayments) },
            onCancel = onCancel,
        )
    }

    entry<DeveloperPaymentsRoute> {
        DeveloperPaymentsScene(
            onSelect = { onAction(SettingsAction.Payment(it)) },
            onCancel = onCancel,
        )
    }

    entry<InAppNotificationsRoute> { key ->
        val context = LocalContext.current
        val uriHandler = LocalUriHandler.current
        InAppNotificationsScreen(
            message = routeMessage(key),
            onMessageShown = { onRouteMessageShown(key) },
            onAction = { action ->
                when (action) {
                    InAppNotificationsAction.Cancel -> onAction(SettingsAction.Cancel)

                    is InAppNotificationsAction.Open -> when (val destination = action.destination) {
                        is GemNotificationDestination.InApp -> onAction(SettingsAction.OpenNotification(destination.action))
                        is GemNotificationDestination.Web -> uriHandler.open(context, destination.url)
                    }
                }
            },
        )
    }

    entry<AboutUsRoute> {
        AboutUsScreen(onCancel = onCancel)
    }

    entry<ChainSettingsRoute> {
        ChainSettingsScreen(onCancel = onCancel)
    }

    entry<PriceAlertsRoute> { key ->
        priceAlertsScreenContent(
            message = routeMessage(key),
            onMessageShown = { onRouteMessageShown(key) },
            onAction = onAction,
        )
    }

    entry<AssetPriceAlertsRoute>(
        metadata = { key -> routeArguments(assetIdArgument(key.assetId)) },
    ) { key ->
        priceAlertsScreenContent(
            message = routeMessage(key),
            onMessageShown = { onRouteMessageShown(key) },
            onAction = onAction,
        )
    }

    entry<SetPriceAlertRoute>(
        metadata = { key -> routeArguments(assetIdArgument(key.assetId)) },
    ) {
        SetPriceAlertScreen(
            onCancel = onCancel,
            onComplete = { onAction(SettingsAction.SetPriceAlertComplete(it)) },
        )
    }

    entry<NotificationsRoute> {
        NotificationsScreen(
            onPriceAlerts = { onAction(SettingsAction.PriceAlerts) },
            onCancel = onCancel,
        )
    }

    entry<PreferencesRoute> {
        PreferencesScreen(
            onAction = { action ->
                when (action) {
                    PreferencesAction.Currencies -> onAction(SettingsAction.Currencies)
                    PreferencesAction.Networks -> onAction(SettingsAction.Networks)
                    PreferencesAction.Contacts -> onAction(SettingsAction.Contacts)
                    PreferencesAction.Cancel -> onCancel()
                }
            },
        )
    }

    entry<SupportRoute> { key ->
        val context = LocalContext.current
        val defaultUriHandler = LocalUriHandler.current
        val currentOnOpenUrl by rememberUpdatedState(onOpenUrl)
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
                message = routeMessage(key),
                onMessageShown = { onRouteMessageShown(key) },
                onCancel = onCancel,
            )
        }
    }
}

@Composable
private fun priceAlertsScreenContent(message: RouteMessage?, onMessageShown: () -> Unit, onAction: (SettingsAction) -> Unit) {
    PriceAlertsScreen(
        message = message,
        onMessageShown = onMessageShown,
        onChart = { onAction(SettingsAction.Chart(it)) },
        onSetPriceAlert = { onAction(SettingsAction.SetPriceAlert(it)) },
        onCancel = { onAction(SettingsAction.Cancel) },
    )
}
