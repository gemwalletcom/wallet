package com.gemwallet.android.di

import android.content.Context
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.SecurityStore
import com.gemwallet.android.application.wallet_import.coordinators.SyncWalletImport
import com.gemwallet.android.blockchain.operators.CreateWalletOperator
import com.gemwallet.android.blockchain.operators.DeleteKeyStoreOperator
import com.gemwallet.android.blockchain.operators.LoadPrivateDataOperator
import com.gemwallet.android.blockchain.operators.MigrateKeystoreOperator
import com.gemwallet.android.blockchain.operators.ValidatePhraseOperator
import com.gemwallet.android.blockchain.operators.gemstone.GemCreateWalletOperator
import com.gemwallet.android.blockchain.operators.gemstone.GemDeleteKeyStoreOperator
import com.gemwallet.android.blockchain.operators.gemstone.GemLoadPrivateDataOperator
import com.gemwallet.android.blockchain.operators.gemstone.GemMigrateKeystoreOperator
import com.gemwallet.android.blockchain.operators.gemstone.GemValidatePhraseOperator
import com.gemwallet.android.blockchain.services.GemSignMessageOperator
import com.gemwallet.android.blockchain.services.GemSignTransactionOperator
import com.gemwallet.android.cases.device.SyncDevice
import com.gemwallet.android.cases.wallet.ImportWalletService
import com.gemwallet.android.data.password.TinkPasswordStore
import com.gemwallet.android.data.password.TinkSecurityStore
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.wallets.PhraseAddressImportWalletService
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAppStartService
import uniffi.gemstone.GemWalletService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object InteractsModule {

    @Singleton
    @Provides
    fun provideValidatePhraseInteract(): ValidatePhraseOperator = GemValidatePhraseOperator()

    @Singleton
    @Provides
    fun provideCreateWalletInteract(): CreateWalletOperator = GemCreateWalletOperator()

    @Singleton
    @Provides
    fun provideMigrateKeystoreOperator(
        @ApplicationContext context: Context,
    ): MigrateKeystoreOperator = GemMigrateKeystoreOperator(context.dataDir.toString())

    @Singleton
    @Provides
    fun provideLoadPhraseInteract(
        @ApplicationContext context: Context
    ): LoadPrivateDataOperator =
        GemLoadPrivateDataOperator(context.dataDir.toString())

    @Singleton
    @Provides
    fun provideSignTransactionOperator(
        @ApplicationContext context: Context,
    ): GemSignTransactionOperator = GemSignTransactionOperator(context.dataDir.toString())

    @Singleton
    @Provides
    fun provideSignMessageOperator(
        @ApplicationContext context: Context,
    ): GemSignMessageOperator = GemSignMessageOperator(context.dataDir.toString())


    @Singleton
    @Provides
    fun provideDeleteKeyStoreOperator(
        @ApplicationContext context: Context,
        passwordStore: PasswordStore,
    ): DeleteKeyStoreOperator = GemDeleteKeyStoreOperator(context.dataDir.toString(), passwordStore)

    @Provides
    @Singleton
    fun providePasswordStore(@ApplicationContext context: Context): PasswordStore =
        TinkPasswordStore(context)

    @Provides
    @Singleton
    fun provideSecurityStore(@ApplicationContext context: Context): SecurityStore<Any> =
        TinkSecurityStore(context)

    @Singleton
    @Provides
    fun provideAddWalletInteract(
        walletService: GemWalletService,
        sessionRepository: SessionRepository,
        appStartService: GemAppStartService,
        syncDevice: SyncDevice,
        walletImportSync: SyncWalletImport,
    ): ImportWalletService = PhraseAddressImportWalletService(
        walletService = walletService,
        sessionRepository = sessionRepository,
        appStartService = appStartService,
        syncDevice = syncDevice,
        walletImportSync = walletImportSync,
    )
}
