package com.gemwallet.android

import android.content.Intent
import androidx.annotation.VisibleForTesting
import androidx.navigation3.runtime.NavKey
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import javax.inject.Inject

internal sealed interface PendingNavigation {
    data class RawIntent(val intent: Intent) : PendingNavigation
    data class Route(val routes: List<NavKey>) : PendingNavigation {
        constructor(route: NavKey) : this(listOf(route))
    }
}

class PendingNavigationCoordinator @Inject constructor(
    private val notificationNavigation: NotificationNavigation,
) {

    private val _pendingNavigation = MutableStateFlow<PendingNavigation?>(null)
    internal val pendingNavigation: StateFlow<PendingNavigation?> = _pendingNavigation.asStateFlow()

    fun handleIntent(intent: Intent) {
        if (intent.hasNotificationPayload() || intent.dataString != null) {
            _pendingNavigation.update { PendingNavigation.RawIntent(Intent(intent)) }
        }
    }

    fun consume() {
        _pendingNavigation.update { null }
    }

    suspend fun resolve(walletConnect: WalletConnectHandler) {
        val pendingIntent = (_pendingNavigation.value as? PendingNavigation.RawIntent)?.intent ?: return
        val uri = pendingIntent.dataString

        uri?.toWalletConnectLink()?.let { link ->
            when (link) {
                is WalletConnectLink.Pairing -> walletConnect.onPairing(link.uri)
                WalletConnectLink.Request -> walletConnect.onRequest()
                WalletConnectLink.Session -> Unit
            }
            replace(pendingIntent, replacement = null)
            return
        }

        uri?.toWebDeepLinkRoute()?.let { route ->
            replace(pendingIntent, PendingNavigation.Route(route))
            return
        }

        if (!pendingIntent.hasNotificationPayload()) {
            replace(pendingIntent, replacement = null)
            return
        }

        val routes = notificationNavigation.prepareNavigation(pendingIntent)
        replace(pendingIntent, replacement = routes.takeIf { it.isNotEmpty() }?.let(PendingNavigation::Route))
    }

    private fun replace(pendingIntent: Intent, replacement: PendingNavigation?) {
        _pendingNavigation.update { current ->
            if (current is PendingNavigation.RawIntent && current.intent === pendingIntent) {
                replacement
            } else {
                current
            }
        }
    }

    @VisibleForTesting
    internal fun setPendingIntentForTest(intent: Intent) {
        _pendingNavigation.update { PendingNavigation.RawIntent(intent) }
    }

    interface WalletConnectHandler {
        fun onPairing(uri: String)
        fun onRequest()
    }
}
