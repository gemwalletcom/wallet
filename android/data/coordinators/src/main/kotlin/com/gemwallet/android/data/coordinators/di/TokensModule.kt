package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.tokens.cases.SearchTokens
import com.gemwallet.android.data.coordinators.tokens.SearchTokensImpl
import com.gemwallet.android.data.services.gemstone.di.IoDispatcher
import kotlinx.coroutines.CoroutineDispatcher
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAssetsService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object TokensCasesModule {

    @Provides
    @Singleton
    fun provideSearchTokens(
        assetsService: GemAssetsService,
        @IoDispatcher ioDispatcher: CoroutineDispatcher,
    ): SearchTokens = SearchTokensImpl(assetsService, ioDispatcher)
}
