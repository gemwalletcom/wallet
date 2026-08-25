package com.gemwallet.android.di

import com.gemwallet.android.application.fiat.coordinators.SyncFiatAssets
import com.gemwallet.android.application.swap.coordinators.SyncSwapAssets
import com.gemwallet.android.blockchain.services.BroadcastService
import com.gemwallet.android.blockchain.services.NodeStatusService
import com.gemwallet.android.blockchain.services.SignerPreloaderProxy
import com.gemwallet.android.application.config.coordinators.GetRemoteConfig
import com.gemwallet.android.cases.device.SyncDevice
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.gemwallet.android.services.DeviceConfirmScanner
import com.gemwallet.android.services.SyncService
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemConfirmScanner
import uniffi.gemstone.GemConfirmService
import uniffi.gemstone.GemConfirmServiceInterface
import uniffi.gemstone.GemGateway
import uniffi.gemstone.TransactionSimulationService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object DataModule {

    @Provides
    @Singleton
    fun providesBroadcastProxy(
        gateway: GemGateway,
    ): BroadcastService = BroadcastService(
        gateway = gateway,
    )

    @Provides
    @Singleton
    fun provideConfirmScanner(
        gemDeviceApiClient: GemDeviceApiClient,
    ): GemConfirmScanner = DeviceConfirmScanner(gemDeviceApiClient)

    @Provides
    @Singleton
    fun provideConfirmService(
        gateway: GemGateway,
        simulationService: TransactionSimulationService,
        scanner: GemConfirmScanner,
    ): GemConfirmServiceInterface = GemConfirmService(gateway, simulationService, scanner)

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
        getRemoteConfig: GetRemoteConfig,
        syncFiatAssets: SyncFiatAssets,
        syncSwapAssets: SyncSwapAssets,
        syncDevice: SyncDevice,
    ): SyncService {
        return SyncService(
            getRemoteConfig = getRemoteConfig,
            syncFiatAssets = syncFiatAssets,
            syncSwapAssets = syncSwapAssets,
            syncDevice = syncDevice,
        )
    }
}
