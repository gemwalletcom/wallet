package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.tokens.cases.SearchTokens
import com.gemwallet.android.application.tokens.cases.WalletSearchScope
import com.gemwallet.android.data.coordinators.tokens.SearchTokensImpl
import com.gemwallet.android.data.coordinators.tokens.WalletSearchTokens
import com.gemwallet.android.application.session.cases.GetSession
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
    fun provideSearchTokensImpl(
        getSession: GetSession,
        searchService: GemSearchService,
        assetsService: GemAssetsService,
    ): SearchTokensImpl = SearchTokensImpl(getSession, searchService, assetsService)

    @Provides
    @Singleton
    fun provideSearchTokens(searchTokens: SearchTokensImpl): SearchTokens = searchTokens

    @Provides
    @Singleton
    fun provideWalletSearchTokens(
        searchTokens: SearchTokensImpl,
        searchService: GemSearchService,
        getSession: GetSession,
    ): WalletSearchTokens = WalletSearchTokens(searchTokens, searchService, getSession)

    @Provides
    @Singleton
    @WalletSearch
    fun provideWalletSearchTokensCase(walletSearchTokens: WalletSearchTokens): SearchTokens = walletSearchTokens

    @Provides
    @Singleton
    fun provideWalletSearchScope(walletSearchTokens: WalletSearchTokens): WalletSearchScope = walletSearchTokens
}
