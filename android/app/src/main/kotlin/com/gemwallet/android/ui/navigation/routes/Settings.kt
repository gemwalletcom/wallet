package com.gemwallet.android.ui.navigation.routes

import androidx.compose.runtime.Composable
import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.settings.aboutus.presents.AboutUsScreen
import com.gemwallet.android.features.settings.currency.presents.CurrenciesScene
import com.gemwallet.android.features.settings.develop.presents.DevelopScene
import com.gemwallet.android.features.settings.in_app_notifications.presents.InAppNotificationsAction
import com.gemwallet.android.features.settings.in_app_notifications.presents.InAppNotificationsScene
import com.gemwallet.android.features.settings.networks.presents.NetworksScreen
import com.gemwallet.android.features.settings.price_alerts.presents.PriceAlertTargetNavScreen
import com.gemwallet.android.features.settings.price_alerts.presents.PriceAlertsNavScreen
import com.gemwallet.android.features.settings.security.presents.SecurityScene
import com.gemwallet.android.features.settings.settings.presents.views.NotificationsScene
import com.gemwallet.android.features.settings.settings.presents.views.PreferencesScene
import com.gemwallet.android.features.settings.settings.presents.views.SupportChatNavScreen
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
    toastMessage: (NavKey) -> String?,
    onToastShown: (NavKey) -> Unit,
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
            onCancel = onCancel,
        )
    }

    entry<InAppNotificationsRoute> {
        InAppNotificationsScene(
            onAction = { action ->
                when (action) {
                    InAppNotificationsAction.Cancel -> onAction(SettingsAction.Cancel)
                    is InAppNotificationsAction.OpenUrl ->
                        onAction(SettingsAction.OpenNotificationUrl(action.url))
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
            toastMessage = toastMessage(key),
            onToastShown = { onToastShown(key) },
            onAction = onAction,
        )
    }

    entry<AssetPriceAlertsRoute>(
        metadata = { key -> routeArguments(assetIdArgument(key.assetId)) },
    ) { key ->
        priceAlertsScreenContent(
            toastMessage = toastMessage(key),
            onToastShown = { onToastShown(key) },
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
            onNetworks = { onAction(SettingsAction.Networks) },
            onCurrencies = { onAction(SettingsAction.Currencies) },
            onContacts = { onAction(SettingsAction.Contacts) },
            onCancel = onCancel,
        )
    }

    entry<SupportRoute> {
        SupportChatNavScreen(onCancel = onCancel)
    }
}

@Composable
private fun priceAlertsScreenContent(
    toastMessage: String?,
    onToastShown: () -> Unit,
    onAction: (SettingsAction) -> Unit,
) {
    PriceAlertsNavScreen(
        toastMessage = toastMessage,
        onToastShown = onToastShown,
        onChart = { onAction(SettingsAction.Chart(it)) },
        onAddPriceAlertTarget = { onAction(SettingsAction.AddPriceAlertTarget(it)) },
        onCancel = { onAction(SettingsAction.Cancel) },
    )
}
