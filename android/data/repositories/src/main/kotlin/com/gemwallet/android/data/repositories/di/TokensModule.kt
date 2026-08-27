package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.cases.tokens.SyncAssetPrices
import com.gemwallet.android.cases.tokens.WalletSearchScopeCase
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.data.repositories.prices.PricesRepository
import com.gemwallet.android.data.repositories.tokens.TokensRepository
import com.gemwallet.android.data.repositories.tokens.WalletSearch
import com.gemwallet.android.data.repositories.tokens.WalletSearchTokens
import com.gemwallet.android.data.service.store.database.AssetListDao
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.service.store.database.SearchDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemSearchService
import uniffi.gemstone.GemSearchStore
import com.gemwallet.android.data.repositories.gemstone.GemstoneSearchStore
import com.gemwallet.android.data.repositories.session.SessionRepository
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object TokensModule {
    @Provides
    @Singleton
    fun provideTokensRepository(
        assetsDao: AssetsDao,
        pricesDao: PricesDao,
        pricesRepository: PricesRepository,
        sessionRepository: SessionRepository,
        searchService: GemSearchService,
        assetsService: GemAssetsService,
    ): TokensRepository = TokensRepository(
        assetsDao = assetsDao,
        pricesDao = pricesDao,
        pricesRepository = pricesRepository,
        sessionRepository = sessionRepository,
        searchService = searchService,
        assetsService = assetsService,
    )

    @Provides
    @Singleton
    fun provideSearchTokensCase(tokensRepository: TokensRepository): SearchTokensCase = tokensRepository

    @Provides
    @Singleton
    fun provideGemSearchStore(searchDao: SearchDao, assetListDao: AssetListDao): GemSearchStore = GemstoneSearchStore(searchDao, assetListDao)

    @Provides
    @Singleton
    fun provideWalletSearchTokens(
        tokensRepository: TokensRepository,
        searchService: GemSearchService,
        sessionRepository: SessionRepository,
    ): WalletSearchTokens = WalletSearchTokens(tokensRepository, searchService, sessionRepository)

    @Provides
    @Singleton
    @WalletSearch
    fun provideWalletSearchTokensCase(walletSearchTokens: WalletSearchTokens): SearchTokensCase = walletSearchTokens

    @Provides
    @Singleton
    fun provideWalletSearchScopeCase(walletSearchTokens: WalletSearchTokens): WalletSearchScopeCase = walletSearchTokens

    @Provides
    @Singleton
    fun provideSyncAssetPrices(tokensRepository: TokensRepository): SyncAssetPrices = tokensRepository
}
