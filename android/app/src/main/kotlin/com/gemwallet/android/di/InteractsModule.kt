package com.gemwallet.android.di

import android.content.Context
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.SecurityStore
import com.gemwallet.android.blockchain.operators.MigrateKeystoreOperator
import com.gemwallet.android.blockchain.operators.ValidatePhraseOperator
import com.gemwallet.android.blockchain.operators.gemstone.GemMigrateKeystoreOperator
import com.gemwallet.android.blockchain.operators.gemstone.GemFindPhraseWord
import com.gemwallet.android.blockchain.operators.gemstone.GemValidatePhraseOperator
import com.gemwallet.android.blockchain.services.KeystoreTransactionSigner
import uniffi.gemstone.GemTransactionSigner
import com.gemwallet.android.data.password.TinkPasswordStore
import com.gemwallet.android.data.password.TinkSecurityStore
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
    fun provideValidatePhraseInteract(): ValidatePhraseOperator = GemValidatePhraseOperator()

    @Singleton
    @Provides
    fun provideFindPhraseWord(): GemFindPhraseWord = GemFindPhraseWord()

    @Singleton
    @Provides
    fun provideMigrateKeystoreOperator(
        @ApplicationContext context: Context,
    ): MigrateKeystoreOperator = GemMigrateKeystoreOperator(context.dataDir.toString())

    @Singleton
    @Provides
    fun provideTransactionSigner(
        @ApplicationContext context: Context,
        passwordStore: PasswordStore,
    ): GemTransactionSigner = KeystoreTransactionSigner(context.dataDir.toString(), passwordStore)

    @Provides
    @Singleton
    fun providePasswordStore(@ApplicationContext context: Context): PasswordStore =
        TinkPasswordStore(context)

    @Provides
    @Singleton
    fun provideSecurityStore(@ApplicationContext context: Context): SecurityStore<Any> =
        TinkSecurityStore(context)

}
