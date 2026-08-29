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
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.data.repositories.session.SessionRepository
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton
import com.gemwallet.android.data.repositories.gemstone.GemstoneAssetStore
import com.gemwallet.android.application.session.cases.GetCurrentWalletId

@InstallIn(SingletonComponent::class)
@Module
object AddAssetModule {

    @Provides
    @Singleton
    fun provideGetAvailableTokenChains(
        sessionRepository: SessionRepository,
    ): GetAvailableTokenChains {
        return GetAvailableTokenChainsImpl(sessionRepository)
    }

    @Provides
    @Singleton
    fun provideSearchCustomToken(
        sessionRepository: SessionRepository,
        searchTokensCase: SearchTokensCase,
    ): SearchCustomToken = SearchCustomTokenImpl(sessionRepository, searchTokensCase)

    @Provides
    @Singleton
    fun provideObserveToken(
        assetStore: GemstoneAssetStore,
        getCurrentWalletId: GetCurrentWalletId,
    ): ObserveToken = ObserveTokenImpl(assetStore, getCurrentWalletId)

    @Provides
    @Singleton
    fun provideAddCustomToken(
        sessionRepository: SessionRepository,
        enableAsset: EnableAsset,
    ): AddCustomToken {
        return AddCustomTokenImpl(sessionRepository, enableAsset)
    }
}
