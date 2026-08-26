package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.cases.name.ResolveName
import com.gemwallet.android.data.coordinators.name.ResolveNameImpl
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemNameService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object NameModule {

    @Provides
    @Singleton
    fun provideResolveName(
        nameService: GemNameService,
    ): ResolveName {
        return ResolveNameImpl(nameService)
    }
}
