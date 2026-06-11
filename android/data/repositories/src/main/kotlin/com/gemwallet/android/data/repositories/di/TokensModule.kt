package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.application.assets.coordinators.GemSearch
import com.gemwallet.android.application.assets.coordinators.SearchAssets
import com.gemwallet.android.blockchain.services.TokenService
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.cases.tokens.SyncAssetPrices
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.data.repositories.tokens.TokensRepository
import com.gemwallet.android.data.repositories.tokens.WalletSearch
import com.gemwallet.android.data.repositories.tokens.WalletSearchTokens
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.service.store.database.SearchDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemGateway
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object TokensModule {
    @Provides
    @Singleton
    fun provideTokensRepository(
        assetsDao: AssetsDao,
        pricesDao: PricesDao,
        searchDao: SearchDao,
        gateway: GemGateway,
        searchAssets: SearchAssets,
    ): TokensRepository = TokensRepository(
        assetsDao = assetsDao,
        pricesDao = pricesDao,
        searchDao = searchDao,
        searchAssets = searchAssets,
        tokenService = TokenService(
            gateway = gateway,
        ),
    )

    @Provides
    @Singleton
    fun provideSearchTokensCase(tokensRepository: TokensRepository): SearchTokensCase = tokensRepository

    @Provides
    @Singleton
    @WalletSearch
    fun provideWalletSearchTokensCase(
        tokensRepository: TokensRepository,
        gemSearch: GemSearch,
        perpetualRepository: PerpetualRepository,
        searchDao: SearchDao,
    ): SearchTokensCase = WalletSearchTokens(tokensRepository, gemSearch, perpetualRepository, searchDao)

    @Provides
    @Singleton
    fun provideSyncAssetPrices(tokensRepository: TokensRepository): SyncAssetPrices = tokensRepository
}
