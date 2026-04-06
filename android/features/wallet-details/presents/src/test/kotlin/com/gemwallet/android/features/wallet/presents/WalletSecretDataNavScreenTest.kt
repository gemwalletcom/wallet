package com.gemwallet.android.features.wallet.presents

import com.gemwallet.android.ui.R
import com.wallet.core.primitives.WalletType
import org.junit.Assert.assertEquals
import org.junit.Test

class WalletSecretDataNavScreenTest {

    @Test
    fun phraseContent_usesSecretPhraseCopy() {
        val content = walletSecretDataContent(walletType = WalletType.Multicoin)

        assertEquals(R.string.common_secret_phrase, content.titleRes)
        assertEquals(R.string.secret_phrase_do_not_share_title, content.warningTitleRes)
        assertEquals(R.string.secret_phrase_do_not_share_description, content.warningDescriptionRes)
    }

    @Test
    fun privateKeyContent_usesPrivateKeyCopy() {
        val content = walletSecretDataContent(walletType = WalletType.PrivateKey)

        assertEquals(R.string.common_private_key, content.titleRes)
        assertEquals(R.string.secret_phrase_do_not_share_title, content.warningTitleRes)
        assertEquals(R.string.secret_phrase_do_not_share_description, content.warningDescriptionRes)
    }
}
