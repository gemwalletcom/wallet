package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.receive.coordinators.GetReceiveAssetInfo
import com.gemwallet.android.application.receive.coordinators.SetAssetVisible
import com.gemwallet.android.data.coordinators.receive.GetReceiveAssetInfoImpl
import com.gemwallet.android.data.coordinators.receive.SetAssetVisibleImpl
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object ReceiveModule {

    @Provides
    @Singleton
    fun provideGetReceiveAssetInfo(
        sessionRepository: SessionRepository,
        assetsRepository: AssetsRepository,
    ): GetReceiveAssetInfo {
        return GetReceiveAssetInfoImpl(sessionRepository, assetsRepository)
    }

    @Provides
    @Singleton
    fun provideSetAssetVisible(
        sessionRepository: SessionRepository,
        assetsRepository: AssetsRepository,
    ): SetAssetVisible {
        return SetAssetVisibleImpl(sessionRepository, assetsRepository)
    }
}
