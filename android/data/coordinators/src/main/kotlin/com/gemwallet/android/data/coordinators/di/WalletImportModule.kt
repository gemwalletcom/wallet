package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.wallet_import.coordinators.GetAvailableAssetIds
import com.gemwallet.android.application.wallet_import.coordinators.GetImportWalletState
import com.gemwallet.android.application.wallet_import.services.ImportAssets
import com.gemwallet.android.application.transactions.coordinators.SyncTransactions
import com.gemwallet.android.cases.device.SyncSubscription
import com.gemwallet.android.cases.nft.SyncNfts
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.data.coordinators.wallet_import.services.ImportWalletService
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
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
        gemDeviceApiClient.getAssets(walletId = walletId, fromTimestamp = 0)
    }

    @Provides
    @Singleton
    fun provideImportAssetsService(
        sessionRepository: SessionRepository,
        getAvailableAssetIds: GetAvailableAssetIds,
        searchTokensCase: SearchTokensCase,
        assetsRepository: AssetsRepository,
        syncSubscription: SyncSubscription,
        syncTransactions: SyncTransactions,
        syncNfts: SyncNfts,
    ): ImportWalletService {
        return ImportWalletService(
            sessionRepository = sessionRepository,
            getAvailableAssetIds = getAvailableAssetIds,
            searchTokensCase = searchTokensCase,
            assetsRepository = assetsRepository,
            syncSubscription = syncSubscription,
            syncTransactions = syncTransactions,
            syncNfts = syncNfts,
        )
    }

    @Provides
    fun provideImportAssets(service: ImportWalletService): ImportAssets = service

    @Provides
    fun provideGetImportWalletState(service: ImportWalletService): GetImportWalletState = service
}
