package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.receive.cases.GetReceiveAssetInfo
import com.gemwallet.android.data.coordinators.receive.GetReceiveAssetInfoImpl
import com.gemwallet.android.application.session.cases.GetSession
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
        getSession: GetSession,
        getAssetTokenInfo: GetAssetTokenInfo,
    ): GetReceiveAssetInfo = GetReceiveAssetInfoImpl(getSession, getAssetTokenInfo)
}
