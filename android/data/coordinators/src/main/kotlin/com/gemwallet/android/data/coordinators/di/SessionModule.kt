package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.session.coordinators.GetSession
import com.gemwallet.android.application.session.coordinators.SetCurrentCurrency
import com.gemwallet.android.cases.device.SyncDeviceInfo
import com.gemwallet.android.data.coordinators.session.GetSessionImpl
import com.gemwallet.android.data.coordinators.session.SetCurrentCurrencyImpl
import com.gemwallet.android.data.repositories.session.SessionRepository
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object SessionModule {

    @Provides
    @Singleton
    fun provideGetSession(
        sessionRepository: SessionRepository,
    ): GetSession = GetSessionImpl(sessionRepository)

    @Provides
    @Singleton
    fun provideSetCurrentCurrency(
        sessionRepository: SessionRepository,
        syncDeviceInfo: SyncDeviceInfo,
    ): SetCurrentCurrency {
        return SetCurrentCurrencyImpl(
            sessionRepository = sessionRepository,
            syncDeviceInfo = syncDeviceInfo,
        )
    }
}
