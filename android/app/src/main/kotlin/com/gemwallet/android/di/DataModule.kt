package com.gemwallet.android.di

import com.gemwallet.android.application.fiat.coordinators.SyncFiatAssets
import com.gemwallet.android.blockchain.services.BroadcastService
import com.gemwallet.android.blockchain.services.NodeStatusService
import com.gemwallet.android.blockchain.services.SignerPreloaderProxy
import com.gemwallet.android.cases.device.SyncDeviceInfo
import com.gemwallet.android.services.SyncService
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemGateway
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
    fun provideSignerPreloader(
        gateway: GemGateway,
    ): SignerPreloaderProxy {
        return SignerPreloaderProxy(
            gateway = gateway,
        )
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
        syncFiatAssets: SyncFiatAssets,
        syncDeviceInfo: SyncDeviceInfo,
    ): SyncService {
        return SyncService(
            syncFiatAssets = syncFiatAssets,
            syncDeviceInfo = syncDeviceInfo,
        )
    }
}
