package com.gemwallet.android.di

import android.content.Context
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ui.R
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemLocalizer
import javax.inject.Singleton

class GemstoneLocalizer(
    private val context: Context,
) : GemLocalizer {

    override fun text(text: GemLocalizedText): String = when (text) {
        is GemLocalizedText.WalletDefaultName -> context.getString(R.string.wallet_default_name, text.index)
        is GemLocalizedText.WalletDefaultNameChain ->
            context.getString(R.string.wallet_default_name_chain, text.chain.requireChain().asset().name, text.index)
    }
}

@InstallIn(SingletonComponent::class)
@Module
object LocalizationModule {
    @Provides
    @Singleton
    fun provideGemLocalizer(@ApplicationContext context: Context): GemLocalizer = GemstoneLocalizer(context)
}
