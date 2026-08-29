package com.gemwallet.android.data.adapters.di

import android.content.Context
import com.gemwallet.android.data.adapters.connection.ConnectionStatusObserver
import com.gemwallet.android.data.adapters.connection.InternetConnectionMonitor
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
object ConnectionModule {

    @Provides
    @Singleton
    fun provideConnectionStatusObserver(
        @ApplicationContext context: Context,
    ): ConnectionStatusObserver = ConnectionStatusObserver(
        monitors = listOf(
            InternetConnectionMonitor(context),
        ),
    )
}
