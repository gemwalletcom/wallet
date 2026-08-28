package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.wallet_import.coordinators.GetImportWalletState
import com.gemwallet.android.application.wallet_import.coordinators.SetupWallet
import com.gemwallet.android.application.wallet_import.coordinators.SyncWalletImport
import com.gemwallet.android.data.coordinators.wallet_import.SetupWalletImpl
import com.gemwallet.android.data.coordinators.wallet_import.services.ImportWalletService
import com.gemwallet.android.data.repositories.session.SessionRepository
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAssetDiscoveryService
import uniffi.gemstone.GemAppStartService
import javax.inject.Singleton
import uniffi.gemstone.GemDeviceService

@InstallIn(SingletonComponent::class)
@Module
object WalletImportModule {


    @Provides
    @Singleton
    fun provideSetupWallet(
        appStartService: GemAppStartService,
    ): SetupWallet = SetupWalletImpl(appStartService)

    @Provides
    @Singleton
    fun provideImportWalletService(
        discoveryService: GemAssetDiscoveryService,
        sessionRepository: SessionRepository,
        deviceService: GemDeviceService,
    ): ImportWalletService = ImportWalletService(
        discoveryService = discoveryService,
        sessionRepository = sessionRepository,
        deviceService = deviceService,
    )

    @Provides
    fun provideSyncWalletImport(service: ImportWalletService): SyncWalletImport = service

    @Provides
    fun provideGetImportWalletState(service: ImportWalletService): GetImportWalletState = service
}
