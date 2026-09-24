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
import com.gemwallet.android.features.settings.aboutus.presents.AboutUsScreen
import com.gemwallet.android.features.settings.currency.presents.CurrenciesScene
import com.gemwallet.android.features.settings.develop.presents.DevelopScene
import com.gemwallet.android.features.settings.develop.presents.PaymentsScene
import com.gemwallet.android.features.settings.in_app_notifications.presents.InAppNotificationsAction
import com.gemwallet.android.features.settings.in_app_notifications.presents.InAppNotificationsScene
import com.gemwallet.android.features.settings.networks.presents.NetworksScreen
import com.gemwallet.android.features.settings.price_alerts.presents.PriceAlertTargetNavScreen
import com.gemwallet.android.features.settings.price_alerts.presents.PriceAlertsNavScreen
import com.gemwallet.android.features.settings.security.presents.SecurityScene
import com.gemwallet.android.features.settings.settings.presents.views.NotificationsScene
import com.gemwallet.android.features.settings.settings.presents.views.PreferencesScene
import com.gemwallet.android.features.settings.settings.presents.views.SupportChatNavScreen
import com.gemwallet.android.ui.models.actions.PreferencesAction
import com.gemwallet.android.ui.models.navigation.RouteMessage
import com.gemwallet.android.ui.navigation.assetIdArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.gemwallet.android.ui.open
import com.wallet.core.primitives.AssetId
import kotlinx.serialization.Serializable
import uniffi.gemstone.GemNotificationDestination

const val settingsRoute = "settings"

@Serializable
data object CurrenciesRoute : NavKey

@Serializable
data object SecurityRoute : NavKey

@Serializable
data object DevelopRoute : NavKey

@Serializable
data object DevelopPaymentsRoute : NavKey

@Serializable
data object InAppNotificationsRoute : NavKey

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
    onAction: (SettingsAction) -> Unit,
    onOpenUrl: (String) -> Boolean,
    routeMessage: (NavKey) -> RouteMessage?,
    onRouteMessageShown: (NavKey) -> Unit,
) {
    val onCancel = { onAction(SettingsAction.Cancel) }

    entry<CurrenciesRoute> {
        CurrenciesScene(onCancel = onCancel)
    }

    entry<SecurityRoute> {
        SecurityScene(onCancel = onCancel)
    }

    entry<DevelopRoute> {
        DevelopScene(
            onInAppNotifications = { onAction(SettingsAction.InAppNotifications) },
            onPayments = { onAction(SettingsAction.DeveloperPayments) },
            onCancel = onCancel,
        )
    }

    entry<DevelopPaymentsRoute> {
        PaymentsScene(
            onSelect = { onAction(SettingsAction.Payment(it)) },
            onCancel = onCancel,
        )
    }

    entry<InAppNotificationsRoute> { key ->
        val context = LocalContext.current
        val uriHandler = LocalUriHandler.current
        InAppNotificationsScene(
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

    entry<AboutusRoute> {
        AboutUsScreen(onCancel = onCancel)
    }

    entry<NetworksRoute> {
        NetworksScreen(onCancel = onCancel)
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

    entry<AddPriceAlertTargetRoute>(
        metadata = { key -> routeArguments(assetIdArgument(key.assetId)) },
    ) {
        PriceAlertTargetNavScreen(
            onCancel = onCancel,
            onComplete = { onAction(SettingsAction.PriceAlertTargetComplete(it)) },
        )
    }

    entry<NotificationsRoute> {
        NotificationsScene(
            onPriceAlerts = { onAction(SettingsAction.PriceAlerts) },
            onCancel = onCancel,
        )
    }

    entry<PreferencesRoute> {
        PreferencesScene(
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
            SupportChatNavScreen(
                message = routeMessage(key),
                onMessageShown = { onRouteMessageShown(key) },
                onCancel = onCancel,
            )
        }
    }
}

@Composable
private fun priceAlertsScreenContent(message: RouteMessage?, onMessageShown: () -> Unit, onAction: (SettingsAction) -> Unit) {
    PriceAlertsNavScreen(
        message = message,
        onMessageShown = onMessageShown,
        onChart = { onAction(SettingsAction.Chart(it)) },
        onAddPriceAlertTarget = { onAction(SettingsAction.AddPriceAlertTarget(it)) },
        onCancel = { onAction(SettingsAction.Cancel) },
    )
}
