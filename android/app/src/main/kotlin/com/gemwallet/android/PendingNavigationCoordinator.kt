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
import uniffi.gemstone.UrlAction
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

    data class FromAction(val action: UrlAction) : Input {
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
        if (intent.flags and Intent.FLAG_ACTIVITY_LAUNCHED_FROM_HISTORY != 0) return
        val code = intent.dataString ?: return
        _pendingNavigation.update { PendingNavigation.FromLink(code) }
    }

    fun pendNotification(type: String, data: String?) {
        _pendingNavigation.update { PendingNavigation.FromNotification(type, data) }
    }

    fun pendScan(code: String) {
        _pendingNavigation.update { PendingNavigation.FromScan(code) }
    }

    fun pendAction(action: UrlAction) {
        _pendingNavigation.update { PendingNavigation.FromAction(action) }
    }

    fun clear() {
        _pendingNavigation.update { null }
    }

    suspend fun buildRoutes(walletConnect: WalletConnectHandler): GemErrorText? {
        val pending = _pendingNavigation.value as? PendingNavigation.Input ?: return null
        val outcome = when (pending) {
            is PendingNavigation.FromNotification -> {
                val destination = notificationNavigation.prepareNavigation(pending.type, pending.data)
                replace(pending, destination?.takeIf { it.routes.isNotEmpty() })
                return null
            }

            is PendingNavigation.FromAction -> navigationService.openAction(pending.action)

            is PendingNavigation.FromLink -> navigationService.openCode(pending.code)

            is PendingNavigation.FromScan -> navigationService.openCode(pending.code)
        }
        return when (outcome) {
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
