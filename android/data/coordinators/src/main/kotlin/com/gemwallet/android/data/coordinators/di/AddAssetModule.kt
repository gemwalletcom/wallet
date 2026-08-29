package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.add_asset.cases.AddCustomToken
import com.gemwallet.android.application.add_asset.cases.GetAvailableTokenChains
import com.gemwallet.android.application.add_asset.cases.ObserveToken
import com.gemwallet.android.application.add_asset.cases.SearchCustomToken
import com.gemwallet.android.application.assets.cases.EnableAsset
import com.gemwallet.android.data.coordinators.add_asset.AddCustomTokenImpl
import com.gemwallet.android.data.coordinators.add_asset.GetAvailableTokenChainsImpl
import com.gemwallet.android.data.coordinators.add_asset.ObserveTokenImpl
import com.gemwallet.android.data.coordinators.add_asset.SearchCustomTokenImpl
import com.gemwallet.android.application.tokens.cases.SearchTokens
import com.gemwallet.android.application.session.cases.GetSession
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton
import com.gemwallet.android.data.adapters.gemstone.GemstoneAssetStore
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.application.session.cases.GetCurrentCurrency

@InstallIn(SingletonComponent::class)
@Module
object AddAssetModule {

    @Provides
    @Singleton
    fun provideGetAvailableTokenChains(
        getSession: GetSession,
    ): GetAvailableTokenChains {
        return GetAvailableTokenChainsImpl(getSession)
    }

    @Provides
    @Singleton
    fun provideSearchCustomToken(
        getCurrentCurrency: GetCurrentCurrency,
        searchTokensCase: SearchTokens,
    ): SearchCustomToken = SearchCustomTokenImpl(getCurrentCurrency, searchTokensCase)

    @Provides
    @Singleton
    fun provideObserveToken(
        assetStore: GemstoneAssetStore,
        getCurrentWalletId: GetCurrentWalletId,
    ): ObserveToken = ObserveTokenImpl(assetStore, getCurrentWalletId)

    @Provides
    @Singleton
    fun provideAddCustomToken(
        getSession: GetSession,
        enableAsset: EnableAsset,
    ): AddCustomToken {
        return AddCustomTokenImpl(getSession, enableAsset)
    }
}
