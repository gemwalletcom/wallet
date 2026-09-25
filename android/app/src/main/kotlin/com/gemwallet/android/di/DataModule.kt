package com.gemwallet.android.di

import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneKeystorePassword
import com.gemwallet.android.data.services.gemstone.stores.GemstoneRecentActivityStore
import com.gemwallet.android.data.services.gemstone.transactions.TransactionStatusService
import com.gemwallet.android.data.services.store.database.AssetsDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAssetSelectionService
import uniffi.gemstone.GemAssetSelectionServiceInterface
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemBalanceService
import uniffi.gemstone.GemChainSettingsService
import uniffi.gemstone.GemChainSettingsServiceInterface
import uniffi.gemstone.GemConfirmService
import uniffi.gemstone.GemConfirmServiceInterface
import uniffi.gemstone.GemConfirmTransferService
import uniffi.gemstone.GemConfirmTransferServiceInterface
import uniffi.gemstone.GemDeviceService
import uniffi.gemstone.GemExplorerService
import uniffi.gemstone.GemGateway
import uniffi.gemstone.GemNameService
import uniffi.gemstone.GemNodeService
import uniffi.gemstone.GemPaymentService
import uniffi.gemstone.GemPerpetualService
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemPriceAlertService
import uniffi.gemstone.GemPriceService
import uniffi.gemstone.GemRecentActivityService
import uniffi.gemstone.GemRecentActivityServiceInterface
import uniffi.gemstone.GemScanService
import uniffi.gemstone.GemSearchService
import uniffi.gemstone.GemSimulationService
import uniffi.gemstone.GemSwapService
import uniffi.gemstone.GemTransactionSigner
import uniffi.gemstone.GemTransactionStateService
import uniffi.gemstone.GemWalletSessionService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object DataModule {

    @Provides
    @Singleton
    fun provideConfirmService(
        gateway: GemGateway,
        simulationService: GemSimulationService,
        scanService: GemScanService,
        transactionStateService: GemTransactionStateService,
        balanceService: GemBalanceService,
        priceService: GemPriceService,
        assetsService: GemAssetsService,
        transactionStatusService: TransactionStatusService,
    ): GemConfirmServiceInterface = GemConfirmService(gateway, simulationService, scanService, transactionStateService, balanceService, priceService, assetsService, transactionStatusService)

    @Provides
    @Singleton
    fun provideGemRecentActivityService(assetsDao: AssetsDao, walletSessionService: GemWalletSessionService): GemRecentActivityService = GemRecentActivityService(GemstoneRecentActivityStore(assetsDao), walletSessionService)

    @Provides
    fun provideGemAssetSelectionService(
        assetsService: GemAssetsService,
        searchService: GemSearchService,
        balanceService: GemBalanceService,
        priceAlertService: GemPriceAlertService,
        recentActivity: GemRecentActivityService,
        preferencesService: GemPreferencesService,
        perpetualService: GemPerpetualService,
        walletSessionService: GemWalletSessionService,
        swapService: GemSwapService,
    ): GemAssetSelectionServiceInterface = GemAssetSelectionService(
        assetsService,
        searchService,
        balanceService,
        priceAlertService,
        recentActivity,
        preferencesService,
        perpetualService,
        walletSessionService,
        swapService,
    )

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
        paymentService: GemPaymentService,
    ): GemConfirmTransferService = GemConfirmTransferService(
        confirmService as GemConfirmService,
        explorerService,
        nameService,
        signer,
        GemstoneKeystorePassword(passwordStore),
        recentActivity,
        preferencesService,
        paymentService,
    )

    @Provides
    fun provideGemChainSettingsService(nodeService: GemNodeService, explorerService: GemExplorerService, gateway: GemGateway): GemChainSettingsServiceInterface = GemChainSettingsService(nodeService, explorerService, gateway)

    @Provides
    fun provideGemRecentActivityServiceInterface(service: GemRecentActivityService): GemRecentActivityServiceInterface = service

    @Provides
    fun provideGemConfirmTransferServiceInterface(service: GemConfirmTransferService): GemConfirmTransferServiceInterface = service
}
