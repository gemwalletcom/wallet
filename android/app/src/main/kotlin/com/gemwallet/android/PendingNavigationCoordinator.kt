package com.gemwallet.android

import android.content.Intent
import androidx.navigation3.runtime.NavKey
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import uniffi.gemstone.GemCodeOutcome
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemNavigationServiceInterface
import uniffi.gemstone.GemNavigationTab
import uniffi.gemstone.WalletConnectLink
import javax.inject.Inject
import javax.inject.Singleton

internal sealed interface PendingNavigation {

    sealed interface Input : PendingNavigation {
        val code: String?
    }

    data class FromLink(override val code: String) : Input

    data class FromScan(override val code: String) : Input

    data class FromNotification(val type: String, val data: String?) : Input {
        override val code: String? = null
    }

    data class Routes(val routes: List<NavKey>, val tab: GemNavigationTab? = null) : PendingNavigation

    data class Loading(val input: Input) : PendingNavigation
}

@Singleton
class PendingNavigationCoordinator @Inject constructor(private val notificationNavigation: NotificationNavigation, private val paymentNavigation: PaymentNavigation, private val navigationService: GemNavigationServiceInterface) {

    private val _pendingNavigation = MutableStateFlow<PendingNavigation?>(null)
    internal val pendingNavigation: StateFlow<PendingNavigation?> = _pendingNavigation.asStateFlow()

    fun pendIntent(intent: Intent) {
        val code = intent.dataString ?: return
        _pendingNavigation.update { PendingNavigation.FromLink(code) }
    }

    fun pendNotification(type: String, data: String?) {
        _pendingNavigation.update { PendingNavigation.FromNotification(type, data) }
    }

    fun pendScan(code: String) {
        _pendingNavigation.update { PendingNavigation.FromScan(code) }
    }

    fun clear() {
        _pendingNavigation.update { null }
    }

    suspend fun buildRoutes(walletConnect: WalletConnectHandler): GemErrorText? {
        val pending = _pendingNavigation.value as? PendingNavigation.Input ?: return null
        val code = pending.code
        if (code == null) {
            val destination = (pending as? PendingNavigation.FromNotification)?.let { notificationNavigation.prepareNavigation(it.type, it.data) }
            replace(pending, destination?.takeIf { it.routes.isNotEmpty() })
            return null
        }
        return when (val outcome = navigationService.openCode(code)) {
            is GemCodeOutcome.Open -> {
                replace(pending, outcome.target.destination().takeIf { it.routes.isNotEmpty() })
                null
            }

            is GemCodeOutcome.WalletConnect -> {
                when (val link = outcome.link) {
                    is WalletConnectLink.Connect -> walletConnect.onPairing(link.uri)
                    WalletConnectLink.Request -> walletConnect.onRequest()
                    is WalletConnectLink.Session -> Unit
                }
                replace(pending, null)
                null
            }

            is GemCodeOutcome.Payment -> {
                val current = if (outcome.showsLoading) PendingNavigation.Loading(pending).also { replace(pending, it) } else pending
                val routes = paymentNavigation.routes(outcome.payment)
                replace(current, PendingNavigation.Routes(routes).takeIf { routes.isNotEmpty() })
                null
            }

            is GemCodeOutcome.Failure -> {
                replace(pending, null)
                outcome.text
            }
        }
    }

    private fun replace(pending: PendingNavigation, replacement: PendingNavigation?) {
        _pendingNavigation.update { current -> if (current === pending) replacement else current }
    }

    interface WalletConnectHandler {
        fun onPairing(uri: String)
        fun onRequest()
    }
}
