package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.assets.cases.EnableAsset
import com.gemwallet.android.application.receive.cases.GetReceiveAssetInfo
import com.gemwallet.android.application.receive.cases.SetAssetVisible
import com.gemwallet.android.data.coordinators.receive.GetReceiveAssetInfoImpl
import com.gemwallet.android.data.coordinators.receive.SetAssetVisibleImpl
import com.gemwallet.android.data.repositories.session.SessionRepository
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton
import com.gemwallet.android.application.assets.cases.GetAssetTokenInfo

@InstallIn(SingletonComponent::class)
@Module
object ReceiveModule {

    @Provides
    @Singleton
    fun provideGetReceiveAssetInfo(
        sessionRepository: SessionRepository,
        getAssetTokenInfo: GetAssetTokenInfo,
    ): GetReceiveAssetInfo = GetReceiveAssetInfoImpl(sessionRepository, getAssetTokenInfo)

    @Provides
    @Singleton
    fun provideSetAssetVisible(
        sessionRepository: SessionRepository,
        enableAsset: EnableAsset,
    ): SetAssetVisible {
        return SetAssetVisibleImpl(sessionRepository, enableAsset)
    }
}
