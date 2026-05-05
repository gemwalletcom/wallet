package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.add_asset.coordinators.AddCustomToken
import com.gemwallet.android.application.add_asset.coordinators.GetAvailableTokenChains
import com.gemwallet.android.application.add_asset.coordinators.ObserveToken
import com.gemwallet.android.application.add_asset.coordinators.SearchCustomToken
import com.gemwallet.android.application.assets.coordinators.EnableAsset
import com.gemwallet.android.data.coordinators.add_asset.AddCustomTokenImpl
import com.gemwallet.android.data.coordinators.add_asset.GetAvailableTokenChainsImpl
import com.gemwallet.android.data.coordinators.add_asset.ObserveTokenImpl
import com.gemwallet.android.data.coordinators.add_asset.SearchCustomTokenImpl
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

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
        assetsRepository: AssetsRepository,
    ): SearchCustomToken {
        return SearchCustomTokenImpl(sessionRepository, assetsRepository)
    }

    @Provides
    @Singleton
    fun provideObserveToken(
        assetsRepository: AssetsRepository,
    ): ObserveToken {
        return ObserveTokenImpl(assetsRepository)
    }

    @Provides
    @Singleton
    fun provideAddCustomToken(
        sessionRepository: SessionRepository,
        enableAsset: EnableAsset,
    ): AddCustomToken {
        return AddCustomTokenImpl(sessionRepository, enableAsset)
    }
}
