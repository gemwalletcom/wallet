package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.support.cases.ClearSupportTyping
import com.gemwallet.android.application.support.cases.GetSupportTyping
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
    fun provideSupportTypingCoordinator(supportStore: GemstoneSupportStore): SupportTypingCoordinator = SupportTypingCoordinator(supportStore)

    @Provides
    @Singleton
    fun provideGetSupportTyping(coordinator: SupportTypingCoordinator): GetSupportTyping = coordinator

    @Provides
    @Singleton
    fun provideClearSupportTyping(coordinator: SupportTypingCoordinator): ClearSupportTyping = coordinator
}
