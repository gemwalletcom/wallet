package com.gemwallet.android

import android.content.Intent
import androidx.annotation.VisibleForTesting
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.Payment
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

    sealed interface Input : PendingNavigation {
        val code: String?
    }

    data class FromIntent(val intent: Intent) : Input {
        override val code: String? = intent.dataString
    }

    data class FromScan(override val code: String) : Input

    data class Routes(val routes: List<NavKey>) : PendingNavigation

    data class Loading(val input: Input) : PendingNavigation
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
            _pendingNavigation.update { PendingNavigation.FromIntent(Intent(intent)) }
        }
    }

    fun handleScan(code: String) {
        _pendingNavigation.update { PendingNavigation.FromScan(code) }
    }

    fun clear() {
        _pendingNavigation.update { null }
    }

    suspend fun buildRoutes(walletConnect: WalletConnectHandler): Boolean {
        val pending = _pendingNavigation.value as? PendingNavigation.Input ?: return true
        val action = pending.code?.let(::urlAction)
        val loading = if (action is UrlAction.Payment && action.payment.toPrimitives() is Payment.Link) {
            PendingNavigation.Loading(pending).also { replace(pending, it) }
        } else {
            null
        }

        val routes = when {
            action != null -> routes(action, walletConnect)
            pending is PendingNavigation.FromIntent -> notificationNavigation.prepareNavigation(pending.intent)
            else -> emptyList()
        }

        replace(loading ?: pending, routes.takeIf { it.isNotEmpty() }?.let(PendingNavigation::Routes))

        return when (pending) {
            is PendingNavigation.FromIntent -> true
            is PendingNavigation.FromScan -> routes.isNotEmpty() || action is UrlAction.WalletConnect
        }
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
        is UrlAction.Payment -> paymentNavigation.routes(action.payment.toPrimitives())
    }

    private fun replace(pending: PendingNavigation, replacement: PendingNavigation?) {
        _pendingNavigation.update { current -> if (current === pending) replacement else current }
    }

    @VisibleForTesting
    internal fun setIntent(intent: Intent) {
        _pendingNavigation.update { PendingNavigation.FromIntent(intent) }
    }

    interface WalletConnectHandler {
        fun onPairing(uri: String)
        fun onRequest()
    }
}
