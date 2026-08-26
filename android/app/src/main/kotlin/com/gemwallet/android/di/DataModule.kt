package com.gemwallet.android.di

import com.gemwallet.android.blockchain.services.NodeStatusService
import com.gemwallet.android.blockchain.services.SignerPreloaderProxy
import com.gemwallet.android.cases.device.SyncDevice
import com.gemwallet.android.services.SyncService
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemConfirmService
import uniffi.gemstone.GemConfirmServiceInterface
import uniffi.gemstone.GemAppStartService
import uniffi.gemstone.GemGateway
import uniffi.gemstone.GemScanService
import uniffi.gemstone.TransactionSimulationService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object DataModule {

    @Provides
    @Singleton
    fun provideConfirmService(
        gateway: GemGateway,
        simulationService: TransactionSimulationService,
        scanService: GemScanService,
    ): GemConfirmServiceInterface = GemConfirmService(gateway, simulationService, scanService)

    @Provides
    @Singleton
    fun provideSignerPreloader(
        confirmService: GemConfirmServiceInterface,
    ): SignerPreloaderProxy {
        return SignerPreloaderProxy(confirmService)
    }

    @Singleton
    @Provides
    fun provideNodeStatusService(
        gateway: GemGateway,
    ): NodeStatusService {
        return NodeStatusService(gateway)
    }

    @Singleton
    @Provides
    fun provideSyncService(
        appStartService: GemAppStartService,
        syncDevice: SyncDevice,
    ): SyncService {
        return SyncService(
            appStartService = appStartService,
            syncDevice = syncDevice,
        )
    }
}
