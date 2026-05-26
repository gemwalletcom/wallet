package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.wallet_import.coordinators.GetAvailableAssetIds
import com.gemwallet.android.application.wallet_import.coordinators.GetImportWalletState
import com.gemwallet.android.application.wallet_import.coordinators.SyncWalletConfiguration
import com.gemwallet.android.application.wallet_import.coordinators.SyncWalletImport
import com.gemwallet.android.application.transactions.coordinators.SyncTransactions
import com.gemwallet.android.cases.banners.AddBanner
import com.gemwallet.android.cases.device.SyncSubscription
import com.gemwallet.android.cases.nft.SyncNfts
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.data.coordinators.wallet_import.SyncWalletConfigurationImpl
import com.gemwallet.android.data.coordinators.wallet_import.services.ImportWalletService
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.WalletPreferencesFactory
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.wallet.core.primitives.WalletId
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object WalletImportModule {

    @Provides
    @Singleton
    fun provideGetAvailableAssetIds(
        gemDeviceApiClient: GemDeviceApiClient,
    ): GetAvailableAssetIds = GetAvailableAssetIds { walletId ->
        gemDeviceApiClient.getAssets(walletId = WalletId(walletId), fromTimestamp = 0)
    }

    @Provides
    @Singleton
    fun provideSyncWalletConfiguration(
        gemDeviceApiClient: GemDeviceApiClient,
        addBanner: AddBanner,
        walletPreferencesFactory: WalletPreferencesFactory,
    ): SyncWalletConfiguration = SyncWalletConfigurationImpl(
        gemDeviceApiClient = gemDeviceApiClient,
        addBanner = addBanner,
        walletPreferencesFactory = walletPreferencesFactory,
    )

    @Provides
    @Singleton
    fun provideImportWalletService(
        sessionRepository: SessionRepository,
        getAvailableAssetIds: GetAvailableAssetIds,
        searchTokensCase: SearchTokensCase,
        assetsRepository: AssetsRepository,
        syncSubscription: SyncSubscription,
        syncTransactions: SyncTransactions,
        syncNfts: SyncNfts,
        walletConfigurationSync: SyncWalletConfiguration,
    ): ImportWalletService = ImportWalletService(
        sessionRepository = sessionRepository,
        getAvailableAssetIds = getAvailableAssetIds,
        searchTokensCase = searchTokensCase,
        assetsRepository = assetsRepository,
        syncSubscription = syncSubscription,
        syncTransactions = syncTransactions,
        syncNfts = syncNfts,
        walletConfigurationSync = walletConfigurationSync,
    )

    @Provides
    fun provideSyncWalletImport(service: ImportWalletService): SyncWalletImport = service

    @Provides
    fun provideGetImportWalletState(service: ImportWalletService): GetImportWalletState = service
}
