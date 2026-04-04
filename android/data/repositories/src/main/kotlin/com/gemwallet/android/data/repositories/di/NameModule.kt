package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.cases.name.ResolveNameCase
import com.gemwallet.android.data.repositories.name.NameRepository
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
class NameModule {

    @Provides
    @Singleton
    fun provideNameRepository(
        gemDeviceApiClient: GemDeviceApiClient,
    ): NameRepository {
        return NameRepository(gemDeviceApiClient)
    }

    @Provides
    fun provideResolveNameCase(nameRepository: NameRepository): ResolveNameCase = nameRepository
}
