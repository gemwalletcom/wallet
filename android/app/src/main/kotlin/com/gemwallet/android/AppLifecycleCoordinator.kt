package com.gemwallet.android

import androidx.lifecycle.DefaultLifecycleObserver
import androidx.lifecycle.LifecycleOwner
import com.gemwallet.android.data.adapters.connection.ConnectionStatusObserver
import com.gemwallet.android.data.adapters.device.DeviceObserverService
import com.gemwallet.android.data.adapters.perpetual.HyperliquidObserverService
import com.gemwallet.android.data.adapters.stream.StreamObserverService
import com.gemwallet.android.data.adapters.transactions.TransactionStateTracker
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class AppLifecycleCoordinator @Inject constructor(
    private val deviceObserver: DeviceObserverService,
    private val streamObserver: StreamObserverService,
    private val hyperliquidObserver: HyperliquidObserverService,
    private val connectionStatusObserver: ConnectionStatusObserver,
    private val transactionStateTracker: TransactionStateTracker,
) : DefaultLifecycleObserver {

    override fun onStart(owner: LifecycleOwner) {
        deviceObserver.start()
        streamObserver.start()
        hyperliquidObserver.start()
        connectionStatusObserver.start()
        transactionStateTracker.start()
    }

    override fun onStop(owner: LifecycleOwner) {
        deviceObserver.stop()
        streamObserver.stop()
        hyperliquidObserver.stop()
        connectionStatusObserver.stop()
        transactionStateTracker.stop()
    }
}
