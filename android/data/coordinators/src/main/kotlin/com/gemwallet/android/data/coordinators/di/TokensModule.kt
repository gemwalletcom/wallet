package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.cases.tokens.WalletSearchScopeCase
import com.gemwallet.android.data.coordinators.tokens.SearchTokensImpl
import com.gemwallet.android.data.coordinators.tokens.WalletSearchTokens
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.tokens.WalletSearch
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemSearchService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object TokensCasesModule {

    @Provides
    @Singleton
    fun provideSearchTokens(
        sessionRepository: SessionRepository,
        searchService: GemSearchService,
        assetsService: GemAssetsService,
    ): SearchTokensImpl = SearchTokensImpl(sessionRepository, searchService, assetsService)

    @Provides
    @Singleton
    fun provideSearchTokensCase(searchTokens: SearchTokensImpl): SearchTokensCase = searchTokens

    @Provides
    @Singleton
    fun provideWalletSearchTokens(
        searchTokens: SearchTokensImpl,
        searchService: GemSearchService,
        sessionRepository: SessionRepository,
    ): WalletSearchTokens = WalletSearchTokens(searchTokens, searchService, sessionRepository)

    @Provides
    @Singleton
    @WalletSearch
    fun provideWalletSearchTokensCase(walletSearchTokens: WalletSearchTokens): SearchTokensCase = walletSearchTokens

    @Provides
    @Singleton
    fun provideWalletSearchScopeCase(walletSearchTokens: WalletSearchTokens): WalletSearchScopeCase = walletSearchTokens
}
