package com.gemwallet.android

import androidx.lifecycle.DefaultLifecycleObserver
import androidx.lifecycle.LifecycleOwner
import com.gemwallet.android.data.repositories.connection.ConnectionStatusObserver
import com.gemwallet.android.data.repositories.perpetual.HyperliquidObserverService
import com.gemwallet.android.data.repositories.stream.StreamObserverService
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class AppLifecycleCoordinator @Inject constructor(
    private val streamObserver: StreamObserverService,
    private val hyperliquidObserver: HyperliquidObserverService,
    private val nodeAuthTokenService: NodeAuthTokenService,
    private val connectionStatusObserver: ConnectionStatusObserver,
) : DefaultLifecycleObserver {

    override fun onStart(owner: LifecycleOwner) {
        streamObserver.start()
        hyperliquidObserver.start()
        nodeAuthTokenService.start()
        connectionStatusObserver.start()
    }

    override fun onStop(owner: LifecycleOwner) {
        streamObserver.stop()
        hyperliquidObserver.stop()
        nodeAuthTokenService.stop()
        connectionStatusObserver.stop()
    }
}
