package com.gemwallet.android.di

import android.content.Context
import com.gemwallet.android.Constants
import com.gemwallet.android.cases.nodes.GetNodeUrlCase
import com.gemwallet.android.data.repositories.config.SharedGemPreferences
import com.gemwallet.android.data.password.TinkGemPreferences
import com.gemwallet.android.data.services.gemapi.NativeProvider
import com.gemwallet.android.data.services.gemapi.NativeProviderConfig
import com.gemwallet.android.ui.R as UiR
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import okhttp3.OkHttpClient
import uniffi.gemstone.AlienProvider
import uniffi.gemstone.GemGateway
import uniffi.gemstone.WalletConnectSimulationClient
import uniffi.gemstone.WalletConnectSimulationClientInterface
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object GatewayModule {

    @Singleton
    @Provides
    fun provideAlienProvider(
        getNodeUrlCase: GetNodeUrlCase,
        okHttpClient: OkHttpClient,
        @ApplicationContext context: Context,
    ): AlienProvider {
        return NativeProvider(
            getNodeUrlCase = getNodeUrlCase,
            httpClient = okHttpClient,
            config = NativeProviderConfig(
                networkOfflineMessage = context.getString(UiR.string.errors_network_offline),
            ),
        )
    }

    @Provides
    @Singleton
    fun provideGateway(
        alienProvider: AlienProvider,
        @ApplicationContext context: Context,
    ): GemGateway {
        return GemGateway(
            alienProvider,
            preferences = SharedGemPreferences(
                sharedPreferences = context.getSharedPreferences("gateway_preferences", Context.MODE_PRIVATE)
            ),
            securePreferences = TinkGemPreferences(context),
            apiUrl = Constants.API_URL
        )
    }

    @Provides
    @Singleton
    fun provideWalletConnectSimulationService(
        alienProvider: AlienProvider,
    ): com.gemwallet.android.blockchain.services.WalletConnectSimulationService =
        com.gemwallet.android.blockchain.services.WalletConnectSimulationService(WalletConnectSimulationClient(alienProvider))
}
