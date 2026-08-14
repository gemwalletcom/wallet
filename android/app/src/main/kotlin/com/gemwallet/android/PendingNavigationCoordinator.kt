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

internal sealed interface PendingNavigation {

    sealed interface Unresolved : PendingNavigation

    data class RawIntent(val intent: Intent) : Unresolved

    data class RawCode(val code: String) : Unresolved

    data class Route(val routes: List<NavKey>) : PendingNavigation {
        constructor(route: NavKey) : this(listOf(route))
    }
}

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

    fun consume() {
        _pendingNavigation.update { null }
    }

    suspend fun resolve(walletConnect: WalletConnectHandler) {
        when (val pending = _pendingNavigation.value) {
            is PendingNavigation.RawCode -> resolve(pending, urlAction(pending.code), walletConnect)
            is PendingNavigation.RawIntent -> resolve(pending, walletConnect)
            else -> Unit
        }
    }

    private suspend fun resolve(pending: PendingNavigation.RawIntent, walletConnect: WalletConnectHandler) {
        val action = pending.intent.dataString?.let(::urlAction)

        when (action) {
            null -> replace(pending, navigation(notificationNavigation.prepareNavigation(pending.intent)))
            else -> resolve(pending, action, walletConnect)
        }
    }

    private suspend fun resolve(pending: PendingNavigation.Unresolved, action: UrlAction?, walletConnect: WalletConnectHandler) {
        replace(pending, action?.let { navigation(routes(it, walletConnect)) })
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
        is UrlAction.Payment -> paymentNavigation.prepareNavigation(action.payment.toPrimitives().request)
    }

    private fun navigation(routes: List<NavKey>): PendingNavigation? =
        routes.takeIf { it.isNotEmpty() }?.let(PendingNavigation::Route)

    private fun replace(pending: PendingNavigation, replacement: PendingNavigation?) {
        _pendingNavigation.update { current -> if (current === pending) replacement else current }
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
