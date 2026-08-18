package com.gemwallet.android

import androidx.lifecycle.DefaultLifecycleObserver
import androidx.lifecycle.LifecycleOwner
import com.gemwallet.android.data.repositories.connection.ConnectionStatusObserver
import com.gemwallet.android.data.repositories.device.DeviceObserverService
import com.gemwallet.android.data.repositories.perpetual.HyperliquidObserverService
import com.gemwallet.android.data.repositories.stream.StreamObserverService
import com.gemwallet.android.data.repositories.transactions.TransactionStateScheduler
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class AppLifecycleCoordinator @Inject constructor(
    private val deviceObserver: DeviceObserverService,
    private val streamObserver: StreamObserverService,
    private val hyperliquidObserver: HyperliquidObserverService,
    private val nodeAuthTokenService: NodeAuthTokenService,
    private val connectionStatusObserver: ConnectionStatusObserver,
    private val transactionStateScheduler: TransactionStateScheduler,
) : DefaultLifecycleObserver {

    override fun onStart(owner: LifecycleOwner) {
        deviceObserver.start()
        streamObserver.start()
        hyperliquidObserver.start()
        nodeAuthTokenService.start()
        connectionStatusObserver.start()
        transactionStateScheduler.start()
    }

    override fun onStop(owner: LifecycleOwner) {
        deviceObserver.stop()
        streamObserver.stop()
        hyperliquidObserver.stop()
        nodeAuthTokenService.stop()
        connectionStatusObserver.stop()
        transactionStateScheduler.stop()
    }
}
