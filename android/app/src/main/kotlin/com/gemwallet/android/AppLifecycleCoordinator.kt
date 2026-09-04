package com.gemwallet.android

import androidx.lifecycle.DefaultLifecycleObserver
import androidx.lifecycle.LifecycleOwner
import com.gemwallet.android.data.services.gemstone.connection.ConnectionStatusObserver
import com.gemwallet.android.data.services.gemstone.device.DeviceObserverService
import com.gemwallet.android.data.services.gemstone.perpetual.HyperliquidObserverService
import com.gemwallet.android.data.services.gemstone.stream.StreamObserverService
import com.gemwallet.android.data.services.gemstone.transactions.TransactionStatusService
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class AppLifecycleCoordinator @Inject constructor(
    private val deviceObserver: DeviceObserverService,
    private val streamObserver: StreamObserverService,
    private val hyperliquidObserver: HyperliquidObserverService,
    private val connectionStatusObserver: ConnectionStatusObserver,
    private val transactionStatusService: TransactionStatusService,
) : DefaultLifecycleObserver {

    override fun onStart(owner: LifecycleOwner) {
        deviceObserver.start()
        streamObserver.start()
        hyperliquidObserver.start()
        connectionStatusObserver.start()
        transactionStatusService.start()
    }

    override fun onStop(owner: LifecycleOwner) {
        deviceObserver.stop()
        streamObserver.stop()
        hyperliquidObserver.stop()
        connectionStatusObserver.stop()
        transactionStatusService.stop()
    }
}
