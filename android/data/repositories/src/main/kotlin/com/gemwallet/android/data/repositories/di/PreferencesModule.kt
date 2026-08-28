package com.gemwallet.android.data.repositories.di

import android.content.Context
import com.gemwallet.android.data.repositories.gemstone.GemstonePreferencesStore
import com.gemwallet.android.data.repositories.nodes.MigrateExplorerPreference
import com.gemwallet.android.data.repositories.pricealerts.MigratePriceAlertsPreference
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemExplorerService
import uniffi.gemstone.GemPreferencesService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object PreferencesModule {

    @Singleton
    @Provides
    fun provideGemstonePreferencesStore(@ApplicationContext context: Context): GemstonePreferencesStore =
        GemstonePreferencesStore(context.getSharedPreferences("gemstone_preferences", Context.MODE_PRIVATE))

    @Singleton
    @Provides
    fun provideGemPreferencesService(store: GemstonePreferencesStore): GemPreferencesService = GemPreferencesService(store)

    @Singleton
    @Provides
    fun provideGemExplorerService(preferencesService: GemPreferencesService): GemExplorerService = GemExplorerService(preferencesService)

    @Singleton
    @Provides
    fun provideMigrateExplorerPreference(
        @ApplicationContext context: Context,
        explorerService: GemExplorerService,
    ): MigrateExplorerPreference = MigrateExplorerPreference(context, explorerService)

    @Singleton
    @Provides
    fun provideMigratePriceAlertsPreference(
        @ApplicationContext context: Context,
        preferencesService: GemPreferencesService,
    ): MigratePriceAlertsPreference = MigratePriceAlertsPreference(context, preferencesService)
}
