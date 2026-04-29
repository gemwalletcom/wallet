package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.application.assets.coordinators.SearchAssets
import com.gemwallet.android.blockchain.services.TokenService
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.cases.tokens.SyncAssetPrices
import com.gemwallet.android.data.repositories.tokens.TokensRepository
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.AssetsPriorityDao
import com.gemwallet.android.data.service.store.database.PricesDao
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
        assetsPriorityDao: AssetsPriorityDao,
        gateway: GemGateway,
        searchAssets: SearchAssets,
    ): TokensRepository = TokensRepository(
        assetsDao = assetsDao,
        pricesDao = pricesDao,
        assetsPriorityDao = assetsPriorityDao,
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
    fun provideSyncAssetPrices(tokensRepository: TokensRepository): SyncAssetPrices = tokensRepository
}
