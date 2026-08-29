package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.support.cases.ClearSupportTyping
import com.gemwallet.android.application.support.cases.FailPendingSupportMessages
import com.gemwallet.android.application.support.cases.GetSupportMessages
import com.gemwallet.android.application.support.cases.GetSupportTyping
import com.gemwallet.android.data.coordinators.support.FailPendingSupportMessagesImpl
import com.gemwallet.android.data.coordinators.support.GetSupportMessagesImpl
import com.gemwallet.android.data.coordinators.support.SupportTypingCoordinator
import com.gemwallet.android.data.repositories.gemstone.GemstoneSupportStore
import com.gemwallet.android.data.service.store.database.SupportMessagesDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object SupportModule {

    @Provides
    @Singleton
    fun provideGetSupportMessages(supportMessagesDao: SupportMessagesDao): GetSupportMessages =
        GetSupportMessagesImpl(supportMessagesDao)

    @Provides
    @Singleton
    fun provideFailPendingSupportMessages(supportMessagesDao: SupportMessagesDao): FailPendingSupportMessages =
        FailPendingSupportMessagesImpl(supportMessagesDao)

    @Provides
    @Singleton
    fun provideSupportTypingCoordinator(supportStore: GemstoneSupportStore): SupportTypingCoordinator =
        SupportTypingCoordinator(supportStore)

    @Provides
    @Singleton
    fun provideGetSupportTyping(coordinator: SupportTypingCoordinator): GetSupportTyping = coordinator

    @Provides
    @Singleton
    fun provideClearSupportTyping(coordinator: SupportTypingCoordinator): ClearSupportTyping = coordinator
}
