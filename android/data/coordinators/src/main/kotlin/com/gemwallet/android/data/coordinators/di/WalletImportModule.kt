package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.wallet_import.coordinators.GetImportWalletState
import com.gemwallet.android.application.wallet_import.coordinators.SyncWalletConfiguration
import com.gemwallet.android.application.wallet_import.coordinators.SyncWalletImport
import com.gemwallet.android.application.transactions.coordinators.SyncTransactions
import com.gemwallet.android.cases.device.SyncDevice
import com.gemwallet.android.cases.nft.SyncNfts
import com.gemwallet.android.data.coordinators.wallet_import.SyncWalletConfigurationImpl
import com.gemwallet.android.data.coordinators.wallet_import.services.ImportWalletService
import com.gemwallet.android.data.service.store.WalletPreferencesFactory
import com.gemwallet.android.data.repositories.session.SessionRepository
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAssetDiscoveryService
import uniffi.gemstone.GemWalletConfigurationService
import uniffi.gemstone.GemWalletConfigurationStore
import com.gemwallet.android.data.coordinators.wallet_import.GemstoneWalletConfigurationStore
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object WalletImportModule {


    @Provides
    @Singleton
    fun provideGemWalletConfigurationStore(
        walletPreferencesFactory: WalletPreferencesFactory,
    ): GemWalletConfigurationStore = GemstoneWalletConfigurationStore(walletPreferencesFactory)

    @Provides
    @Singleton
    fun provideSyncWalletConfiguration(
        walletConfigurationService: GemWalletConfigurationService,
    ): SyncWalletConfiguration = SyncWalletConfigurationImpl(walletConfigurationService)

    @Provides
    @Singleton
    fun provideImportWalletService(
        discoveryService: GemAssetDiscoveryService,
        sessionRepository: SessionRepository,
        syncDevice: SyncDevice,
        syncTransactions: SyncTransactions,
        syncNfts: SyncNfts,
        walletConfigurationSync: SyncWalletConfiguration,
    ): ImportWalletService = ImportWalletService(
        discoveryService = discoveryService,
        sessionRepository = sessionRepository,
        syncDevice = syncDevice,
        syncTransactions = syncTransactions,
        syncNfts = syncNfts,
        walletConfigurationSync = walletConfigurationSync,
    )

    @Provides
    fun provideSyncWalletImport(service: ImportWalletService): SyncWalletImport = service

    @Provides
    fun provideGetImportWalletState(service: ImportWalletService): GetImportWalletState = service
}
