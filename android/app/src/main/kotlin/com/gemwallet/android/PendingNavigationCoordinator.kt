package com.gemwallet.android

import android.content.Intent
import androidx.annotation.VisibleForTesting
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.ext.request
import com.gemwallet.android.ext.toPrimitives
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import uniffi.gemstone.UrlAction
import uniffi.gemstone.WalletConnectLink
import uniffi.gemstone.urlAction
import javax.inject.Inject
import javax.inject.Singleton

internal sealed interface PendingNavigation {

    sealed interface Raw : PendingNavigation {
        val code: String?
    }

    data class RawIntent(val intent: Intent) : Raw {
        override val code: String? = intent.dataString
    }

    data class RawCode(override val code: String) : Raw

    data class Routes(val routes: List<NavKey>) : PendingNavigation
}

@Singleton
class PendingNavigationCoordinator @Inject constructor(
    private val notificationNavigation: NotificationNavigation,
    private val paymentNavigation: PaymentNavigation,
) {

    private val _pendingNavigation = MutableStateFlow<PendingNavigation?>(null)
    internal val pendingNavigation: StateFlow<PendingNavigation?> = _pendingNavigation.asStateFlow()

    fun handleIntent(intent: Intent) {
        if (intent.hasNotificationPayload() || intent.dataString != null) {
            _pendingNavigation.update { PendingNavigation.RawIntent(Intent(intent)) }
        }
    }

    fun handleScan(code: String) {
        _pendingNavigation.update { PendingNavigation.RawCode(code) }
    }

    fun clear() {
        _pendingNavigation.update { null }
    }

    suspend fun consume(walletConnect: WalletConnectHandler) {
        val pending = _pendingNavigation.value as? PendingNavigation.Raw ?: return
        val action = pending.code?.let(::urlAction)

        val routes = when {
            action != null -> routes(action, walletConnect)
            pending is PendingNavigation.RawIntent -> notificationNavigation.prepareNavigation(pending.intent)
            else -> emptyList()
        }

        replace(pending, routes.takeIf { it.isNotEmpty() }?.let(PendingNavigation::Routes))
    }

    private suspend fun routes(action: UrlAction, walletConnect: WalletConnectHandler): List<NavKey> = when (action) {
        is UrlAction.WalletConnect -> {
            when (val link = action.link) {
                is WalletConnectLink.Connect -> walletConnect.onPairing(link.uri)
                WalletConnectLink.Request -> walletConnect.onRequest()
                is WalletConnectLink.Session -> Unit
            }
            emptyList()
        }
        is UrlAction.Deeplink -> listOfNotNull(action.deeplink.toRoute())
        is UrlAction.Payment -> when (val request = action.payment.toPrimitives().request) {
            null -> emptyList()
            else -> paymentNavigation.prepareNavigation(request)
        }
    }

    private fun replace(pending: PendingNavigation, replacement: PendingNavigation?) {
        _pendingNavigation.update { current -> if (current === pending) replacement else current }
    }

    @VisibleForTesting
    internal fun setPendingIntent(intent: Intent) {
        _pendingNavigation.update { PendingNavigation.RawIntent(intent) }
    }

    interface WalletConnectHandler {
        fun onPairing(uri: String)
        fun onRequest()
    }
}
