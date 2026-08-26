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
import com.gemwallet.android.cases.device.SyncDevice
import com.gemwallet.android.services.DeviceSyncPreflight
import com.gemwallet.android.application.device.coordinators.GetDeviceId
import com.gemwallet.android.math.fromHex
import kotlinx.coroutines.runBlocking
import com.gemwallet.android.data.services.gemapi.NativeProvider
import com.gemwallet.android.data.services.gemapi.NativeProviderConfig
import com.gemwallet.android.ui.R as UiR
import dagger.Lazy
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import okhttp3.OkHttpClient
import java.util.concurrent.TimeUnit
import uniffi.gemstone.AlienProvider
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemChartService
import uniffi.gemstone.GemConfigService
import uniffi.gemstone.GemPriceService
import uniffi.gemstone.GemAuthService
import uniffi.gemstone.GemDeviceApiClient as GemstoneDeviceApiClient
import uniffi.gemstone.GemDeviceService
import uniffi.gemstone.GemFiatService
import uniffi.gemstone.GemNameService
import uniffi.gemstone.GemNotificationService
import uniffi.gemstone.GemPortfolioService
import uniffi.gemstone.GemPriceAlertService
import uniffi.gemstone.GemRewardsService
import uniffi.gemstone.GemSubscriptionService
import uniffi.gemstone.GemSupportService
import uniffi.gemstone.GemTransactionsService
import uniffi.gemstone.GemWalletConfigurationService
import javax.inject.Named
import uniffi.gemstone.GemGateway
import uniffi.gemstone.GemApiClient as GemstoneApiClient
import uniffi.gemstone.GemNftService
import uniffi.gemstone.GemScanService
import uniffi.gemstone.GemStaticApiClient
import uniffi.gemstone.GemStaticAssetsService
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
        deviceService: GemDeviceService,
        isDeviceRegistered: IsDeviceRegistered,
        preferences: GemPreferences,
    ): NodeAuthTokenService = NodeAuthTokenService(deviceService, isDeviceRegistered, preferences)

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
    fun provideGemstoneApiClient(alienProvider: AlienProvider): GemstoneApiClient =
        GemstoneApiClient(alienProvider, Constants.API_URL)

    @Provides
    @Singleton
    @Named("registration")
    fun provideDeviceRegistrationApiClient(
        alienProvider: AlienProvider,
        getDeviceId: GetDeviceId,
    ): GemstoneDeviceApiClient = GemstoneDeviceApiClient(
        alienProvider,
        Constants.API_URL,
        runBlocking { getDeviceId.getDeviceKey().fromHex() },
    )

    @Provides
    @Singleton
    fun provideGemstoneDeviceApiClient(
        alienProvider: AlienProvider,
        getDeviceId: GetDeviceId,
        syncDevice: Lazy<SyncDevice>,
    ): GemstoneDeviceApiClient = GemstoneDeviceApiClient.withPreflight(
        alienProvider,
        Constants.API_URL,
        runBlocking { getDeviceId.getDeviceKey().fromHex() },
        DeviceSyncPreflight(syncDevice),
    )

    @Provides
    @Singleton
    fun provideGemDeviceService(@Named("registration") apiClient: GemstoneDeviceApiClient): GemDeviceService = GemDeviceService(apiClient)

    @Provides
    @Singleton
    fun provideGemSubscriptionService(@Named("registration") apiClient: GemstoneDeviceApiClient): GemSubscriptionService = GemSubscriptionService(apiClient)

    @Provides
    @Singleton
    fun provideGemAuthService(apiClient: GemstoneDeviceApiClient): GemAuthService = GemAuthService(apiClient)

    @Provides
    @Singleton
    fun provideGemTransactionsService(apiClient: GemstoneDeviceApiClient): GemTransactionsService = GemTransactionsService(apiClient)

    @Provides
    @Singleton
    fun provideGemWalletConfigurationService(apiClient: GemstoneDeviceApiClient): GemWalletConfigurationService = GemWalletConfigurationService(apiClient)

    @Provides
    @Singleton
    fun provideGemPriceAlertService(apiClient: GemstoneDeviceApiClient): GemPriceAlertService = GemPriceAlertService(apiClient)

    @Provides
    @Singleton
    fun provideGemSupportService(apiClient: GemstoneDeviceApiClient): GemSupportService = GemSupportService(apiClient)

    @Provides
    @Singleton
    fun provideGemRewardsService(apiClient: GemstoneDeviceApiClient): GemRewardsService = GemRewardsService(apiClient)

    @Provides
    @Singleton
    fun provideGemNotificationService(apiClient: GemstoneDeviceApiClient): GemNotificationService = GemNotificationService(apiClient)

    @Provides
    @Singleton
    fun provideGemFiatService(apiClient: GemstoneDeviceApiClient): GemFiatService = GemFiatService(apiClient)

    @Provides
    @Singleton
    fun provideGemNameService(apiClient: GemstoneDeviceApiClient): GemNameService = GemNameService(apiClient)

    @Provides
    @Singleton
    fun provideGemPortfolioService(apiClient: GemstoneDeviceApiClient): GemPortfolioService = GemPortfolioService(apiClient)

    @Provides
    @Singleton
    fun provideGemStaticApiClient(alienProvider: AlienProvider): GemStaticApiClient =
        GemStaticApiClient(alienProvider, Constants.ASSETS_URL)


    @Provides
    @Singleton
    fun provideGemChartService(apiClient: GemstoneApiClient): GemChartService = GemChartService(apiClient)

    @Provides
    @Singleton
    fun provideGemConfigService(apiClient: GemstoneApiClient): GemConfigService = GemConfigService(apiClient)

    @Provides
    @Singleton
    fun provideGemAssetsService(apiClient: GemstoneApiClient): GemAssetsService = GemAssetsService(apiClient)

    @Provides
    @Singleton
    fun provideGemPriceService(apiClient: GemstoneApiClient): GemPriceService = GemPriceService(apiClient)

    @Provides
    @Singleton
    fun provideGemScanService(apiClient: GemstoneDeviceApiClient): GemScanService = GemScanService(apiClient)

    @Provides
    @Singleton
    fun provideGemNftService(apiClient: GemstoneDeviceApiClient): GemNftService = GemNftService(apiClient)

    @Provides
    @Singleton
    fun provideGemStaticAssetsService(apiClient: GemStaticApiClient): GemStaticAssetsService =
        GemStaticAssetsService(apiClient)

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
