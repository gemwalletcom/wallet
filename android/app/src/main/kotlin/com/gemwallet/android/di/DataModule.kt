package com.gemwallet.android.di

import com.gemwallet.android.blockchain.services.SignerPreloaderProxy
import com.gemwallet.android.services.SyncService
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemBalanceService
import com.gemwallet.android.data.services.gemstone.assets.RecentAssetsService
import com.gemwallet.android.data.services.gemstone.stores.GemstoneRecentActivityStore
import uniffi.gemstone.GemRecentActivityService
import uniffi.gemstone.GemConfirmTransferService
import uniffi.gemstone.GemExplorerService
import uniffi.gemstone.GemNameService
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemAssetConfigService
import uniffi.gemstone.GemTransactionSigner
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneKeystorePassword
import uniffi.gemstone.GemConfirmService
import uniffi.gemstone.GemConfirmServiceInterface
import uniffi.gemstone.GemAppStartService
import uniffi.gemstone.GemGateway
import uniffi.gemstone.GemNodeStatusService
import uniffi.gemstone.GemPriceService
import uniffi.gemstone.GemScanService
import uniffi.gemstone.GemTransactionStateService
import uniffi.gemstone.TransactionSimulationService
import javax.inject.Singleton
import uniffi.gemstone.GemDeviceService

@InstallIn(SingletonComponent::class)
@Module
object DataModule {

    @Provides
    @Singleton
    fun provideConfirmService(
        gateway: GemGateway,
        simulationService: TransactionSimulationService,
        scanService: GemScanService,
        transactionStateService: GemTransactionStateService,
        balanceService: GemBalanceService,
        priceService: GemPriceService,
        assetsService: GemAssetsService,
    ): GemConfirmServiceInterface = GemConfirmService(gateway, simulationService, scanService, transactionStateService, balanceService, priceService, assetsService)

    @Provides
    @Singleton
    fun provideGemRecentActivityService(
        recentAssetsService: RecentAssetsService,
    ): GemRecentActivityService = GemRecentActivityService(GemstoneRecentActivityStore(recentAssetsService))

    @Provides
    @Singleton
    fun provideGemConfirmTransferService(
        confirmService: GemConfirmServiceInterface,
        explorerService: GemExplorerService,
        nameService: GemNameService,
        signer: GemTransactionSigner,
        passwordStore: PasswordStore,
        recentActivity: GemRecentActivityService,
        preferencesService: GemPreferencesService,
    ): GemConfirmTransferService = GemConfirmTransferService(
        confirmService as GemConfirmService,
        explorerService,
        nameService,
        GemAssetConfigService(),
        signer,
        GemstoneKeystorePassword(passwordStore),
        recentActivity,
        preferencesService,
    )

    @Provides
    @Singleton
    fun provideSignerPreloader(
        confirmService: GemConfirmTransferService,
    ): SignerPreloaderProxy {
        return SignerPreloaderProxy(confirmService)
    }

    @Singleton
    @Provides
    fun provideGemNodeStatusService(
        gateway: GemGateway,
    ): GemNodeStatusService = GemNodeStatusService(gateway)

    @Singleton
    @Provides
    fun provideSyncService(
        appStartService: GemAppStartService,
    ): SyncService = SyncService(appStartService = appStartService)
}
