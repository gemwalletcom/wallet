package com.gemwallet.android.di

import android.content.Context
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.SecurityStore
import com.gemwallet.android.application.wallet_import.coordinators.SyncWalletImport
import com.gemwallet.android.blockchain.operators.AddAccountsOperator
import com.gemwallet.android.blockchain.operators.CreateAccountOperator
import com.gemwallet.android.blockchain.operators.CreateWalletOperator
import com.gemwallet.android.blockchain.operators.DeleteKeyStoreOperator
import com.gemwallet.android.blockchain.operators.GemValidateAddressOperator
import com.gemwallet.android.blockchain.operators.LoadPrivateDataOperator
import com.gemwallet.android.blockchain.operators.MigrateKeystoreOperator
import com.gemwallet.android.blockchain.operators.StorePhraseOperator
import com.gemwallet.android.blockchain.operators.ValidateAddressOperator
import com.gemwallet.android.blockchain.operators.ValidatePhraseOperator
import com.gemwallet.android.blockchain.operators.gemstone.GemAddAccountsOperator
import com.gemwallet.android.blockchain.operators.gemstone.GemCreateAccountOperator
import com.gemwallet.android.blockchain.operators.gemstone.GemCreateWalletOperator
import com.gemwallet.android.blockchain.operators.gemstone.GemDeleteKeyStoreOperator
import com.gemwallet.android.blockchain.operators.gemstone.GemLoadPrivateDataOperator
import com.gemwallet.android.blockchain.operators.gemstone.GemMigrateKeystoreOperator
import com.gemwallet.android.blockchain.operators.gemstone.GemStorePhraseOperator
import com.gemwallet.android.blockchain.operators.gemstone.GemValidatePhraseOperator
import com.gemwallet.android.blockchain.services.GemSignAuthOperator
import com.gemwallet.android.blockchain.services.GemSignMessageOperator
import com.gemwallet.android.blockchain.services.GemSignTransactionOperator
import com.gemwallet.android.cases.device.SyncSubscription
import com.gemwallet.android.cases.wallet.ImportWalletService
import com.gemwallet.android.data.password.PreferencePasswordStore
import com.gemwallet.android.data.password.TinkSecurityStore
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.wallets.PhraseAddressImportWalletService
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object InteractsModule {

    @Singleton
    @Provides
    fun provideValidateAddressInteract(): ValidateAddressOperator = GemValidateAddressOperator()

    @Singleton
    @Provides
    fun provideValidatePhraseInteract(): ValidatePhraseOperator = GemValidatePhraseOperator()

    @Singleton
    @Provides
    fun provideCreateWalletInteract(): CreateWalletOperator = GemCreateWalletOperator()

    @Singleton
    @Provides
    fun provideCreateAccountInteract(
        @ApplicationContext context: Context,
    ): CreateAccountOperator = GemCreateAccountOperator(context.dataDir.toString())

    @Singleton
    @Provides
    fun provideAddAccountsInteract(
        @ApplicationContext context: Context,
    ): AddAccountsOperator = GemAddAccountsOperator(context.dataDir.toString())

    @Singleton
    @Provides
    fun provideMigrateKeystoreOperator(
        @ApplicationContext context: Context,
    ): MigrateKeystoreOperator = GemMigrateKeystoreOperator(context.dataDir.toString())

    @Singleton
    @Provides
    fun provideStorePhraseInteract(
        @ApplicationContext context: Context
    ): StorePhraseOperator =
        GemStorePhraseOperator(context.dataDir.toString())

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
    fun provideSignAuthOperator(
        @ApplicationContext context: Context,
    ): GemSignAuthOperator = GemSignAuthOperator(context.dataDir.toString())

    @Singleton
    @Provides
    fun provideDeleteKeyStoreOperator(
        @ApplicationContext context: Context,
        passwordStore: PasswordStore,
    ): DeleteKeyStoreOperator = GemDeleteKeyStoreOperator(context.dataDir.toString(), passwordStore)

    @Provides
    @Singleton
    fun providePasswordStore(@ApplicationContext context: Context): PasswordStore =
        PreferencePasswordStore(context)

    @Provides
    @Singleton
    fun provideSecurityStore(@ApplicationContext context: Context): SecurityStore<Any> =
        TinkSecurityStore(context)

    @Singleton
    @Provides
    fun provideAddWalletInteract(
        walletsRepository: WalletsRepository,
        assetsRepository: AssetsRepository,
        sessionRepository: SessionRepository,
        storePhraseOperator: StorePhraseOperator,
        phraseValidate: ValidatePhraseOperator,
        addressValidate: ValidateAddressOperator,
        passwordStore: PasswordStore,
        syncSubscription: SyncSubscription,
        walletImportSync: SyncWalletImport,
    ): ImportWalletService = PhraseAddressImportWalletService(
        walletsRepository = walletsRepository,
        assetsRepository = assetsRepository,
        sessionRepository = sessionRepository,
        storePhraseOperator = storePhraseOperator,
        phraseValidate = phraseValidate,
        addressValidate = addressValidate,
        passwordStore = passwordStore,
        syncSubscription = syncSubscription,
        walletImportSync = walletImportSync,
    )
}
