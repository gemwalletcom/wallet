package com.gemwallet.android

import android.content.Intent
import android.util.Log
import androidx.annotation.VisibleForTesting
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.serializer.decodeJson
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import uniffi.gemstone.Deeplink
import uniffi.gemstone.GemDeeplinkService
import uniffi.gemstone.GemDeeplinkServiceInterface
import uniffi.gemstone.GemNavigationServiceInterface
import uniffi.gemstone.GemNavigationTab
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.Payment
import uniffi.gemstone.UrlAction
import uniffi.gemstone.WalletConnectLink
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

    data class Routes(val routes: List<NavKey>, val tab: GemNavigationTab? = null) : PendingNavigation

    data class Loading(val input: Input) : PendingNavigation
}

@Singleton
class PendingNavigationCoordinator @Inject constructor(
    private val notificationNavigation: NotificationNavigation,
    private val paymentNavigation: PaymentNavigation,
    private val navigationService: GemNavigationServiceInterface,
    private val deeplinkService: GemDeeplinkServiceInterface,
) {

    private companion object {
        const val TAG = "PendingNavigation"
    }

    private val _pendingNavigation = MutableStateFlow<PendingNavigation?>(null)
    internal val pendingNavigation: StateFlow<PendingNavigation?> = _pendingNavigation.asStateFlow()

    fun pendIntent(intent: Intent) {
        if (intent.hasNotificationPayload() || intent.dataString != null) {
            _pendingNavigation.update { PendingNavigation.FromIntent(Intent(intent)) }
        }
    }

    fun pendScan(code: String) {
        _pendingNavigation.update { PendingNavigation.FromScan(code) }
    }

    fun clear() {
        _pendingNavigation.update { null }
    }

    suspend fun buildRoutes(walletConnect: WalletConnectHandler): Boolean {
        val pending = _pendingNavigation.value as? PendingNavigation.Input ?: return true
        val action = pending.code?.let(deeplinkService::urlAction)
        val loading = if (action is UrlAction.Payment && action.payment is Payment.Link) {
            PendingNavigation.Loading(pending).also { replace(pending, it) }
        } else {
            null
        }

        val destination = when {
            action != null -> destination(action, walletConnect)
            pending is PendingNavigation.FromIntent -> notificationNavigation.prepareNavigation(pending.intent)
            else -> PendingNavigation.Routes(emptyList())
        }
        val hasRoutes = destination.routes.isNotEmpty()

        replace(loading ?: pending, destination.takeIf { hasRoutes })

        return when (pending) {
            is PendingNavigation.FromIntent -> hasRoutes || action !is UrlAction.Payment
            is PendingNavigation.FromScan -> hasRoutes || action is UrlAction.WalletConnect
        }
    }

    private suspend fun destination(action: UrlAction, walletConnect: WalletConnectHandler): PendingNavigation.Routes = when (action) {
        is UrlAction.WalletConnect -> {
            when (val link = action.link) {
                is WalletConnectLink.Connect -> walletConnect.onPairing(link.uri)
                WalletConnectLink.Request -> walletConnect.onRequest()
                is WalletConnectLink.Session -> Unit
            }
            PendingNavigation.Routes(emptyList())
        }

        is UrlAction.Deeplink -> destination(action.deeplink)

        is UrlAction.Payment -> PendingNavigation.Routes(paymentNavigation.routes(action.payment))
    }

    private suspend fun destination(deeplink: Deeplink): PendingNavigation.Routes = runCatching { navigationService.openDeeplink(deeplink).destination() }
        .onFailure { if (it is GemServiceException.NoAccountForChain) throw it }
        .onFailure { Log.e(TAG, "preparing a deep link failed", it) }
        .getOrDefault(PendingNavigation.Routes(emptyList()))

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
