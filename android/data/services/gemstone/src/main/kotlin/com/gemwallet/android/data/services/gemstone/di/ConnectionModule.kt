package com.gemwallet.android.data.services.gemstone.di

import android.content.Context
import com.gemwallet.android.data.services.gemstone.connection.ConnectionComponentHealth
import com.gemwallet.android.data.services.gemstone.connection.ConnectionStatusObserver
import com.gemwallet.android.data.services.gemstone.connection.InternetConnectionMonitor
import com.wallet.core.primitives.ConnectionComponent
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemConnectionService
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
object ConnectionModule {

    @Provides
    @Singleton
    fun provideStreamHealth(): ConnectionComponentHealth = ConnectionComponentHealth(ConnectionComponent.Stream)

    @Provides
    @Singleton
    fun provideConnectionStatusObserver(@ApplicationContext context: Context, connectionService: GemConnectionService, streamHealth: ConnectionComponentHealth): ConnectionStatusObserver = ConnectionStatusObserver(
        monitors = listOf(
            InternetConnectionMonitor(context),
            streamHealth,
        ),
        connectionService = connectionService,
    )
}
