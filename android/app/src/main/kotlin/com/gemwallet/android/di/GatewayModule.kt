package com.gemwallet.android.di

import android.content.Context
import com.gemwallet.android.Constants
import com.gemwallet.android.NodeAuthInterceptor
import com.gemwallet.android.NodeAuthTokenService
import com.gemwallet.android.blockchain.services.ServiceStatusService
import com.gemwallet.android.cases.device.IsDeviceRegistered
import com.gemwallet.android.cases.nodes.GetNodeUrlCase
import com.gemwallet.android.data.password.TinkGemPreferences
import com.gemwallet.android.data.repositories.config.SharedGemPreferences
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.gemwallet.android.data.services.gemapi.NativeProvider
import com.gemwallet.android.data.services.gemapi.NativeProviderConfig
import com.gemwallet.android.ui.R as UiR
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import okhttp3.OkHttpClient
import java.util.concurrent.TimeUnit
import uniffi.gemstone.AlienProvider
import uniffi.gemstone.GemGateway
import uniffi.gemstone.GemPreferences
import uniffi.gemstone.PaymentService
import uniffi.gemstone.PaymentServiceInterface
import uniffi.gemstone.GemServiceStatus
import uniffi.gemstone.serviceStatusTimeoutSeconds
import uniffi.gemstone.TransactionSimulationService
import uniffi.gemstone.TransactionSimulationServiceInterface
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object GatewayModule {

    @Singleton
    @Provides
    fun provideGemPreferences(@ApplicationContext context: Context): GemPreferences = TinkGemPreferences(context)

    @Singleton
    @Provides
    fun provideNodeAuthTokenService(
        deviceApiClient: GemDeviceApiClient,
        isDeviceRegistered: IsDeviceRegistered,
        preferences: GemPreferences,
    ): NodeAuthTokenService = NodeAuthTokenService(deviceApiClient, isDeviceRegistered, preferences)

    @Singleton
    @Provides
    fun provideAlienProvider(
        getNodeUrlCase: GetNodeUrlCase,
        okHttpClient: OkHttpClient,
        nodeAuthInterceptor: NodeAuthInterceptor,
        @ApplicationContext context: Context,
    ): AlienProvider {
        return NativeProvider(
            getNodeUrlCase = getNodeUrlCase,
            httpClient = okHttpClient.newBuilder().addInterceptor(nodeAuthInterceptor).build(),
            config = NativeProviderConfig(
                networkOfflineMessage = context.getString(UiR.string.errors_network_offline),
            ),
        )
    }

    @Provides
    @Singleton
    fun provideGateway(
        alienProvider: AlienProvider,
        securePreferences: GemPreferences,
        @ApplicationContext context: Context,
    ): GemGateway {
        return GemGateway(
            alienProvider,
            preferences = SharedGemPreferences(
                sharedPreferences = context.getSharedPreferences("gateway_preferences", Context.MODE_PRIVATE)
            ),
            securePreferences = securePreferences,
        )
    }

    @Provides
    @Singleton
    fun providePaymentService(alienProvider: AlienProvider): PaymentServiceInterface = PaymentService(alienProvider)

    @Singleton
    @Provides
    fun provideNodeAuthInterceptor(preferences: GemPreferences): NodeAuthInterceptor = NodeAuthInterceptor(preferences)

    @Provides
    @Singleton
    fun provideServiceStatusService(
        getNodeUrlCase: GetNodeUrlCase,
        okHttpClient: OkHttpClient,
        nodeAuthInterceptor: NodeAuthInterceptor,
        @ApplicationContext context: Context,
    ): ServiceStatusService {
        val httpClient = okHttpClient.newBuilder()
            .addInterceptor(nodeAuthInterceptor)
            .callTimeout(serviceStatusTimeoutSeconds().toLong(), TimeUnit.SECONDS)
            .build()
        val provider = NativeProvider(
            getNodeUrlCase = getNodeUrlCase,
            httpClient = httpClient,
            config = NativeProviderConfig(
                networkOfflineMessage = context.getString(UiR.string.errors_network_offline),
            ),
        )
        return ServiceStatusService(GemServiceStatus(provider))
    }

    @Provides
    @Singleton
    fun provideGemTransactionSimulationService(
        alienProvider: AlienProvider,
    ): TransactionSimulationService = TransactionSimulationService(alienProvider)

    @Provides
    @Singleton
    fun provideTransactionSimulationServiceInterface(
        service: TransactionSimulationService,
    ): TransactionSimulationServiceInterface = service
}
