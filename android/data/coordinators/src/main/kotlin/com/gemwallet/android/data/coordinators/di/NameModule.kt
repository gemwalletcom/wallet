package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.recipient.cases.GetNameRecord
import com.gemwallet.android.data.coordinators.name.GetNameRecordImpl
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
    fun provideGetNameRecord(
        nameService: GemNameService,
    ): GetNameRecord {
        return GetNameRecordImpl(nameService)
    }
}
