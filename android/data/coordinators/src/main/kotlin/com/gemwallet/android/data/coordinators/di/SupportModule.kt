package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.support.cases.ClearSupportTyping
import com.gemwallet.android.application.support.cases.FailPendingSupportMessages
import com.gemwallet.android.application.support.cases.GetSupportMessages
import com.gemwallet.android.application.support.cases.GetSupportTyping
import com.gemwallet.android.data.coordinators.support.FailPendingSupportMessagesImpl
import com.gemwallet.android.data.coordinators.support.GetSupportMessagesImpl
import com.gemwallet.android.data.coordinators.support.SupportTypingCoordinator
import com.gemwallet.android.data.services.gemstone.stores.GemstoneSupportStore
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
    fun provideGetSupportMessages(supportStore: GemstoneSupportStore): GetSupportMessages =
        GetSupportMessagesImpl(supportStore)

    @Provides
    @Singleton
    fun provideFailPendingSupportMessages(supportStore: GemstoneSupportStore): FailPendingSupportMessages =
        FailPendingSupportMessagesImpl(supportStore)

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
