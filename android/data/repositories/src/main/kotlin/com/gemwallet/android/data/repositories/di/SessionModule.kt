package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.application.session.coordinators.GetCurrentCurrency
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.session.SessionRepositoryImpl
import com.gemwallet.android.data.service.store.database.SessionDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object SessionModule {
    @Singleton
    @Provides
    fun provideSessionRepository(
        sessionDao: SessionDao,
        walletsRepository: com.gemwallet.android.data.repositories.wallets.WalletsRepository,
    ): SessionRepository = SessionRepositoryImpl(
        sessionDao = sessionDao,
        walletsRepository = walletsRepository
    )

    @Provides
    fun provideGetCurrentCurrency(sessionRepository: SessionRepository): GetCurrentCurrency = sessionRepository
}
