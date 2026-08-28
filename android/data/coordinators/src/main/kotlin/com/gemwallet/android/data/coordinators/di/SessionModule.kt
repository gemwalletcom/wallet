package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.session.cases.SetCurrentCurrency
import com.gemwallet.android.data.coordinators.session.GetSessionImpl
import com.gemwallet.android.data.coordinators.session.SetCurrentCurrencyImpl
import com.gemwallet.android.data.repositories.session.SessionRepository
import uniffi.gemstone.GemPriceService
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton
import uniffi.gemstone.GemDeviceService

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
        priceService: GemPriceService,
        deviceService: GemDeviceService,
    ): SetCurrentCurrency {
        return SetCurrentCurrencyImpl(
            sessionRepository = sessionRepository,
            priceService = priceService,
            deviceService = deviceService,
        )
    }
}
